#![windows_subsystem = "windows"]
use anyhow::Result;
use log::{error, info};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicI16, AtomicI32, AtomicU64, Ordering};

mod ui;
mod vkey;
use ui::*;
use vigem_client::*;
use windows::{
    core::*,
    Win32::Devices::HumanInterfaceDevice::*,
    Win32::Foundation::*,
    Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS},
    Win32::System::LibraryLoader::*,
    Win32::UI::Input::*,
    Win32::UI::WindowsAndMessaging::*,
};

static MOVEMENT_MULTIPLIER: AtomicI16 = AtomicI16::new(1400);
static INTERVAL_MICROS: AtomicU64 = AtomicU64::new(2000);

static ENABLE_MOUSE: AtomicBool = AtomicBool::new(true);
static LBUTTONDOWN: AtomicBool = AtomicBool::new(false);
static RBUTTONDOWN: AtomicBool = AtomicBool::new(false);
static START: AtomicBool = AtomicBool::new(false);
static DPADUP: AtomicBool = AtomicBool::new(false);
static DPADDOWN: AtomicBool = AtomicBool::new(false);
static DPADRIGHT: AtomicBool = AtomicBool::new(false);
static DPADLEFT: AtomicBool = AtomicBool::new(false);
static X: AtomicI32 = AtomicI32::new(0);
static Y: AtomicI32 = AtomicI32::new(0);

static CODE_DPAD_L: AtomicI32 = AtomicI32::new(0);
static CODE_DPAD_R: AtomicI32 = AtomicI32::new(0);
static CODE_DPAD_U: AtomicI32 = AtomicI32::new(0);
static CODE_DPAD_D: AtomicI32 = AtomicI32::new(0);
static CODE_LSTICK_L: AtomicI32 = AtomicI32::new(0);
static CODE_LSTICK_R: AtomicI32 = AtomicI32::new(0);
static CODE_LSTICK_U: AtomicI32 = AtomicI32::new(0);
static CODE_LSTICK_D: AtomicI32 = AtomicI32::new(0);
static CODE_BUTTON_A: AtomicI32 = AtomicI32::new(0);
static CODE_BUTTON_B: AtomicI32 = AtomicI32::new(0);
static CODE_BUTTON_X: AtomicI32 = AtomicI32::new(0);
static CODE_BUTTON_Y: AtomicI32 = AtomicI32::new(0);
static CODE_BUTTON_START: AtomicI32 = AtomicI32::new(0);
static CODE_SHOULDER_L: AtomicI32 = AtomicI32::new(0);
static CODE_SHOULDER_R: AtomicI32 = AtomicI32::new(0);

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mouse_enabled = ENABLE_MOUSE.load(Ordering::Relaxed);
    if !mouse_enabled && wparam.0 == WM_MOUSEMOVE as usize {
        return LRESULT { 0: 1 };
    } else if !mouse_enabled && wparam.0 == WM_LBUTTONDOWN as usize {
        LBUTTONDOWN.store(true, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else if !mouse_enabled && wparam.0 == WM_LBUTTONUP as usize {
        LBUTTONDOWN.store(false, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else if !mouse_enabled && wparam.0 == WM_RBUTTONDOWN as usize {
        RBUTTONDOWN.store(true, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else if !mouse_enabled && wparam.0 == WM_RBUTTONUP as usize {
        RBUTTONDOWN.store(false, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else {
        return CallNextHookEx(None, code, wparam, lparam);
    }
}

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let pcode = lparam.0 as *const i32;
    let down = wparam.0 == WM_KEYDOWN as usize;
    let mouse_enabled = ENABLE_MOUSE.load(Ordering::Relaxed);

    // NB handle CAPS differently to these since it must be triggered in or out
    // of mouse mouse.
    if !mouse_enabled {
        let pairs = vec![(&CODE_BUTTON_START, &START)];
        for (code, button) in pairs {
            if *pcode == code.load(Ordering::Relaxed) {
                button.store(down, Ordering::Relaxed);
                return LRESULT { 0: 1 };
            }
        }
    }

    // Caps lock toggle mouse capture
    if *pcode == 0x14 && down {
        info!("Toggled mouse capture.");
        let enabled = ENABLE_MOUSE.load(Ordering::Relaxed);
        ENABLE_MOUSE.store(!enabled, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else if *pcode == 0x70 && down {
        // f1 decreases sensitivity
        let last = MOVEMENT_MULTIPLIER.fetch_sub(100, Ordering::Relaxed);
        info!("Decreased multiplier to {}", last - 100)
    } else if *pcode == 0x71 && down {
        // f2 increases sensitivity
        let last = MOVEMENT_MULTIPLIER.fetch_add(100, Ordering::Relaxed);
        info!("Increased multiplier to {}", last + 100)
    } else if *pcode == 0x25 && down {
        DPADLEFT.store(true, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else if *pcode == 0x25 && !down {
        DPADLEFT.store(false, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else if *pcode == 0x26 && down {
        DPADUP.store(true, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else if *pcode == 0x26 && !down {
        DPADUP.store(false, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else if *pcode == 0x27 && down {
        DPADRIGHT.store(true, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else if *pcode == 0x27 && !down {
        DPADRIGHT.store(false, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else if *pcode == 0x28 && down {
        DPADDOWN.store(true, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else if *pcode == 0x28 && !down {
        DPADDOWN.store(false, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    }

    return CallNextHookEx(None, code, wparam, lparam);
}

fn run_controller(mut gamepad: XGamepad, mut target: Xbox360Wired<Client>) {
    info!("Launching serf controller.");

    info!("Virtual gamepad attached.");
    loop {
        std::thread::sleep(std::time::Duration::from_micros(
            INTERVAL_MICROS.load(Ordering::Relaxed),
        ));
        let start = std::time::Instant::now();
        let multiplier = MOVEMENT_MULTIPLIER.load(Ordering::Relaxed);
        let thumb_rx = i16::saturating_mul(X.swap(0, Ordering::Relaxed) as i16, multiplier);
        let thumb_ry = i16::saturating_mul(Y.swap(0, Ordering::Relaxed) as i16, multiplier);

        gamepad.thumb_rx = thumb_rx;
        gamepad.thumb_ry = thumb_ry;
        if RBUTTONDOWN.load(Ordering::Relaxed) {
            gamepad.left_trigger = 255;
        } else {
            gamepad.left_trigger = 0;
        }
        if LBUTTONDOWN.load(Ordering::Relaxed) {
            gamepad.right_trigger = 255;
        } else {
            gamepad.right_trigger = 0;
        }

        gamepad.buttons = XButtons::default();
        if START.load(Ordering::Relaxed) {
            info!("Start pressed");
            gamepad.buttons.raw = gamepad.buttons.raw | XButtons::START;
        }
        if DPADUP.load(Ordering::Relaxed) {
            gamepad.buttons.raw = gamepad.buttons.raw | XButtons::UP;
        }
        if DPADDOWN.load(Ordering::Relaxed) {
            gamepad.buttons.raw = gamepad.buttons.raw | XButtons::DOWN;
        }
        if DPADRIGHT.load(Ordering::Relaxed) {
            gamepad.buttons.raw = gamepad.buttons.raw | XButtons::RIGHT;
        }
        if DPADLEFT.load(Ordering::Relaxed) {
            gamepad.buttons.raw = gamepad.buttons.raw | XButtons::LEFT;
        }

        target
            .update(&gamepad)
            .expect("should be able to update our gamepad");

        // TODO once mapped all buttons, take this out.
        info!(
            "Processing took {} micros",
            (std::time::Instant::now() - start).as_micros()
        );
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mouse_enabled = ENABLE_MOUSE.load(Ordering::Relaxed);
    unsafe {
        match message {
            WM_INPUT => {
                if mouse_enabled {
                    return LRESULT(0);
                }
                let mut size: u32 = 0;
                // Get required buffer size
                GetRawInputData(
                    HRAWINPUT(lparam.0),
                    RID_INPUT,
                    None,
                    &mut size,
                    std::mem::size_of::<RAWINPUTHEADER>() as u32,
                );
                let mut data: RAWINPUT = std::mem::zeroed();
                let pdata = (&mut data as *mut RAWINPUT) as *mut c_void;
                GetRawInputData(
                    HRAWINPUT(lparam.0),
                    RID_INPUT,
                    Some(pdata),
                    &mut size,
                    std::mem::size_of::<RAWINPUTHEADER>() as u32,
                );

                X.fetch_add(data.data.mouse.lLastX, Ordering::Relaxed);
                Y.fetch_add(data.data.mouse.lLastY, Ordering::Relaxed);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}

fn run_messages() -> Result<()> {
    let id = TargetId::XBOX360_WIRED;
    let client = Client::connect()?;
    let mut target = Xbox360Wired::new(client, id);

    target.plugin()?;
    target.wait_ready()?;
    let gamepad = vigem_client::XGamepad::default();

    let _thread = std::thread::spawn(move || run_controller(gamepad, target));

    unsafe {
        let instance = GetModuleHandleA(None)?;
        debug_assert!(instance.0 != 0);

        let window_class = s!("window");

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
            s!("message"),
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

        // register for raw mouse input
        let inputdevices = vec![RAWINPUTDEVICE {
            usUsage: HID_USAGE_GENERIC_MOUSE,
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            hwndTarget: hwnd,
            dwFlags: RIDEV_INPUTSINK,
        }];
        if !RegisterRawInputDevices(&inputdevices, std::mem::size_of::<RAWINPUTDEVICE>() as u32)
            .as_bool()
        {
            let err = GetLastError();
            error!("Failed to register raw input: {:?}", err);
            std::process::exit(1);
        }

        let _keyboard_hook =
            SetWindowsHookExA(WH_KEYBOARD_LL, Some(keyboard_hook), Some(instance), 0)?;
        let _mouse_hook = SetWindowsHookExA(WH_MOUSE_LL, Some(mouse_hook), Some(instance), 0)?;

        let mut message = MSG::default();
        info!("Waiting for messages");
    }
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(460.0, 340.0)),
        resizable: false,
        ..Default::default()
    };
    eframe::run_native(
        "Serf - the console peasants are revolting",
        options,
        Box::new(|_cc| Box::new(SerfApp::default())),
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

    if let Err(e) = run_messages() {
        error!("{:?}", e);
        unsafe {
            let message = format!("{:?}", e);
            MessageBoxA(
                None,
                Some(PCSTR::from_raw(message.as_ptr())),
                s!("Error"),
                MB_OK,
            );
        }
    }
}
