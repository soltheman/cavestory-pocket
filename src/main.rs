#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::exit;

#[cfg(not(target_os = "ios"))]
use clap::Parser;

#[cfg(target_os = "ios")]
fn main() {
    use std::ffi::{c_char, c_int, CString};

    extern "C" {
        // SDL's UIKit entry point: runs UIApplicationMain and calls `main_function` once the app has launched.
        fn SDL_UIKitRunApp(
            argc: c_int,
            argv: *mut *mut c_char,
            main_function: extern "C" fn(c_int, *mut *mut c_char) -> c_int,
        ) -> c_int;
    }

    extern "C" fn sdl_main(_argc: c_int, _argv: *mut *mut c_char) -> c_int {
        match doukutsu_rs::game::init(doukutsu_rs::game::LaunchOptions::default()) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("Initialization error: {}", e);
                1
            }
        }
    }

    let args: Vec<CString> = std::env::args().map(|a| CString::new(a).unwrap_or_default()).collect();
    let mut argv: Vec<*mut c_char> = args.iter().map(|a| a.as_ptr() as *mut c_char).collect();
    argv.push(std::ptr::null_mut());

    let code = unsafe { SDL_UIKitRunApp(args.len() as c_int, argv.as_mut_ptr(), sdl_main) };
    exit(code);
}

#[cfg(not(target_os = "ios"))]
fn main() {
    let options = doukutsu_rs::game::LaunchOptions::parse();

    let result = doukutsu_rs::game::init(options);

    #[cfg(target_os = "windows")]
    unsafe {
        use std::ffi::OsStr;
        use std::os::windows::prelude::*;
        use winapi::_core::ptr::null_mut;
        use winapi::shared::ntdef::LPCWSTR;
        use winapi::um::winuser::MessageBoxW;
        use winapi::um::winuser::MB_OK;

        if let Err(e) = result {
            let title = OsStr::new("Error!").encode_wide().chain(Some(0)).collect::<Vec<u16>>();
            let message = OsStr::new(format!("Whoops, doukutsu-rs crashed: {}", e).as_str())
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<u16>>();

            MessageBoxW(null_mut(), message.as_ptr() as LPCWSTR, title.as_ptr() as LPCWSTR, MB_OK);
            exit(1);
        }
    }

    if let Err(e) = result {
        eprintln!("Initialization error: {}", e);
        exit(1);
    }
}
