use core::f32;
use std::{sync::Arc};

use crate::PrismatineParams;



#[inline]
pub fn process_ko2(params: Arc<PrismatineParams>, prev:&mut [f32;2], phase:&mut [f32;2], channel_idx: usize, sample: &mut f32)
{
    let dphi = *sample * params.ko2_params.phase_gain.smoothed.next();
    prev[channel_idx] = *sample;
    //prevent NaN poisoning
    if prev[channel_idx].is_nan() {
        prev[channel_idx] = 0.0;
    }
    phase[channel_idx] += dphi;
    phase[channel_idx] = phase[channel_idx].rem_euclid(2.0 * std::f32::consts::PI);

    *sample = params.ko2_params.critical_current.smoothed.next() * 
                    (phase[channel_idx] / 2.0).sin() * ((phase[channel_idx] / 2.0).cos() / (2.0 * params.ko2_params.temperature.value()) ).tanh()
}