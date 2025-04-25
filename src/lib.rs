use fft_filter::FFTHelper;
use nih_plug::prelude::*;
use realfft::{num_complex::{Complex, Complex32, ComplexFloat}, num_traits::Inv, ComplexToReal, RealFftPlanner, RealToComplex};
use std::sync::Arc;

mod fft_filter;
// FT stuff:
// Sample rate ~ maximum frequency
// Window size ~ minimum frequency
// Number of samples ~ frequency resolution

// The size of the windows we'll process at a time.
const WINDOW_SIZE: usize = 1024;
/// The length of the filter's impulse response.
const FILTER_WINDOW_SIZE: usize = 0;
/// The length of the FFT window we will use to perform FFT convolution. This includes padding to
/// prevent time domain aliasing as a result of cyclic convolution.
const FFT_WINDOW_SIZE: usize = WINDOW_SIZE; //+ FILTER_WINDOW_SIZE - 1;

/// The gain compensation we need to apply for the STFT process.
const GAIN_COMPENSATION: f32 = 1.0 / FFT_WINDOW_SIZE as f32;

fn kinetic_spectrum_from_window_size(window_size: usize, sample_rate: f32) -> Vec<Complex<f32>>
{
    let filter_spectrum: Vec<Complex32> = (0..window_size/2).map(|i| (i as f32)* sample_rate/(2.0 *window_size as f32) ) //construced frequency values
    .map(|f| {if f != 0.0 {Complex32{re:1.0/f, im:1.0/f}}
                    else {Complex32::new(0.0, 0.0)}})
    .collect();
    let gain_compensation: f32 = filter_spectrum.iter().map(|c| c.abs()).sum::<f32>().inv();
    filter_spectrum.iter().map(|c| c*gain_compensation).collect()
}
struct Prismatine {
    params: Arc<PrismatineParams>,

        /// An adapter that performs most of the overlap-add algorithm for us.
        stft: FFTHelper,

        /// The FFT of a simple low-pass FIR filter.
        filter_spectrum: Vec<Complex32>,
    
        /// The algorithm for the FFT operation.
        r2c_plan: Arc<dyn RealToComplex<f32>>,
        /// The algorithm for the IFFT operation.
        c2r_plan: Arc<dyn ComplexToReal<f32>>,
        /// The output of our real->complex FFT.
        complex_fft_buffer: Vec<Complex32>,

        scratch_buffer: [Complex32; 2048],
        window_buff: [f32; FFT_WINDOW_SIZE]
}

#[derive(Params)]
struct PrismatineParams {

}

impl Default for Prismatine {
    fn default() -> Self {
        let mut planner = RealFftPlanner::new();
        let r2c_plan = planner.plan_fft_forward(FFT_WINDOW_SIZE);
        let c2r_plan = planner.plan_fft_inverse(FFT_WINDOW_SIZE);
        let complex_fft_buffer = r2c_plan.make_output_vec();
        nih_dbg!(complex_fft_buffer.len());
        
        
        

        Self {
            params: Arc::new(PrismatineParams::default()),
            stft: FFTHelper::new(2, WINDOW_SIZE),

            filter_spectrum: vec![Complex32{re: 0.0, im: 0.0}; complex_fft_buffer.len()],

            r2c_plan,
            c2r_plan,
            complex_fft_buffer,
            scratch_buffer: [Complex32::new(0.0, 0.0); 2048],
            window_buff: [0.0; FFT_WINDOW_SIZE]
        }
    }
}

impl Default for PrismatineParams {
    fn default() -> Self {
        Self {
        }
    }
}

impl Plugin for Prismatine {
    const NAME: &'static str = "Prismatine";
    const VENDOR: &'static str = "royalmustard";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "royalmustard@memium.de";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];


    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        context.set_latency_samples(WINDOW_SIZE as u32);
        self.filter_spectrum = kinetic_spectrum_from_window_size(WINDOW_SIZE, buffer_config.sample_rate);
        //nih_dbg!(&self.filter_spectrum);
        util::window::hann_in_place(&mut self.window_buff);
        nih_dbg!(self.filter_spectrum.iter().map(|c| c.abs()).sum::<f32>());
        //nih_dbg!(self.scratch_buffer);
        true
    }

    fn reset(&mut self) {
        //self.stft.set_block_size(WINDOW_SIZE);
        self.stft.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        nih_dbg!(buffer.samples());
        self.stft
            .process(buffer, |_channel_idx, real_fft_buffer| {
                
                

                //util::window::multiply_with_window(real_fft_buffer, &self.window_buff);
                self.r2c_plan
                    .process_with_scratch(real_fft_buffer, &mut self.complex_fft_buffer, &mut self.scratch_buffer)
                    .unwrap();
                
                nih_dbg!(self.filter_spectrum.len());
                nih_dbg!(self.complex_fft_buffer.len());
                for (fft_bin, kernel_bin) in self
                    .complex_fft_buffer
                    .iter_mut()
                    .zip(&self.filter_spectrum)
                {
                    *fft_bin *=  GAIN_COMPENSATION* kernel_bin;
                }
                
                // Inverse FFT back into the scratch buffer. This will be added to a ring buffer
                // which gets written back to the host at a one block delay.
                self.c2r_plan
                    .process_with_scratch(&mut self.complex_fft_buffer, real_fft_buffer, &mut self.scratch_buffer)
                    .unwrap();
                
            });
        

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Prismatine {
    const CLAP_ID: &'static str = "de.memium.prismatine";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A short description of your plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}



nih_export_clap!(Prismatine);

