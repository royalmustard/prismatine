use core::f32;
use std::{any::Any, sync::Arc};

use crate::PrismatineParams;



#[inline]
pub fn process_ko1(params: Arc<PrismatineParams>, prev:&mut [f32;2], phase:&mut [f32;2], channel_idx: usize, sample: &mut f32)
{
    let dphi = *sample * params.ko1_params.phase_gain.smoothed.next();
    prev[channel_idx] = *sample;
    //prevent NaN poisoning
    if prev[channel_idx].is_nan() {
        prev[channel_idx] = 0.0;
    }
    phase[channel_idx] += dphi;
    phase[channel_idx] = phase[channel_idx].rem_euclid(2.0 * std::f32::consts::PI);

    *sample = 0.0;
    for i in 1..(params.ko1_params.terms.value()+1)
    {
        let matsubara = f32::consts::PI * params.ko1_params.temperature.value() *(2.0*i as f32+1.0);
        let delta = ((phase[channel_idx]/2.0).cos().powi(2) + matsubara.powi(2)).sqrt();

        *sample += ((phase[channel_idx]/2.0).cos() / delta) * ((phase[channel_idx]/2.0).sin() / delta).atan();

    }

    *sample *= params.ko1_params.temperature.value() * params.ko1_params.critical_current.smoothed.next();
}