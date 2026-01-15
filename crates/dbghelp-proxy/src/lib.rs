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

static NAMES: [&CStr; 242] = [
    c"DbgHelpCreateUserDump",
    c"DbgHelpCreateUserDumpW",
    c"EnumDirTree",
    c"EnumDirTreeW",
    c"EnumerateLoadedModules",
    c"EnumerateLoadedModules64",
    c"EnumerateLoadedModulesEx",
    c"EnumerateLoadedModulesExW",
    c"EnumerateLoadedModulesW64",
    c"ExtensionApiVersion",
    c"FindDebugInfoFile",
    c"FindDebugInfoFileEx",
    c"FindDebugInfoFileExW",
    c"FindExecutableImage",
    c"FindExecutableImageEx",
    c"FindExecutableImageExW",
    c"FindFileInPath",
    c"FindFileInSearchPath",
    c"GetSymLoadError",
    c"GetTimestampForLoadedLibrary",
    c"ImageDirectoryEntryToData",
    c"ImageDirectoryEntryToDataEx",
    c"ImageNtHeader",
    c"ImageRvaToSection",
    c"ImageRvaToVa",
    c"ImagehlpApiVersion",
    c"ImagehlpApiVersionEx",
    c"MakeSureDirectoryPathExists",
    c"MiniDumpReadDumpStream",
    c"MiniDumpWriteDump",
    c"RangeMapAddPeImageSections",
    c"RangeMapCreate",
    c"RangeMapFree",
    c"RangeMapRead",
    c"RangeMapRemove",
    c"RangeMapWrite",
    c"RemoveInvalidModuleList",
    c"ReportSymbolLoadSummary",
    c"SearchTreeForFile",
    c"SearchTreeForFileW",
    c"SetCheckUserInterruptShared",
    c"SetSymLoadError",
    c"StackWalk",
    c"StackWalk64",
    c"StackWalkEx",
    c"SymAddSourceStream",
    c"SymAddSourceStreamA",
    c"SymAddSourceStreamW",
    c"SymAddSymbol",
    c"SymAddSymbolW",
    c"SymAddrIncludeInlineTrace",
    c"SymAllocDiaString",
    c"SymCleanup",
    c"SymCompareInlineTrace",
    c"SymDeleteSymbol",
    c"SymDeleteSymbolW",
    c"SymEnumLines",
    c"SymEnumLinesW",
    c"SymEnumProcesses",
    c"SymEnumSourceFileTokens",
    c"SymEnumSourceFiles",
    c"SymEnumSourceFilesW",
    c"SymEnumSourceLines",
    c"SymEnumSourceLinesW",
    c"SymEnumSym",
    c"SymEnumSymbols",
    c"SymEnumSymbolsEx",
    c"SymEnumSymbolsExW",
    c"SymEnumSymbolsForAddr",
    c"SymEnumSymbolsForAddrW",
    c"SymEnumSymbolsW",
    c"SymEnumTypes",
    c"SymEnumTypesByName",
    c"SymEnumTypesByNameW",
    c"SymEnumTypesW",
    c"SymEnumerateModules",
    c"SymEnumerateModules64",
    c"SymEnumerateModulesW64",
    c"SymEnumerateSymbols",
    c"SymEnumerateSymbols64",
    c"SymEnumerateSymbolsW",
    c"SymEnumerateSymbolsW64",
    c"SymFindDebugInfoFile",
    c"SymFindDebugInfoFileW",
    c"SymFindExecutableImage",
    c"SymFindExecutableImageW",
    c"SymFindFileInPath",
    c"SymFindFileInPathW",
    c"SymFreeDiaString",
    c"SymFromAddr",
    c"SymFromAddrW",
    c"SymFromIndex",
    c"SymFromIndexW",
    c"SymFromInlineContext",
    c"SymFromInlineContextW",
    c"SymFromName",
    c"SymFromNameW",
    c"SymFromToken",
    c"SymFromTokenW",
    c"SymFunctionTableAccess",
    c"SymFunctionTableAccess64",
    c"SymFunctionTableAccess64AccessRoutines",
    c"SymGetDiaSession",
    c"SymGetExtendedOption",
    c"SymGetFileLineOffsets64",
    c"SymGetHomeDirectory",
    c"SymGetHomeDirectoryW",
    c"SymGetLineFromAddr",
    c"SymGetLineFromAddr64",
    c"SymGetLineFromAddrEx",
    c"SymGetLineFromAddrW64",
    c"SymGetLineFromInlineContext",
    c"SymGetLineFromInlineContextW",
    c"SymGetLineFromName",
    c"SymGetLineFromName64",
    c"SymGetLineFromNameEx",
    c"SymGetLineFromNameW64",
    c"SymGetLineNext",
    c"SymGetLineNext64",
    c"SymGetLineNextEx",
    c"SymGetLineNextW64",
    c"SymGetLinePrev",
    c"SymGetLinePrev64",
    c"SymGetLinePrevEx",
    c"SymGetLinePrevW64",
    c"SymGetModuleBase",
    c"SymGetModuleBase64",
    c"SymGetModuleInfo",
    c"SymGetModuleInfo64",
    c"SymGetModuleInfoW",
    c"SymGetModuleInfoW64",
    c"SymGetOmapBlockBase",
    c"SymGetOmaps",
    c"SymGetOptions",
    c"SymGetScope",
    c"SymGetScopeW",
    c"SymGetSearchPath",
    c"SymGetSearchPathW",
    c"SymGetSourceFile",
    c"SymGetSourceFileChecksum",
    c"SymGetSourceFileChecksumW",
    c"SymGetSourceFileFromToken",
    c"SymGetSourceFileFromTokenW",
    c"SymGetSourceFileToken",
    c"SymGetSourceFileTokenW",
    c"SymGetSourceFileW",
    c"SymGetSourceVarFromToken",
    c"SymGetSourceVarFromTokenW",
    c"SymGetSymFromAddr",
    c"SymGetSymFromAddr64",
    c"SymGetSymFromName",
    c"SymGetSymFromName64",
    c"SymGetSymNext",
    c"SymGetSymNext64",
    c"SymGetSymPrev",
    c"SymGetSymPrev64",
    c"SymGetSymbolFile",
    c"SymGetSymbolFileW",
    c"SymGetTypeFromName",
    c"SymGetTypeFromNameW",
    c"SymGetTypeInfo",
    c"SymGetTypeInfoEx",
    c"SymGetUnwindInfo",
    c"SymInitialize",
    c"SymInitializeW",
    c"SymLoadModule",
    c"SymLoadModule64",
    c"SymLoadModuleEx",
    c"SymLoadModuleExW",
    c"SymMatchFileName",
    c"SymMatchFileNameW",
    c"SymMatchString",
    c"SymMatchStringA",
    c"SymMatchStringW",
    c"SymNext",
    c"SymNextW",
    c"SymPrev",
    c"SymPrevW",
    c"SymQueryInlineTrace",
    c"SymRefreshModuleList",
    c"SymRegisterCallback",
    c"SymRegisterCallback64",
    c"SymRegisterCallbackW64",
    c"SymRegisterFunctionEntryCallback",
    c"SymRegisterFunctionEntryCallback64",
    c"SymSearch",
    c"SymSearchW",
    c"SymSetContext",
    c"SymSetDiaSession",
    c"SymSetExtendedOption",
    c"SymSetHomeDirectory",
    c"SymSetHomeDirectoryW",
    c"SymSetOptions",
    c"SymSetParentWindow",
    c"SymSetScopeFromAddr",
    c"SymSetScopeFromIndex",
    c"SymSetScopeFromInlineContext",
    c"SymSetSearchPath",
    c"SymSetSearchPathW",
    c"SymSrvDeltaName",
    c"SymSrvDeltaNameW",
    c"SymSrvGetFileIndexInfo",
    c"SymSrvGetFileIndexInfoW",
    c"SymSrvGetFileIndexString",
    c"SymSrvGetFileIndexStringW",
    c"SymSrvGetFileIndexes",
    c"SymSrvGetFileIndexesW",
    c"SymSrvGetSupplement",
    c"SymSrvGetSupplementW",
    c"SymSrvIsStore",
    c"SymSrvIsStoreW",
    c"SymSrvStoreFile",
    c"SymSrvStoreFileW",
    c"SymSrvStoreSupplement",
    c"SymSrvStoreSupplementW",
    c"SymUnDName",
    c"SymUnDName64",
    c"SymUnloadModule",
    c"SymUnloadModule64",
    c"UnDecorateSymbolName",
    c"UnDecorateSymbolNameW",
    c"WinDbgExtensionDllInit",
    c"_EFN_DumpImage",
    c"block",
    c"chksym",
    c"dbghelp",
    c"dh",
    c"fptr",
    c"homedir",
    c"inlinedbg",
    c"itoldyouso",
    c"lmi",
    c"lminfo",
    c"omap",
    c"optdbgdump",
    c"optdbgdumpaddr",
    c"srcfiles",
    c"stack_force_ebp",
    c"stackdbg",
    c"sym",
    c"symsrv",
    c"vc7fpo",
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

forward!(DbgHelpCreateUserDump, 0);
forward!(DbgHelpCreateUserDumpW, 1);
forward!(EnumDirTree, 2);
forward!(EnumDirTreeW, 3);
forward!(EnumerateLoadedModules, 4);
forward!(EnumerateLoadedModules64, 5);
forward!(EnumerateLoadedModulesEx, 6);
forward!(EnumerateLoadedModulesExW, 7);
forward!(EnumerateLoadedModulesW64, 8);
forward!(ExtensionApiVersion, 9);
forward!(FindDebugInfoFile, 10);
forward!(FindDebugInfoFileEx, 11);
forward!(FindDebugInfoFileExW, 12);
forward!(FindExecutableImage, 13);
forward!(FindExecutableImageEx, 14);
forward!(FindExecutableImageExW, 15);
forward!(FindFileInPath, 16);
forward!(FindFileInSearchPath, 17);
forward!(GetSymLoadError, 18);
forward!(GetTimestampForLoadedLibrary, 19);
forward!(ImageDirectoryEntryToData, 20);
forward!(ImageDirectoryEntryToDataEx, 21);
forward!(ImageNtHeader, 22);
forward!(ImageRvaToSection, 23);
forward!(ImageRvaToVa, 24);
forward!(ImagehlpApiVersion, 25);
forward!(ImagehlpApiVersionEx, 26);
forward!(MakeSureDirectoryPathExists, 27);
forward!(MiniDumpReadDumpStream, 28);
forward!(MiniDumpWriteDump, 29);
forward!(RangeMapAddPeImageSections, 30);
forward!(RangeMapCreate, 31);
forward!(RangeMapFree, 32);
forward!(RangeMapRead, 33);
forward!(RangeMapRemove, 34);
forward!(RangeMapWrite, 35);
forward!(RemoveInvalidModuleList, 36);
forward!(ReportSymbolLoadSummary, 37);
forward!(SearchTreeForFile, 38);
forward!(SearchTreeForFileW, 39);
forward!(SetCheckUserInterruptShared, 40);
forward!(SetSymLoadError, 41);
forward!(StackWalk, 42);
forward!(StackWalk64, 43);
forward!(StackWalkEx, 44);
forward!(SymAddSourceStream, 45);
forward!(SymAddSourceStreamA, 46);
forward!(SymAddSourceStreamW, 47);
forward!(SymAddSymbol, 48);
forward!(SymAddSymbolW, 49);
forward!(SymAddrIncludeInlineTrace, 50);
forward!(SymAllocDiaString, 51);
forward!(SymCleanup, 52);
forward!(SymCompareInlineTrace, 53);
forward!(SymDeleteSymbol, 54);
forward!(SymDeleteSymbolW, 55);
forward!(SymEnumLines, 56);
forward!(SymEnumLinesW, 57);
forward!(SymEnumProcesses, 58);
forward!(SymEnumSourceFileTokens, 59);
forward!(SymEnumSourceFiles, 60);
forward!(SymEnumSourceFilesW, 61);
forward!(SymEnumSourceLines, 62);
forward!(SymEnumSourceLinesW, 63);
forward!(SymEnumSym, 64);
forward!(SymEnumSymbols, 65);
forward!(SymEnumSymbolsEx, 66);
forward!(SymEnumSymbolsExW, 67);
forward!(SymEnumSymbolsForAddr, 68);
forward!(SymEnumSymbolsForAddrW, 69);
forward!(SymEnumSymbolsW, 70);
forward!(SymEnumTypes, 71);
forward!(SymEnumTypesByName, 72);
forward!(SymEnumTypesByNameW, 73);
forward!(SymEnumTypesW, 74);
forward!(SymEnumerateModules, 75);
forward!(SymEnumerateModules64, 76);
forward!(SymEnumerateModulesW64, 77);
forward!(SymEnumerateSymbols, 78);
forward!(SymEnumerateSymbols64, 79);
forward!(SymEnumerateSymbolsW, 80);
forward!(SymEnumerateSymbolsW64, 81);
forward!(SymFindDebugInfoFile, 82);
forward!(SymFindDebugInfoFileW, 83);
forward!(SymFindExecutableImage, 84);
forward!(SymFindExecutableImageW, 85);
forward!(SymFindFileInPath, 86);
forward!(SymFindFileInPathW, 87);
forward!(SymFreeDiaString, 88);
forward!(SymFromAddr, 89);
forward!(SymFromAddrW, 90);
forward!(SymFromIndex, 91);
forward!(SymFromIndexW, 92);
forward!(SymFromInlineContext, 93);
forward!(SymFromInlineContextW, 94);
forward!(SymFromName, 95);
forward!(SymFromNameW, 96);
forward!(SymFromToken, 97);
forward!(SymFromTokenW, 98);
forward!(SymFunctionTableAccess, 99);
forward!(SymFunctionTableAccess64, 100);
forward!(SymFunctionTableAccess64AccessRoutines, 101);
forward!(SymGetDiaSession, 102);
forward!(SymGetExtendedOption, 103);
forward!(SymGetFileLineOffsets64, 104);
forward!(SymGetHomeDirectory, 105);
forward!(SymGetHomeDirectoryW, 106);
forward!(SymGetLineFromAddr, 107);
forward!(SymGetLineFromAddr64, 108);
forward!(SymGetLineFromAddrEx, 109);
forward!(SymGetLineFromAddrW64, 110);
forward!(SymGetLineFromInlineContext, 111);
forward!(SymGetLineFromInlineContextW, 112);
forward!(SymGetLineFromName, 113);
forward!(SymGetLineFromName64, 114);
forward!(SymGetLineFromNameEx, 115);
forward!(SymGetLineFromNameW64, 116);
forward!(SymGetLineNext, 117);
forward!(SymGetLineNext64, 118);
forward!(SymGetLineNextEx, 119);
forward!(SymGetLineNextW64, 120);
forward!(SymGetLinePrev, 121);
forward!(SymGetLinePrev64, 122);
forward!(SymGetLinePrevEx, 123);
forward!(SymGetLinePrevW64, 124);
forward!(SymGetModuleBase, 125);
forward!(SymGetModuleBase64, 126);
forward!(SymGetModuleInfo, 127);
forward!(SymGetModuleInfo64, 128);
forward!(SymGetModuleInfoW, 129);
forward!(SymGetModuleInfoW64, 130);
forward!(SymGetOmapBlockBase, 131);
forward!(SymGetOmaps, 132);
forward!(SymGetOptions, 133);
forward!(SymGetScope, 134);
forward!(SymGetScopeW, 135);
forward!(SymGetSearchPath, 136);
forward!(SymGetSearchPathW, 137);
forward!(SymGetSourceFile, 138);
forward!(SymGetSourceFileChecksum, 139);
forward!(SymGetSourceFileChecksumW, 140);
forward!(SymGetSourceFileFromToken, 141);
forward!(SymGetSourceFileFromTokenW, 142);
forward!(SymGetSourceFileToken, 143);
forward!(SymGetSourceFileTokenW, 144);
forward!(SymGetSourceFileW, 145);
forward!(SymGetSourceVarFromToken, 146);
forward!(SymGetSourceVarFromTokenW, 147);
forward!(SymGetSymFromAddr, 148);
forward!(SymGetSymFromAddr64, 149);
forward!(SymGetSymFromName, 150);
forward!(SymGetSymFromName64, 151);
forward!(SymGetSymNext, 152);
forward!(SymGetSymNext64, 153);
forward!(SymGetSymPrev, 154);
forward!(SymGetSymPrev64, 155);
forward!(SymGetSymbolFile, 156);
forward!(SymGetSymbolFileW, 157);
forward!(SymGetTypeFromName, 158);
forward!(SymGetTypeFromNameW, 159);
forward!(SymGetTypeInfo, 160);
forward!(SymGetTypeInfoEx, 161);
forward!(SymGetUnwindInfo, 162);
forward!(SymInitialize, 163);
forward!(SymInitializeW, 164);
forward!(SymLoadModule, 165);
forward!(SymLoadModule64, 166);
forward!(SymLoadModuleEx, 167);
forward!(SymLoadModuleExW, 168);
forward!(SymMatchFileName, 169);
forward!(SymMatchFileNameW, 170);
forward!(SymMatchString, 171);
forward!(SymMatchStringA, 172);
forward!(SymMatchStringW, 173);
forward!(SymNext, 174);
forward!(SymNextW, 175);
forward!(SymPrev, 176);
forward!(SymPrevW, 177);
forward!(SymQueryInlineTrace, 178);
forward!(SymRefreshModuleList, 179);
forward!(SymRegisterCallback, 180);
forward!(SymRegisterCallback64, 181);
forward!(SymRegisterCallbackW64, 182);
forward!(SymRegisterFunctionEntryCallback, 183);
forward!(SymRegisterFunctionEntryCallback64, 184);
forward!(SymSearch, 185);
forward!(SymSearchW, 186);
forward!(SymSetContext, 187);
forward!(SymSetDiaSession, 188);
forward!(SymSetExtendedOption, 189);
forward!(SymSetHomeDirectory, 190);
forward!(SymSetHomeDirectoryW, 191);
forward!(SymSetOptions, 192);
forward!(SymSetParentWindow, 193);
forward!(SymSetScopeFromAddr, 194);
forward!(SymSetScopeFromIndex, 195);
forward!(SymSetScopeFromInlineContext, 196);
forward!(SymSetSearchPath, 197);
forward!(SymSetSearchPathW, 198);
forward!(SymSrvDeltaName, 199);
forward!(SymSrvDeltaNameW, 200);
forward!(SymSrvGetFileIndexInfo, 201);
forward!(SymSrvGetFileIndexInfoW, 202);
forward!(SymSrvGetFileIndexString, 203);
forward!(SymSrvGetFileIndexStringW, 204);
forward!(SymSrvGetFileIndexes, 205);
forward!(SymSrvGetFileIndexesW, 206);
forward!(SymSrvGetSupplement, 207);
forward!(SymSrvGetSupplementW, 208);
forward!(SymSrvIsStore, 209);
forward!(SymSrvIsStoreW, 210);
forward!(SymSrvStoreFile, 211);
forward!(SymSrvStoreFileW, 212);
forward!(SymSrvStoreSupplement, 213);
forward!(SymSrvStoreSupplementW, 214);
forward!(SymUnDName, 215);
forward!(SymUnDName64, 216);
forward!(SymUnloadModule, 217);
forward!(SymUnloadModule64, 218);
forward!(UnDecorateSymbolName, 219);
forward!(UnDecorateSymbolNameW, 220);
forward!(WinDbgExtensionDllInit, 221);
forward!(_EFN_DumpImage, 222);
forward!(block, 223);
forward!(chksym, 224);
forward!(dbghelp, 225);
forward!(dh, 226);
forward!(fptr, 227);
forward!(homedir, 228);
forward!(inlinedbg, 229);
forward!(itoldyouso, 230);
forward!(lmi, 231);
forward!(lminfo, 232);
forward!(omap, 233);
forward!(optdbgdump, 234);
forward!(optdbgdumpaddr, 235);
forward!(srcfiles, 236);
forward!(stack_force_ebp, 237);
forward!(stackdbg, 238);
forward!(sym, 239);
forward!(symsrv, 240);
forward!(vc7fpo, 241);

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

                    init::init(config);
                }
                Err(e) => {
                    panic!("Failed to initialize dbghelp proxy: {e}");
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

        const NAME: &[u8] = c"\\dbghelp.dll".to_bytes_with_nul();

        buf[len as usize..len as usize + NAME.len()]
            .copy_from_slice(NAME);

        match LoadLibraryA(PCSTR::from_raw(buf.as_ptr() as _)) {
            Ok(h) => h,
            Err(e) => {
                let str = CStr::from_bytes_until_nul(&buf)
                    .unwrap_or(c"dbghelp.dll from system directory");

                return Err(format!("Could not load {str:?}: {e}"));
            }
        }
    };

    if dll.is_invalid() {
        return Err("Could not load dbghelp.dll".into());
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
                println!("Warning: {} is missing in dbghelp.dll", NAMES[i].to_string_lossy());

                unsafe {
                    M_PROCS[i] = missing_proc as _;
                }
            }
        };
    }

    Ok(())
}

fn missing_proc() {
    panic!("called a missing function on dbghelp.dll");
}
