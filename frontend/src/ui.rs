use common::vkey::*;
use eframe::egui;
use egui_extras::{Size, TableBuilder};
use windows::{
    s,
    Win32::{
        Foundation::{LPARAM, WPARAM},
        System::DataExchange::COPYDATASTRUCT,
        UI::WindowsAndMessaging::{FindWindowA, SendMessageA, WM_COPYDATA},
    },
};

use crate::exit_with_error;

pub struct SerfApp {
    pub active_game_index: usize,
    pub configuration: common::Configuration,
    pub previous: common::ButtonMapping,
    pub rx: crossbeam::channel::Receiver<common::ButtonMapping>,
}

fn selection_dropdown(label: &str, variable: &mut i32, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add_sized([100., 20.], egui::Label::new(label));
        egui::ComboBox::from_id_source(label)
            .selected_text(format!("{}", label_for_code(variable)))
            .show_ui(ui, |ui| {
                for (l, v) in KEYS {
                    ui.selectable_value(variable, *v, *l);
                }
            });
    });
}

fn game_selection_dropdown(
    label: &str,
    active_game_index: &mut usize,
    games: Vec<common::Game>,
    ui: &mut egui::Ui,
) {
    ui.horizontal(|ui| {
        ui.add_sized([100., 20.], egui::Label::new(label));
        egui::ComboBox::from_id_source(label)
            .width(225.)
            .selected_text(format!("{}", games[*active_game_index].name))
            .show_ui(ui, |ui| {
                for (ix, g) in games.iter().enumerate() {
                    ui.selectable_value(active_game_index, ix, g.name.clone());
                }
            });
    });
}

impl eframe::App for SerfApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        _ = crate::CONTEXT.set(ctx.clone());
        let dark = egui::Visuals::dark();
        ctx.set_visuals(egui::Visuals { ..dark });

        let games = self.configuration.games.clone();
        let active_game = &mut self.configuration.games[self.active_game_index];

        if let Ok(new_button_map) = self.rx.try_recv() {
            active_game.controls = new_button_map;
        }

        // On each update, send out the updated configuration to the controller backend.
        if self.previous != active_game.controls {
            unsafe {
                let hwui = FindWindowA(s!("serf-message-window"), s!("serf-controller"));
                if hwui.0 == 0 {
                    exit_with_error(anyhow::anyhow!(
                        "Could not find message sink for back end controller"
                    ));
                }
                let mut data = active_game.controls.clone();
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
                    exit_with_error(anyhow::anyhow!(
                        "Failed dispatch message to sink for back end controller"
                    ));
                }
            }
            self.previous = active_game.controls.clone();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                game_selection_dropdown(
                    "Active configuration",
                    &mut self.active_game_index,
                    games,
                    ui,
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_sized([40., 20.], egui::Button::new("\u{2795}"))
                        .on_hover_text("New");
                });
            });
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_sized([40., 20.], egui::Button::new("\u{1f4be}"))
                        .on_hover_text("Save");
                    ui.add_sized([40., 20.], egui::Button::new("\u{2397}"))
                        .on_hover_text("Revert");
                });
            });
            ui.separator();
            ui.push_id("Shoulders", |ui| {
                TableBuilder::new(ui)
                    .column(Size::exact(220.))
                    .column(Size::exact(220.))
                    .body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown(
                                    "Left shoulder",
                                    &mut active_game.controls.shoulderl,
                                    ui,
                                );
                            });
                            row.col(|ui| {
                                selection_dropdown(
                                    "Right shoulder",
                                    &mut active_game.controls.shoulderr,
                                    ui,
                                );
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown(
                                    "Left thumb",
                                    &mut active_game.controls.lthumb,
                                    ui,
                                );
                            });
                            row.col(|ui| {
                                selection_dropdown(
                                    "Right thumb",
                                    &mut active_game.controls.rthumb,
                                    ui,
                                );
                            });
                        });
                    });
            });
            ui.separator();

            ui.push_id("Buttons", |ui| {
                TableBuilder::new(ui)
                    .column(Size::exact(220.))
                    .column(Size::exact(220.))
                    .body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("A", &mut active_game.controls.buttona, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("B", &mut active_game.controls.buttonb, ui);
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("X", &mut active_game.controls.buttonx, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("Y", &mut active_game.controls.buttony, ui);
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("Start", &mut active_game.controls.start, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("Back", &mut active_game.controls.back, ui);
                            });
                        });
                    });
            });
            ui.separator();

            ui.push_id("DPad", |ui| {
                TableBuilder::new(ui)
                    .column(Size::exact(220.))
                    .column(Size::exact(220.))
                    .body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("DPad Up", &mut active_game.controls.dpadu, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown(
                                    "DPad Down",
                                    &mut active_game.controls.dpadd,
                                    ui,
                                );
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown(
                                    "DPad Left",
                                    &mut active_game.controls.dpadl,
                                    ui,
                                );
                            });
                            row.col(|ui| {
                                selection_dropdown(
                                    "DPad Right",
                                    &mut active_game.controls.dpadr,
                                    ui,
                                );
                            });
                        });
                    });
            });
            ui.separator();

            ui.push_id("LStick", |ui| {
                TableBuilder::new(ui)
                    .column(Size::exact(220.))
                    .column(Size::exact(220.))
                    .body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown(
                                    "LStick Up",
                                    &mut active_game.controls.lsticku,
                                    ui,
                                );
                            });
                            row.col(|ui| {
                                selection_dropdown(
                                    "LStick Down",
                                    &mut active_game.controls.lstickd,
                                    ui,
                                );
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown(
                                    "LStick Left",
                                    &mut active_game.controls.lstickl,
                                    ui,
                                );
                            });
                            row.col(|ui| {
                                selection_dropdown(
                                    "LStick Right",
                                    &mut active_game.controls.lstickr,
                                    ui,
                                );
                            });
                        });
                    });
            });
            ui.separator();

            ui.push_id("Sliders", |ui| {
                TableBuilder::new(ui)
                    .column(Size::exact(100.))
                    .column(Size::exact(340.))
                    .body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::TOP),
                                    |ui| {
                                        ui.label("Movement rate");
                                    },
                                );
                            });
                            row.col(|ui| {
                                ui.style_mut().spacing.slider_width = 288.;
                                ui.add(
                                    egui::Slider::new(
                                        &mut active_game.controls.movement_multiplier,
                                        0..=8000,
                                    )
                                    .step_by(100.)
                                    .integer(),
                                );
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::TOP),
                                    |ui| {
                                        ui.label("Interval \u{3bc}sec");
                                    },
                                );
                            });
                            row.col(|ui| {
                                ui.style_mut().spacing.slider_width = 288.;
                                ui.add(
                                    egui::Slider::new(
                                        &mut active_game.controls.sampling_interval,
                                        0..=8000,
                                    )
                                    .step_by(100.)
                                    .integer(),
                                );
                            });
                        });
                    });
            });
            ui.separator();
        });
    }
}
