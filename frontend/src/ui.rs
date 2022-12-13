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
    pub map: common::ButtonMapping,
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

impl eframe::App for SerfApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        _ = crate::CONTEXT.set(ctx.clone());
        let dark = egui::Visuals::dark();
        ctx.set_visuals(egui::Visuals { ..dark });

        if let Ok(new_button_map) = self.rx.try_recv() {
            self.map = new_button_map;
        }

        // On each update, send out the updated configuration to the controller backend.
        if self.previous != self.map {
            unsafe {
                let hwui = FindWindowA(s!("serf-message-window"), s!("serf-controller"));
                if hwui.0 == 0 {
                    exit_with_error(anyhow::anyhow!(
                        "Could not find message sink for back end controller"
                    ));
                }
                let mut data = self.map.clone();
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
            self.previous = self.map.clone();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.push_id("Shoulders", |ui| {
                TableBuilder::new(ui)
                    .column(Size::exact(220.))
                    .column(Size::exact(220.))
                    .body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("Left shoulder", &mut self.map.shoulderl, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("Right shoulder", &mut self.map.shoulderr, ui);
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("Left thumb", &mut self.map.lthumb, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("Right thumb", &mut self.map.rthumb, ui);
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
                                selection_dropdown("A", &mut self.map.buttona, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("B", &mut self.map.buttonb, ui);
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("X", &mut self.map.buttonx, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("Y", &mut self.map.buttony, ui);
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("Start", &mut self.map.start, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("Back", &mut self.map.back, ui);
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
                                selection_dropdown("DPad Up", &mut self.map.dpadu, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("DPad Down", &mut self.map.dpadd, ui);
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("DPad Left", &mut self.map.dpadl, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("DPad Right", &mut self.map.dpadr, ui);
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
                                selection_dropdown("LStick Up", &mut self.map.lsticku, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("LStick Down", &mut self.map.lstickd, ui);
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("LStick Left", &mut self.map.lstickl, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("LStick Right", &mut self.map.lstickr, ui);
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
                                    egui::Slider::new(&mut self.map.movement_multiplier, 0..=8000)
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
                                    egui::Slider::new(&mut self.map.sampling_interval, 0..=8000)
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
