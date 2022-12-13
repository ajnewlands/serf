use std::sync::atomic::{AtomicBool, AtomicI16, AtomicI32, AtomicU64, Ordering};

pub static MOVEMENT_MULTIPLIER: AtomicI16 = AtomicI16::new(1400);
pub static INTERVAL_MICROS: AtomicU64 = AtomicU64::new(2000);

// Indicate whether a particular controller function is currently active
pub static ENABLE_MOUSE: AtomicBool = AtomicBool::new(true);
pub static LBUTTONDOWN: AtomicBool = AtomicBool::new(false);
pub static RBUTTONDOWN: AtomicBool = AtomicBool::new(false);
pub static START: AtomicBool = AtomicBool::new(false);
pub static DPADUP: AtomicBool = AtomicBool::new(false);
pub static DPADDOWN: AtomicBool = AtomicBool::new(false);
pub static DPADRIGHT: AtomicBool = AtomicBool::new(false);
pub static DPADLEFT: AtomicBool = AtomicBool::new(false);
pub static LSTICKUP: AtomicBool = AtomicBool::new(false);
pub static LSTICKDOWN: AtomicBool = AtomicBool::new(false);
pub static LSTICKRIGHT: AtomicBool = AtomicBool::new(false);
pub static LSTICKLEFT: AtomicBool = AtomicBool::new(false);
pub static BUTTONA: AtomicBool = AtomicBool::new(false);
pub static BUTTONB: AtomicBool = AtomicBool::new(false);
pub static BUTTONX: AtomicBool = AtomicBool::new(false);
pub static BUTTONY: AtomicBool = AtomicBool::new(false);
pub static SHOULDER_L: AtomicBool = AtomicBool::new(false);
pub static SHOULDER_R: AtomicBool = AtomicBool::new(false);
pub static X: AtomicI32 = AtomicI32::new(0);
pub static Y: AtomicI32 = AtomicI32::new(0);

// Codes assigned to particular controller functions
pub static CODE_DPAD_L: AtomicI32 = AtomicI32::new(0);
pub static CODE_DPAD_R: AtomicI32 = AtomicI32::new(0);
pub static CODE_DPAD_U: AtomicI32 = AtomicI32::new(0);
pub static CODE_DPAD_D: AtomicI32 = AtomicI32::new(0);
pub static CODE_LSTICK_L: AtomicI32 = AtomicI32::new(0);
pub static CODE_LSTICK_R: AtomicI32 = AtomicI32::new(0);
pub static CODE_LSTICK_U: AtomicI32 = AtomicI32::new(0);
pub static CODE_LSTICK_D: AtomicI32 = AtomicI32::new(0);
pub static CODE_BUTTON_A: AtomicI32 = AtomicI32::new(0);
pub static CODE_BUTTON_B: AtomicI32 = AtomicI32::new(0);
pub static CODE_BUTTON_X: AtomicI32 = AtomicI32::new(0);
pub static CODE_BUTTON_Y: AtomicI32 = AtomicI32::new(0);
pub static CODE_BUTTON_START: AtomicI32 = AtomicI32::new(0);
pub static CODE_SHOULDER_L: AtomicI32 = AtomicI32::new(0);
pub static CODE_SHOULDER_R: AtomicI32 = AtomicI32::new(0);

pub fn apply_button_map(map: &common::ButtonMapping) {
    for (control, vcode) in [
        (&CODE_DPAD_L, map.dpadl),
        (&CODE_DPAD_U, map.dpadu),
        (&CODE_DPAD_R, map.dpadr),
        (&CODE_DPAD_D, map.dpadd),
        (&CODE_LSTICK_L, map.lstickl),
        (&CODE_LSTICK_U, map.lsticku),
        (&CODE_LSTICK_R, map.lstickr),
        (&CODE_LSTICK_D, map.lstickd),
        (&CODE_BUTTON_A, map.buttona),
        (&CODE_BUTTON_B, map.buttonb),
        (&CODE_BUTTON_X, map.buttonx),
        (&CODE_BUTTON_Y, map.buttony),
        (&CODE_BUTTON_START, map.start),
        (&CODE_SHOULDER_L, map.shoulderl),
        (&CODE_SHOULDER_R, map.shoulderr),
    ] {
        control.store(vcode, std::sync::atomic::Ordering::Relaxed);
    }

    INTERVAL_MICROS.store(map.sampling_interval, std::sync::atomic::Ordering::Relaxed);
    MOVEMENT_MULTIPLIER.store(
        map.movement_multiplier,
        std::sync::atomic::Ordering::Relaxed,
    );
}

pub fn create_button_map() -> common::ButtonMapping {
    common::ButtonMapping {
        dpadl: CODE_DPAD_L.load(Ordering::Relaxed),
        dpadu: CODE_DPAD_U.load(Ordering::Relaxed),
        dpadr: CODE_DPAD_R.load(Ordering::Relaxed),
        dpadd: CODE_DPAD_D.load(Ordering::Relaxed),
        lstickl: CODE_LSTICK_L.load(Ordering::Relaxed),
        lsticku: CODE_LSTICK_U.load(Ordering::Relaxed),
        lstickr: CODE_LSTICK_R.load(Ordering::Relaxed),
        lstickd: CODE_LSTICK_D.load(Ordering::Relaxed),
        buttona: CODE_BUTTON_A.load(Ordering::Relaxed),
        buttonb: CODE_BUTTON_B.load(Ordering::Relaxed),
        buttonx: CODE_BUTTON_X.load(Ordering::Relaxed),
        buttony: CODE_BUTTON_Y.load(Ordering::Relaxed),
        start: CODE_BUTTON_START.load(Ordering::Relaxed),
        shoulderl: CODE_SHOULDER_L.load(Ordering::Relaxed),
        shoulderr: CODE_SHOULDER_R.load(Ordering::Relaxed),
        movement_multiplier: MOVEMENT_MULTIPLIER.load(Ordering::Relaxed),
        sampling_interval: INTERVAL_MICROS.load(Ordering::Relaxed),
    }
}
