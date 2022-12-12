pub mod vkey;
pub use vkey::*;

#[derive(Debug, Clone)]
pub enum Signals {
    CapturingMouse = 0,
    ReleasingMouse = 1,
    Terminating = 2,
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
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
    pub start: i32,
    pub movement_multiplier: i16,
    pub sampling_interval: u64,
}
