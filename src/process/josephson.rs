use std::sync::Arc;

use crate::{util, PrismatineParams, MAX_PHASE};

#[inline]
pub fn process_josephson(params: Arc<PrismatineParams>, prev:&mut [f32;2], phase:&mut [f32;2], channel_idx: usize, sample: &mut f32)
{
    let diff = prev[channel_idx] - *sample;
                let dphi = 
                if params.invert_phase.value()
                {
                   util::map_range_linear(1.0/(prev[channel_idx] - *sample), 0.0, 1.0/f32::EPSILON, 0.0, 1.0) * params.phase_gain.smoothed.next()
                }
                else {
                    (prev[channel_idx] - *sample) * params.phase_gain.smoothed.next()
                };
                
                prev[channel_idx] = *sample;
                //prevent NaN poisoning
                if prev[channel_idx].is_nan()
                {
                    prev[channel_idx] = 0.0;
                }
                //limit maximum phase for numerical precision
                if phase[channel_idx] + dphi > MAX_PHASE {
                    phase[channel_idx] += -MAX_PHASE + dphi;
                    
                } else if phase[channel_idx] + dphi < -MAX_PHASE {
                    phase[channel_idx] += MAX_PHASE + dphi;
                   
                } else {
                    phase[channel_idx] += dphi;
                }
                if params.invert_phase.value()
                {
                   *sample = diff * params.I_c.smoothed.next() * phase[channel_idx].sin();
                }
                else {
                    *sample = params.I_c.smoothed.next() * phase[channel_idx].sin();
                }
                if sample.is_nan()
                {
                    *sample = 0.0;
                }
}