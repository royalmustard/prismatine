use core::f32;
use std::sync::Arc;

use crate::PrismatineParams;

#[inline]
pub fn process_ab(
    params: Arc<PrismatineParams>,
    prev: &mut [f32; 2],
    phase: &mut [f32; 2],
    channel_idx: usize,
    sample: &mut f32,
) {
    let dphi = *sample * params.ab_params.phase_gain.smoothed.next();
    prev[channel_idx] = *sample;
    //prevent NaN poisoning
    if prev[channel_idx].is_nan() {
        prev[channel_idx] = 0.0;
    }
    phase[channel_idx] += dphi;
    phase[channel_idx] = phase[channel_idx].rem_euclid(2.0 * f32::consts::PI);
    *sample = params.ab_params.critical_current.smoothed.next() * phase[channel_idx].sin();
    if sample.is_nan() {
        *sample = 0.0;
    }
}
