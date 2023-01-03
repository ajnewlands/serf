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
            if RIGHT_AUTOFIRE.load(Ordering::Relaxed) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Cant get systemtime")
                    .as_millis() as u64;
                let delta = now - RIGHT_DOWN_INSTANT.load(Ordering::Relaxed);
                // 37 ms on, 37 off gives circa 800 RPM.
                if delta % 74 < 37 {
                    gamepad.left_trigger = 255;
                } else {
                    gamepad.left_trigger = 0;
                }
            } else {
                gamepad.left_trigger = 255;
            }
        } else {
            gamepad.left_trigger = 0;
        }
        if LBUTTONDOWN.load(Ordering::Relaxed) {
            // Recoil compensation adjusts the gamepad stick position by a given percentage
            if RECOIL_COMPENSATION_ACTIVE.load(Ordering::Relaxed) {
                gamepad.thumb_rx = gamepad.thumb_rx.saturating_add(
                    i16::MAX / 100 * RECOIL_COMPENSATION_SIDEWAYS.load(Ordering::Relaxed) as i16,
                );
                gamepad.thumb_ry = gamepad.thumb_ry.saturating_sub(
                    i16::MAX / 100 * RECOIL_COMPENSATION_VERTICAL.load(Ordering::Relaxed) as i16,
                );
            }

            // Autofire
            if LEFT_AUTOFIRE.load(Ordering::Relaxed) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Cant get systemtime")
                    .as_millis() as u64;
                let delta = now - LEFT_DOWN_INSTANT.load(Ordering::Relaxed);
                // 37 ms on, 37 off gives circa 800 RPM.
                if delta % 74 < 37 {
                    gamepad.right_trigger = 255;
                } else {
                    gamepad.right_trigger = 0;
                }
            } else {
                gamepad.right_trigger = 255;
            }
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
