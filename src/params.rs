use nih_plug::{
    formatters, params::{FloatParam, IntParam, Params}, prelude::{FloatRange, SmoothingStyle}, util::db_to_gain
};

fn float_gain(min: f32, max:f32) -> FloatParam {
    FloatParam::new(
        "Phase Gain",
        db_to_gain(0.0),
        FloatRange::Skewed {
            min: db_to_gain(min),
            max: db_to_gain(max),
            factor: FloatRange::gain_skew_factor(0.0, 60.0),
        },
    )
    .with_smoother(SmoothingStyle::Logarithmic(50.0))
    .with_unit(" dB")
    .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
    .with_string_to_value(formatters::s2v_f32_gain_to_db())
}

#[derive(Params)]
pub struct KO1Params {
    #[id = "phase_gain"]
    pub phase_gain: FloatParam,
    #[id = "temperature"]
    pub temperature: FloatParam,
    #[id = "terms"]
    pub terms: IntParam,
}

impl Default for KO1Params {
    fn default() -> Self {
        Self {
            phase_gain: float_gain(0.0, 60.0),
            temperature: FloatParam::new(
                "Temperature",
                0.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 300.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            ),
            terms: IntParam::new(
                "Terms",
                3,
                nih_plug::prelude::IntRange::Linear { min: 1, max: 10 },
            ),
        }
    }
}

#[derive(Params)]
pub struct ABParams {
    #[id = "phase_gain"]
    pub phase_gain: FloatParam,
    #[id = "critical_current"]
    pub critical_current: FloatParam,
    #[id = "temperature"]
    pub temperature: FloatParam,
}
impl Default for ABParams
{
     fn default() -> Self {
        Self {
            phase_gain: float_gain(-30.0, 30.0),
            critical_current: float_gain(-30.0, 30.0),
            temperature: FloatParam::new(
                "Temperature",
                0.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 300.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            ),
        }
    }
}