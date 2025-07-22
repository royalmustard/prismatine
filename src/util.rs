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