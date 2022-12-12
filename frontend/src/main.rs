#![windows_subsystem = "windows"]
use anyhow::Result;
use log::{error, info};

mod ui;
use ui::*;
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS},
    Win32::{
        System::{DataExchange::COPYDATASTRUCT, LibraryLoader::GetModuleHandleA},
        UI::WindowsAndMessaging::*,
    },
};

fn exit_with_error(e: anyhow::Error) {
    unsafe {
        let message = format!("{:?}", e);
        error!("{}", message);
        MessageBoxA(
            None,
            Some(PCSTR::from_raw(message.as_ptr())),
            s!("Error"),
            MB_OK,
        );
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_COPYDATA => {
                info!("copy data");
                return LRESULT(1);
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}

fn run_frontend() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        debug_assert!(instance.0 != 0);

        let window_class = s!("serf-message-window");

        let wc = WNDCLASSA {
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            lpszClassName: window_class,
            style: CS_HREDRAW | CS_VREDRAW,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        };

        let _atom = RegisterClassA(&wc);

        let hwnd = CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            window_class,
            s!("serf-frontend"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            None,
        );
    }

    // Show the configuration screen
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(460.0, 340.0)),
        resizable: false,
        ..Default::default()
    };

    let app = Box::new(SerfApp::default());
    eframe::run_native(
        "Serf - the console peasants are revolting",
        options,
        Box::new(|_cc| app),
    );

    Ok(())
}

fn main() {
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
    // Normally this won't be launched from a console;
    // logging is strictly for development.
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    if let Err(e) = run_frontend() {
        exit_with_error(e);
    }
}
