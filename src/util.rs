

use std::fmt::Display;

use nih_plug::prelude::Enum;

pub fn map_range_linear(in_value: f32, in_start: f32, in_stop: f32, out_start: f32, out_stop: f32) -> f32
{
    let in_range = in_stop - in_start;
    let in_progress = (in_value - in_start) / in_range;
    let out_range = out_stop - out_start;
    let mut out = out_start + in_progress * out_range;
    if out.is_nan() || !out.is_finite()
    {
        out = out_stop;
    }
    out
}

#[derive(Enum, PartialEq, Clone, Copy, Debug)]
pub enum ProcessMode{
    Josephson, 
    KO1, 
    KO2, 
    AB
}

impl Display for ProcessMode
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self
        {
            ProcessMode::Josephson => write!(f, "Josephson"),
            ProcessMode::AB => write!(f, "AB"),
            ProcessMode::KO1=> write!(f, "KO-1"),
            ProcessMode::KO2 => write!(f, "KO-2"),
        }
    }
}