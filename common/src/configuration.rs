use crate::vkey::code_for_label;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Configuration {
    pub games: Vec<Game>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Game {
    pub name: String,
    pub controls: crate::ButtonMapping,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            name: "Unnamed mapping".into(),
            controls: crate::ButtonMapping::default(),
        }
    }
}

impl Configuration {
    pub fn save(&self) -> Result<()> {
        // Try to write to the matching location we read from (or initially created).
        // Try finding a configuration.json in the executable dir
        let mut edir = std::env::current_exe()?;
        edir.pop();
        edir.push("configuration.json");
        if edir.exists() {
            std::fs::write(edir, serde_json::to_string_pretty(self)?)?;
        }

        // Otherwise try to current working directory
        let mut cdir = std::env::current_dir()?;
        cdir.push("configuration.json");
        if cdir.exists() {
            std::fs::write(cdir, serde_json::to_string_pretty(self)?)?;
        }

        Ok(())
    }

    pub fn load() -> Result<Self> {
        // Try finding a configuration.json in the executable dir
        let mut edir = std::env::current_exe()?;
        edir.pop();
        edir.push("configuration.json");
        if edir.exists() {
            return Ok(serde_json::from_slice::<Configuration>(&std::fs::read(
                edir,
            )?)?);
        }

        // Try finding a configuration.json in the current directory
        let mut cdir = std::env::current_dir()?;
        cdir.push("configuration.json");
        if cdir.exists() {
            return Ok(serde_json::from_slice::<Configuration>(&std::fs::read(
                cdir,
            )?)?);
        }

        // Try to generate a brand new configuration in the executable dir.
        //let cfg = Configuration::default();

        let games = vec![Game {
            name: "CoD Mediocre Warfare".into(),
            controls: {
                crate::ButtonMapping {
                    dpadl: code_for_label("Left Arrow"),
                    dpadr: code_for_label("Right Arrow"),
                    dpadu: code_for_label("Up Arrow"),
                    dpadd: code_for_label("Down Arrow"),
                    lsticku: code_for_label("W"),
                    lstickd: code_for_label("S"),
                    lstickr: code_for_label("A"),
                    lstickl: code_for_label("D"),
                    buttona: code_for_label("Spacebar"),
                    buttonb: code_for_label("Left Control"),
                    buttonx: code_for_label("F"),
                    buttony: code_for_label("1"),
                    start: code_for_label("Escape"),
                    shoulderl: code_for_label("Q"),
                    shoulderr: code_for_label("E"),
                    lthumb: code_for_label("Shift"),
                    rthumb: code_for_label("V"),
                    back: code_for_label("Tab"),
                    left_autofire: false,
                    right_autofire: false,
                    movement_multiplier: 2000,
                    sampling_interval: 2000,
                    recoil_compensation_active: false,
                    recoil_sideways_compensation: 0,
                    recoil_vertical_compensation: 0,
                    recoil_impulse_vertical: 0,
                    recoil_impulse_duration: 0,
                }
            },
        }];
        let cfg = Configuration { games };
        std::fs::write(edir, serde_json::to_string_pretty(&cfg)?)?;

        Ok(cfg)
    }
}
