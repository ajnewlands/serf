#![windows_subsystem = "windows"]
use anyhow::Result;
use image::GenericImageView;
use log::error;

mod ui;
use crossbeam::channel::*;
use once_cell::sync::OnceCell;
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

static CONTEXT: OnceCell<eframe::egui::Context> = OnceCell::new();
static TX: OnceCell<Sender<common::ButtonMapping>> = OnceCell::new();

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
                let pdata: *const COPYDATASTRUCT = lparam.0 as *const u8 as *const COPYDATASTRUCT;
                let pbmap = (*pdata).lpData as *mut common::ButtonMapping;
                TX.get()
                    .expect("TX hasn't been initialized.")
                    .send((*pbmap).clone())
                    .expect("Failed to send updated button map to UI");
                CONTEXT
                    .get()
                    .expect("Context hasn't been initialized.")
                    .request_repaint();
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

fn get_icon_data() -> Option<eframe::IconData> {
    let bytes = include_bytes!("../../icons/serf.png");

    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .expect("Embedded icon must be a valid PNG");

    Some(eframe::IconData {
        width: image.dimensions().0,
        height: image.dimensions().1,
        rgba: image.into_bytes(),
    })
}

fn run_frontend() -> Result<()> {
    let configuration = common::Configuration::load()?;
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

        let _hwnd = CreateWindowExA(
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

    let icon_data = get_icon_data();

    // Show the configuration screen
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(460.0, 460.0)),
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        icon_data,
        resizable: false,
        ..Default::default()
    };

    let (tx, rx) = unbounded::<common::ButtonMapping>();
    TX.set(tx)
        .map_err(|_| anyhow::anyhow!("TX already initialized."))?;
    let app = Box::new(SerfApp {
        active_game_index: 0,
        configuration,
        previous: common::ButtonMapping::default(),
        rx: rx,
    });
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
