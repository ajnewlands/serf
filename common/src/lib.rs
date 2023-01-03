pub mod vkey;
pub use vkey::*;

pub mod configuration;
pub use configuration::*;

use serde::{Deserialize, Serialize};

#[repr(usize)]
pub enum CopyTypes {
    ButtonMap = 0,
    CaptureMouse = 1,
    ReleaseMouse = 2,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ButtonMapping {
    pub dpadl: i32,
    pub dpadr: i32,
    pub dpadu: i32,
    pub dpadd: i32,
    pub lsticku: i32,
    pub lstickd: i32,
    pub lstickr: i32,
    pub lstickl: i32,
    pub buttona: i32,
    pub buttonb: i32,
    pub buttonx: i32,
    pub buttony: i32,
    pub shoulderl: i32,
    pub shoulderr: i32,
    pub lthumb: i32,
    pub rthumb: i32,
    pub start: i32,
    pub back: i32,
    #[serde(default)]
    pub left_autofire: bool,
    #[serde(default)]
    pub right_autofire: bool,
    pub movement_multiplier: i16,
    pub sampling_interval: u64,
    #[serde(default)]
    pub recoil_compensation_active: bool,
    #[serde(default)]
    pub recoil_vertical_compensation: i32,
    #[serde(default)]
    pub recoil_sideways_compensation: i32,
}
