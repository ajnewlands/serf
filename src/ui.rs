use crate::vkey::*;
use eframe::egui;
use egui_extras::{Size, TableBuilder};

pub struct SerfApp {
    selected: Option<i32>,
    dpadl: Option<i32>,
    dpadr: Option<i32>,
    dpadu: Option<i32>,
    dpadd: Option<i32>,
    lsticku: Option<i32>,
    lstickd: Option<i32>,
    lstickr: Option<i32>,
    lstickl: Option<i32>,
    movement_multiplier: i16,
    sampling_interval: u64,
}

impl Default for SerfApp {
    fn default() -> Self {
        Self {
            selected: None,
            dpadl: code_for_label("Left Arrow"),
            dpadr: code_for_label("Right Arrow"),
            dpadu: code_for_label("Up Arrow"),
            dpadd: code_for_label("Down Arrow"),
            lsticku: code_for_label("W"),
            lstickd: code_for_label("S"),
            lstickr: code_for_label("A"),
            lstickl: code_for_label("D"),
            movement_multiplier: 1400,
            sampling_interval: 2000,
        }
    }
}

fn selection_dropdown(label: &str, variable: &mut Option<i32>, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add_sized([100., 20.], egui::Label::new(label));
        egui::ComboBox::from_id_source(label)
            .selected_text(format!("{}", crate::vkey::label_for_code(variable)))
            .show_ui(ui, |ui| {
                for (l, v) in crate::vkey::KEYS {
                    ui.selectable_value(variable, *v, *l);
                }
            });
    });
}

impl eframe::App for SerfApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dark = egui::Visuals::dark();
        ctx.set_visuals(egui::Visuals { ..dark });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.push_id("Shoulders", |ui| {
                TableBuilder::new(ui)
                    .column(Size::exact(220.))
                    .column(Size::exact(220.))
                    .body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("Left shoulder", &mut self.selected, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("Right shoulder", &mut self.selected, ui);
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
                                selection_dropdown("A", &mut self.selected, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("B", &mut self.selected, ui);
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("X", &mut self.selected, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("Y", &mut self.selected, ui);
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
                                selection_dropdown("DPad Up", &mut self.dpadu, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("DPad Down", &mut self.dpadd, ui);
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("DPad Left", &mut self.dpadl, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("DPad Right", &mut self.dpadr, ui);
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
                                selection_dropdown("LStick Up", &mut self.lsticku, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("LStick Down", &mut self.lstickd, ui);
                            });
                        });
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                selection_dropdown("LStick Left", &mut self.lstickl, ui);
                            });
                            row.col(|ui| {
                                selection_dropdown("LStick Right", &mut self.lstickr, ui);
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
                                    egui::Slider::new(&mut self.movement_multiplier, 0..=8000)
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
                                    egui::Slider::new(&mut self.sampling_interval, 0..=8000)
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
