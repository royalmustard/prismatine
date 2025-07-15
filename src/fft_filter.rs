
use nih_plug::buffer::{Buffer, SamplesIter};

pub struct FFTHelper
{
    input_buffer: Vec<Vec<Vec<f32>>>, //A/B buffer system
    output_buffer: Vec<Vec<f32>>,
    block_size: usize,
    buffer_idx: usize,
    out_buffer_idx: usize,
    sample_cnt: usize
}


///Only use this when the window size of the FFT is larger than the buffer size provided in process
/// Accumulates samples until window size is reached
impl FFTHelper
{

    pub fn new(channels: usize, block_size: usize) -> Self
    {
        Self { input_buffer: vec![vec![vec![0.0; block_size]; channels];2],
            output_buffer: vec![vec![0.0;block_size]; channels],
            block_size: block_size,
        buffer_idx: 0,
        out_buffer_idx: 0,
        sample_cnt: 0 }
    }
    pub fn reset(&mut self)
    {
        //Clear buffers
        for u in self.input_buffer.iter_mut()
        {
            for v in u.iter_mut()
            {
                for x in v
                {
                    *x = 0.0;
                }
            }
        }
    }
    pub fn process<F>(&mut self, buf: &mut Buffer,mut callback: F)
    where F: FnMut(usize, &mut [f32])
    {
        //Collect samples until block_size is reached, then call callback
        //Callback will write back to buf
        //What do we do when we have only processed a part of buf?
        //We need a local buffer with at least the size of buf
        //We copy into A buffer, on fill we copy the rest into B buffer and then callback, swap buffers after
        let mut samples_processed: usize = 0;
        let samples_to_process: usize =buf.samples();
        //Fill A buffer
        let mut buf_iter = buf.iter_samples();
        let buffer_idx_before_process = self.buffer_idx;
        while samples_processed < samples_to_process
        {
            if self.sample_cnt >= self.block_size
            {
                //Bitwise XOR does not seems possible as usize is machine dependent
                self.buffer_idx ^= 0b1;
                self.sample_cnt = 0;
                for (id, ch_buff) in self.input_buffer[self.buffer_idx^0b1].iter_mut().enumerate()
                {
                    callback(id, ch_buff);
                }
            }
           if let Some(samples_iter) = buf_iter.next()
           {
            for (channel_no, channel_sample) in samples_iter.into_iter().enumerate()
            {
                self.input_buffer[self.buffer_idx][channel_no][self.sample_cnt] = *channel_sample;
                *channel_sample = self.input_buffer[self.buffer_idx^0b1][channel_no][self.out_buffer_idx]
            }
            self.out_buffer_idx+=1;
            self.sample_cnt +=1;
            samples_processed +=1;
            if self.out_buffer_idx >= self.block_size
            {
                self.out_buffer_idx = 0;
            }
           }
        }
        
        //Need to write output

    }
}