pub mod logging;
mod init;
mod log;

use std::{arch::naked_asm, ffi::{c_void, CStr}, panic::PanicHookInfo, ptr};
use std::ffi::CString;

use mod_loader::Config;
use windows::{
    Win32::{
        Foundation::{HINSTANCE, MAX_PATH, TRUE},
        System::{
            Console::{AllocConsole, GetConsoleWindow},
            LibraryLoader::{GetProcAddress, LoadLibraryA},
            SystemInformation::GetSystemDirectoryA,
            SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
        },
        UI::WindowsAndMessaging::{MB_OK, MessageBoxA, SW_SHOW, ShowWindow},
    },
    core::{BOOL, PCSTR},
};

static NAMES: [&CStr; 6] = [
    c"DirectInput8Create",
    c"DllCanUnloadNow",
    c"DllGetClassObject",
    c"DllRegisterServer",
    c"DllUnregisterServer",
    c"GetdfDIJoystick",
];

static mut M_PROCS: [*const c_void; NAMES.len()] = [ptr::null(); NAMES.len()];

macro_rules! forward {
    ($name:ident, $idx:expr) => {
        #[unsafe(no_mangle)]
        #[unsafe(naked)]
        unsafe extern "system" fn $name() {
            naked_asm!(
                "jmp [rip + {base} + {offset}]",
                base = sym M_PROCS,
                offset = const $idx * 8,
            );
        }
    };
}

forward!(DirectInput8Create, 0);
forward!(DllCanUnloadNow,    1);
forward!(DllGetClassObject,  2);
forward!(DllRegisterServer,  3);
forward!(DllUnregisterServer,4);
forward!(GetdfDIJoystick,    5);

#[unsafe(no_mangle)]
extern "system" fn DllMain(
    _hinst: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    #[allow(non_snake_case)] // false positive
    match reason {
        DLL_PROCESS_ATTACH => {
            std::panic::set_hook(Box::new(panic_handler));

            match unsafe { init_procs() } {
                Ok(()) => {
                    let enable_console = std::env::var("EML_CONSOLE")
                        .map(|s| matches!(s.to_lowercase().as_str(), "true" | "1" | "yes"))
                        .unwrap_or(false);

                    let config = Config::load("eml.json")
                        .unwrap_or_default();

                    if enable_console || config.enable_console {
                        crate::enable_console();
                    }

                    init::init();
                }
                Err(e) => {
                    panic!("Failed to initialize dinput8 proxy: {e}");
                }
            }
        }
        DLL_PROCESS_DETACH => {
            init::deinit();
        }
        _ => {}
    }

    TRUE
}

pub fn enable_console()  {
    unsafe {
        match AllocConsole() {
            Ok(_) => {
                let _ = ShowWindow(GetConsoleWindow(), SW_SHOW);
            }
            Err(e) => {
                println!("Warning: Could not allocate console: {e}");
            }
        }
    }
}

fn panic_handler(info: &PanicHookInfo) {
    let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        format!("{:?}", info.payload())
    };
    let loc = info.location()
        .map(|l| format!("{}:{}", l.file(), l.line()))
        .unwrap_or_default();

    let full = CString::new(format!("Panic at {loc}:\n{msg}"))
        .unwrap_or_else(|_| CString::new("Panic at unknown location").unwrap());

    unsafe {
        MessageBoxA(
            None,
            PCSTR::from_raw(full.as_ptr() as _),
            PCSTR::from_raw(c"Fatal error".as_ptr() as _),
            MB_OK,
        );
    }
}

unsafe fn init_procs() -> Result<(), String> {
    let dll = unsafe {
        let mut buf = [0u8; MAX_PATH as _];
        let len = GetSystemDirectoryA(Some(&mut buf));

        if len == 0 || len >= MAX_PATH {
            return Err("Failed to get system directory".into());
        }

        const NAME: &[u8] = c"\\dinput8.dll".to_bytes_with_nul();

        buf[len as usize..len as usize + NAME.len()]
            .copy_from_slice(NAME);

        match LoadLibraryA(PCSTR::from_raw(buf.as_ptr() as _)) {
            Ok(h) => h,
            Err(e) => {
                let str = CStr::from_bytes_until_nul(&buf)
                    .unwrap_or(c"dinput8.dll from system directory");

                return Err(format!("Could not load {str:?}: {e}"));
            }
        }
    };

    if dll.is_invalid() {
        return Err("Could not load dinput8.dll".into());
    }

    for i in 0..NAMES.len() {
        let addr = unsafe {
            GetProcAddress(dll, PCSTR::from_raw(NAMES[i].as_ptr() as _))
        };

        match addr {
            Some(addr) => unsafe {
                M_PROCS[i] = addr as _;
            },
            None => {
                println!("Warning: {} is missing in dinput8.dll", NAMES[i].to_string_lossy());

                unsafe {
                    M_PROCS[i] = missing_proc as _;
                }
            }
        };
    }

    Ok(())
}

fn missing_proc() {
    panic!("called a missing function on dinput8.dll");
}
