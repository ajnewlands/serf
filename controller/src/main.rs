#![windows_subsystem = "windows"]
use anyhow::{anyhow, Context, Result};
use log::{error, info};
use std::ffi::c_void;
use std::sync::atomic::Ordering;
mod statics;
use statics::*;

mod controller;
use vigem_client::*;
use windows::{
    core::*,
    Win32::Devices::HumanInterfaceDevice::*,
    Win32::Foundation::*,
    Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS},
    Win32::System::{DataExchange::COPYDATASTRUCT, LibraryLoader::*},
    Win32::UI::Input::*,
    Win32::UI::WindowsAndMessaging::*,
};

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mouse_enabled = ENABLE_MOUSE.load(Ordering::Relaxed);
    if !mouse_enabled && wparam.0 == WM_MOUSEMOVE as usize {
        return LRESULT { 0: 1 };
    } else if !mouse_enabled && wparam.0 == WM_LBUTTONDOWN as usize {
        LBUTTONDOWN.store(true, Ordering::Relaxed);

        if LEFT_AUTOFIRE.load(Ordering::Relaxed) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Cant get systemtime")
                .as_millis() as u64;
            LEFT_DOWN_INSTANT.store(now, Ordering::Relaxed);
        }

        return LRESULT { 0: 1 };
    } else if !mouse_enabled && wparam.0 == WM_LBUTTONUP as usize {
        LBUTTONDOWN.store(false, Ordering::Relaxed);
        return LRESULT { 0: 1 };
    } else if !mouse_enabled && wparam.0 == WM_RBUTTONDOWN as usize {
        RBUTTONDOWN.store(true, Ordering::Relaxed);
        if RIGHT_AUTOFIRE.load(Ordering::Relaxed) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Cant get systemtime")
                .as_millis() as u64;
            RIGHT_DOWN_INSTANT.store(now, Ordering::Relaxed);
        }
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
        let pairs = vec![
            (&CODE_BUTTON_START, &START),
            (&CODE_DPAD_U, &DPADUP),
            (&CODE_DPAD_D, &DPADDOWN),
            (&CODE_DPAD_R, &DPADRIGHT),
            (&CODE_DPAD_L, &DPADLEFT),
            (&CODE_BUTTON_A, &BUTTONA),
            (&CODE_BUTTON_B, &BUTTONB),
            (&CODE_BUTTON_X, &BUTTONX),
            (&CODE_BUTTON_Y, &BUTTONY),
            (&CODE_SHOULDER_L, &SHOULDER_L),
            (&CODE_SHOULDER_R, &SHOULDER_R),
            (&CODE_LSTICK_D, &LSTICKDOWN),
            (&CODE_LSTICK_U, &LSTICKUP),
            (&CODE_LSTICK_R, &LSTICKRIGHT),
            (&CODE_LSTICK_L, &LSTICKLEFT),
        ];
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
        info!("Decreased multiplier to {}", last - 100);
        send_updated_buttonmap();
    } else if *pcode == 0x71 && down {
        // f2 increases sensitivity
        let last = MOVEMENT_MULTIPLIER.fetch_add(100, Ordering::Relaxed);
        info!("Increased multiplier to {}", last + 100);
        send_updated_buttonmap();
    } else if *pcode == 0x74 && down {
        info!("Toggle left auto fire");
        LEFT_AUTOFIRE.fetch_xor(true, Ordering::Relaxed);
        send_updated_buttonmap();
    } else if *pcode == 0x75 && down {
        info!("Toggle right auto fire");
        RIGHT_AUTOFIRE.fetch_xor(true, Ordering::Relaxed);
        send_updated_buttonmap();
    }
    return CallNextHookEx(None, code, wparam, lparam);
}

fn send_updated_buttonmap() {
    unsafe {
        let hwui = FindWindowA(s!("serf-message-window"), s!("serf-frontend"));
        if hwui.0 == 0 {
            exit_with_error(anyhow!("Could not find message sink for front end"));
        }
        let mut data = create_button_map();
        let copydata = COPYDATASTRUCT {
            dwData: common::CopyTypes::ButtonMap as usize,
            cbData: std::mem::size_of::<common::ButtonMapping>() as u32,
            lpData: (&mut data) as *mut common::ButtonMapping as *mut std::ffi::c_void,
        };
        let res = SendMessageA(
            hwui,
            WM_COPYDATA,
            WPARAM(0),
            LPARAM(&copydata as *const COPYDATASTRUCT as isize),
        );
        if res.0 != 1 {
            exit_with_error(anyhow!("Failed dispatch message to sink for front end"));
        }
    }
}

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
    let mouse_enabled = ENABLE_MOUSE.load(Ordering::Relaxed);
    unsafe {
        match message {
            WM_COPYDATA => {
                let pdata: *const COPYDATASTRUCT = lparam.0 as *const u8 as *const COPYDATASTRUCT;
                let pbmap = (*pdata).lpData as *mut common::ButtonMapping;
                statics::apply_button_map(&*pbmap);
                return LRESULT(1);
            }
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

    // Run the actual gamepad thingy.
    let _thread = std::thread::spawn(move || controller::run_controller(gamepad, target));
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
            s!("serf-controller"),
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
    }

    // Now spawn the front end instance.

    let mut dir = std::env::current_exe().context("Couldn't get executable container directory")?;
    dir.pop();
    dir.push("serf-ui.exe");
    let mut child = std::process::Command::new(dir)
        .spawn()
        .context("Failed to launch front end")?;
    std::thread::spawn(move || match child.wait() {
        Ok(_) => {
            info!("Closing due to front end shutdown");
            std::process::exit(0);
        }
        Err(e) => {
            let message = format!("{:?}", e);
            unsafe {
                MessageBoxA(
                    None,
                    Some(PCSTR::from_raw(message.as_ptr())),
                    s!("Error"),
                    MB_OK,
                );
            }
            std::process::exit(1);
        }
    });

    // Now do actual message processing.
    let mut message = MSG::default();
    loop {
        unsafe {
            GetMessageA(&mut message, None, 0, 0);
            TranslateMessage(&mut message);
            DispatchMessageA(&mut message);
        }
    }
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
