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
    Recovering,
    Failed,
}

/// Checks every file installed by the release before its native code is loaded.
/// A missing or invalid manifest also requires repairing the installation.
pub fn missing_required_files(root: &Path) -> Vec<String> {
    use std::path::Component;

    let manifest = fs::read(root.join("shroudforge/version.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let Some(manifest_paths) = manifest
        .as_ref()
        .and_then(|value| value["managedPaths"].as_array())
    else {
        return vec!["shroudforge/version.json".into()];
    };
    let Some(manifest_paths) = manifest_paths
        .iter()
        .map(serde_json::Value::as_str)
        .collect::<Option<Vec<_>>>()
    else {
        return vec!["shroudforge/version.json".into()];
    };
    if manifest_paths.is_empty() {
        return vec!["shroudforge/version.json".into()];
    }
    let mut paths = manifest_paths
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if !paths.iter().any(|path| path == "shroudforge/version.json") {
        paths.push("shroudforge/version.json".into());
    }
    if paths.iter().any(|path| {
        let path = Path::new(path);
        path.as_os_str().is_empty()
            || !path
                .components()
                .all(|component| matches!(component, Component::Normal(_)))
    }) {
        return vec!["shroudforge/version.json".into()];
    }
    let mods = configured_directory(root, "mods", "mods");
    paths
        .into_iter()
        .filter(|path| {
            let installed = path
                .strip_prefix("mods/")
                .map(|relative| mods.join(relative))
                .unwrap_or_else(|| root.join(path));
            !installed.is_file()
        })
        .collect()
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
    let bootstrap = root.join("winmm.dll");
    let missing = missing_required_files(&root);
    if !missing.is_empty() {
        return failed_handoff(
            Some(bootstrap),
            format!(
                "installation requires repair, missing or invalid files: {}",
                missing.join(", ")
            ),
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
    #[cfg(windows)]
    {
        queue_recovery(message)
    }
    #[cfg(not(windows))]
    {
        Handoff::Failed
    }
}

#[cfg(windows)]
fn queue_recovery(message: String) -> Handoff {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError},
        System::Threading::{CreateMutexW, CreateThread, GetCurrentProcessId},
    };

    // Each proxy links this bridge separately. Keep one named handle alive until
    // process exit so they cannot start competing recovery workers.
    let name = format!("Local\\ShroudForgeRecovery-{}", unsafe {
        GetCurrentProcessId()
    })
    .encode_utf16()
    .chain(Some(0))
    .collect::<Vec<_>>();
    let mutex = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
    if mutex.is_null() {
        let _ = write_diagnostic(
            'E',
            "native-proxy",
            &format!(
                "Could not reserve recovery worker: Windows error {}",
                unsafe { GetLastError() }
            ),
        );
        return Handoff::Failed;
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe { CloseHandle(mutex) };
        return Handoff::Recovering;
    }

    let context = Box::into_raw(Box::new(message));
    // DllMain must return successfully so the proxy remains loaded. Windows
    // starts this callback only after DLL initialization releases the loader lock.
    // Never wait for the thread here or launch a process/UI from DllMain.
    let thread = unsafe {
        CreateThread(
            std::ptr::null(),
            0,
            Some(recover_installation),
            context.cast(),
            0,
            std::ptr::null_mut(),
        )
    };
    if thread.is_null() {
        let error = unsafe { GetLastError() };
        unsafe {
            drop(Box::from_raw(context));
            CloseHandle(mutex);
        }
        let _ = write_diagnostic(
            'E',
            "native-proxy",
            &format!("Could not start recovery worker: Windows error {error}"),
        );
        return Handoff::Failed;
    }
    unsafe { CloseHandle(thread) };
    Handoff::Recovering
}

#[cfg(windows)]
unsafe extern "system" fn recover_installation(context: *mut std::ffi::c_void) -> u32 {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::{
        Foundation::ERROR_DLL_INIT_FAILED,
        System::Threading::{CREATE_NO_WINDOW, ExitProcess},
        UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MB_SETFOREGROUND, MessageBoxW},
    };

    let message = unsafe { Box::from_raw(context.cast::<String>()) };
    let result: Result<(), String> = (|| {
        let root = game_root().ok_or("Could not determine the game directory".to_owned())?;
        let updater = root.join("shroudforge/shroudforge-updater.exe");
        std::process::Command::new(&updater)
            .args(["--open-window", "--root"])
            .arg(&root)
            .current_dir(&root)
            .creation_flags(CREATE_NO_WINDOW)
            // Recovery owns its dialogs and logging. Do not wait for it or keep
            // the game process alive while it prepares the independent UI.
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|error| format!("Could not start {}: {error}", updater.display()))?;
        Ok(())
    })();
    match result {
        Ok(()) => {
            let _ = write_diagnostic(
                'I',
                "native-proxy",
                "Independent recovery updater launched. Enshrouded is exiting so the installation can be repaired",
            );
        }
        Err(error) => {
            let _ = write_diagnostic('E', "native-proxy", &error);
            let text = format!("{message}\n\n{error}\n\nEnshrouded will close. Restore ShroudForge from its release package before starting the game again.")
                .encode_utf16().chain(Some(0)).collect::<Vec<_>>();
            let title = "ShroudForge startup error"
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>();
            unsafe {
                MessageBoxW(
                    std::ptr::null_mut(),
                    text.as_ptr(),
                    title.as_ptr(),
                    MB_OK | MB_ICONERROR | MB_SETFOREGROUND,
                )
            };
        }
    }
    unsafe { ExitProcess(ERROR_DLL_INIT_FAILED) }
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
