use crate::statics::*;
use log::info;
use std::sync::atomic::Ordering;
use vigem_client::*;

pub fn run_controller(mut gamepad: XGamepad, mut target: Xbox360Wired<Client>) {
    info!("Launching serf controller.");

    info!("Virtual gamepad attached.");
    loop {
        std::thread::sleep(std::time::Duration::from_micros(
            INTERVAL_MICROS.load(Ordering::Relaxed),
        ));
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

        let button_map = vec![
            (&START, XButtons::START),
            (&DPADUP, XButtons::UP),
            (&DPADDOWN, XButtons::DOWN),
            (&DPADLEFT, XButtons::LEFT),
            (&DPADRIGHT, XButtons::RIGHT),
            (&BUTTONA, XButtons::A),
            (&BUTTONB, XButtons::B),
            (&BUTTONX, XButtons::X),
            (&BUTTONY, XButtons::Y),
            (&SHOULDER_L, XButtons::LB),
            (&SHOULDER_R, XButtons::RB),
            (&THUMB_L, XButtons::LTHUMB),
            (&THUMB_R, XButtons::RTHUMB),
            (&BACK, XButtons::BACK),
        ];
        for (is_pressed, button) in button_map {
            if is_pressed.load(Ordering::Relaxed) {
                gamepad.buttons.raw = gamepad.buttons.raw | button;
            }
        }

        // Left thumbstick. Why is X backwards?
        if LSTICKUP.load(Ordering::Relaxed) {
            gamepad.thumb_ly = i16::MAX;
        } else if LSTICKDOWN.load(Ordering::Relaxed) {
            gamepad.thumb_ly = i16::MIN;
        } else {
            gamepad.thumb_ly = 0;
        }

        if LSTICKRIGHT.load(Ordering::Relaxed) {
            gamepad.thumb_lx = i16::MIN;
        } else if LSTICKLEFT.load(Ordering::Relaxed) {
            gamepad.thumb_lx = i16::MAX;
        } else {
            gamepad.thumb_lx = 0;
        }

        target
            .update(&gamepad)
            .expect("should be able to update our gamepad");
    }
}
