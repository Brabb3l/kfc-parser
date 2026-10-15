use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::OnceLock,
    time::Instant,
};

static STARTED: OnceLock<Instant> = OnceLock::new();
static SELECTED_BACKEND: OnceLock<Backend> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Eml,
    ShroudForge,
}

/// Reads the selected proxy backend from the game installation config.
/// Without a ShroudForge config, standalone EML keeps its original startup path.
pub fn selected_backend() -> Backend {
    *SELECTED_BACKEND.get_or_init(|| {
        let Some(root) = game_root() else {
            return Backend::Eml;
        };
        let config_path = root.join("shroudforge/config/modloader-config.json");
        let Ok(bytes) = fs::read(config_path) else {
            return Backend::Eml;
        };
        let config = serde_json::from_slice::<serde_json::Value>(&bytes).ok();
        if config
            .as_ref()
            .and_then(|value| value.get("modloader"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|backend| backend.eq_ignore_ascii_case("eml"))
        {
            Backend::Eml
        } else {
            Backend::ShroudForge
        }
    })
}

#[derive(Debug)]
pub enum Handoff {
    EmlSelected,
    Started,
    Failed,
}

/// Starts the configured external backend, or leaves startup to EML.
pub fn try_start() -> Handoff {
    match selected_backend() {
        Backend::Eml => Handoff::EmlSelected,
        Backend::ShroudForge => try_start_shroudforge(),
    }
}

#[cfg(windows)]
fn try_start_shroudforge() -> Handoff {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::{
        Foundation::GetLastError,
        System::LibraryLoader::LoadLibraryW,
    };

    let Some(root) = game_root() else {
        return failed_handoff(None, "could not determine the game directory".to_owned());
    };
    let runtime = root.join("shroudforge/shroudforge-runtime.dll");
    let bootstrap = root.join("winmm.dll");

    if !bootstrap.is_file() {
        return failed_handoff(
            Some(bootstrap),
            "winmm.dll bootstrap is missing".to_owned(),
        );
    }
    if !runtime.is_file() {
        return failed_handoff(
            Some(bootstrap),
            "shroudforge-runtime.dll is missing".to_owned(),
        );
    }

    let wide_path = bootstrap
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let handle = unsafe { LoadLibraryW(wide_path.as_ptr()) };
    if !handle.is_null() {
        return Handoff::Started;
    }

    let error = format!("LoadLibraryW failed with Windows error {}", unsafe {
        GetLastError()
    });
    failed_handoff(Some(bootstrap), error)
}

#[cfg(not(windows))]
fn try_start_shroudforge() -> Handoff {
    failed_handoff(None, "ShroudForge mode requires Windows".to_owned())
}

fn failed_handoff(bootstrap: Option<PathBuf>, error: String) -> Handoff {
    let path = bootstrap
        .as_ref()
        .map(|path| format!(" at {}", path.display()))
        .unwrap_or_default();
    let message = format!(
        "ShroudForge mode is enabled, but bootstrap{path} failed: {error}. EML was not started"
    );
    let _ = write_diagnostic('E', "native-proxy", &message);
    Handoff::Failed
}

/// Writes a diagnostic for ShroudForge to consume before its logger starts.
pub fn write_diagnostic(level: char, source: &str, message: &str) -> bool {
    if selected_backend() != Backend::ShroudForge {
        return false;
    }

    let Some(root) = game_root() else {
        return false;
    };

    if !allows(&root, level) {
        return true;
    }

    let elapsed = STARTED.get_or_init(Instant::now).elapsed();
    let source = source
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace(']', "_");
    let message = message.replace('\r', "\\r").replace('\n', "\\n");
    let line = format!(
        "[{level} {:02}:{:02}:{:02},{:03}] [{source}] {message}\n",
        elapsed.as_secs() / 3600,
        elapsed.as_secs() / 60 % 60,
        elapsed.as_secs() % 60,
        elapsed.subsec_millis()
    );

    #[cfg(windows)]
    {
        use windows_sys::Win32::{
            Foundation::{CloseHandle, WAIT_ABANDONED, WAIT_OBJECT_0},
            System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject},
        };

        let mutex_name: Vec<u16> = "Local\\ShroudForgeLog"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mutex = unsafe { CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr()) };
        if mutex.is_null() {
            return append_line(&pending_path(&root), line.as_bytes()).is_ok();
        }
        let wait = unsafe { WaitForSingleObject(mutex, u32::MAX) };
        if wait != WAIT_OBJECT_0 && wait != WAIT_ABANDONED {
            unsafe { CloseHandle(mutex) };
            return append_line(&pending_path(&root), line.as_bytes()).is_ok();
        }

        let result = append_line(&pending_path(&root), line.as_bytes());
        unsafe {
            ReleaseMutex(mutex);
            CloseHandle(mutex);
        }
        return result.is_ok();
    }

    #[cfg(not(windows))]
    {
        let _ = (root, line);
        false
    }
}

fn game_root() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    executable.parent().map(Path::to_path_buf)
}

fn allows(root: &Path, level: char) -> bool {
    let minimum = fs::read(root.join("shroudforge/config/modloader-config.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| {
            value
                .pointer("/logging/minimumLevel")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "INFO".to_owned());
    let minimum = match minimum.to_ascii_uppercase().as_str() {
        "TRACE" | "ALL" => 0,
        "DEBUG" => 1,
        "INFO" => 2,
        "WARN" | "WARNING" => 3,
        "ERROR" => 4,
        _ => 2,
    };
    let level = match level {
        'T' => 0,
        'D' => 1,
        'I' => 2,
        'W' => 3,
        'E' => 4,
        _ => 2,
    };
    level >= minimum
}

fn configured_directory(root: &Path, key: &str, default: &str) -> PathBuf {
    fs::read(root.join("shroudforge/config/modloader-config.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|value| {
            value
                .pointer(&format!("/paths/{key}"))
                .and_then(serde_json::Value::as_str)
                .map(PathBuf::from)
        })
        .map(|path| if path.is_absolute() { path } else { root.join(path) })
        .unwrap_or_else(|| root.join(default))
}

fn pending_path(root: &Path) -> PathBuf {
    let server = std::env::current_exe().ok().is_some_and(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("enshrouded_server.exe"))
    });
    let role = if server
        || (root.join("enshrouded_server.exe").is_file()
            && !root.join("enshrouded.exe").is_file())
    {
        "server"
    } else {
        "client"
    };
    let logs = configured_directory(root, "logs", "shroudforge/logs");
    logs.join(format!("native-proxy-{role}.pending"))
}

fn append_line(path: &Path, line: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(line)?;
    file.flush()
}
