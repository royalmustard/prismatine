use core::f32;
use fft_filter::FFTHelper;


use nih_plug::prelude::*;
use nih_plug::prelude::util::db_to_gain;
use nih_plug::prelude::util as nih_util;

use nih_plug_iced::IcedState;
use realfft::{
    num_complex::{Complex, Complex32, ComplexFloat},
    num_traits::{Inv},
    ComplexToReal, RealFftPlanner, RealToComplex,
};
use std::sync::Arc;

use crate::editor::PrismatineEditorParams;

mod editor;
mod fft_filter;
mod util;

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

const MAX_PHASE: f32 = 1.0e4 * f32::consts::PI;
fn kinetic_spectrum_from_window_size(window_size: usize, sample_rate: f32) -> Vec<Complex<f32>> {
    let filter_spectrum: Vec<Complex32> = (0..window_size / 2)
        .map(|i| (i as f32) * sample_rate / (2.0 * window_size as f32)) //construced frequency values
        .map(|f| {
            if f != 0.0 {
                Complex32 {
                    re: 1.0 / f,
                    im: 1.0 / f,
                }
            } else {
                Complex32::new(0.0, 0.0)
            }
        })
        .collect();
    let gain_compensation: f32 = filter_spectrum.iter().map(|c| c.abs()).sum::<f32>().inv();
    filter_spectrum
        .iter()
        .map(|c| c * gain_compensation)
        .collect()
}
pub struct Prismatine {
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
    window_buff: [f32; FFT_WINDOW_SIZE],

    prev: [f32; 2],
    phase: Arc<[AtomicF32; 2]>,
}

#[derive(Params)]
struct PrismatineParams {

     #[persist = "editor-state"]
    editor_state: Arc<IcedState>,
    //TODO: Dry/Wet
    #[id = "phase_gain"]
    phase_gain: FloatParam,

    #[id = "I_c"]
    I_c: FloatParam,

    #[id = "invert_phase"]
    invert_phase: BoolParam,

    #[id = "remove_dc"]
    remove_dc: BoolParam,
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

            filter_spectrum: vec![Complex32 { re: 0.0, im: 0.0 }; complex_fft_buffer.len()],

            r2c_plan,
            c2r_plan,
            complex_fft_buffer,
            scratch_buffer: [Complex32::new(0.0, 0.0); 2048],
            window_buff: [0.0; FFT_WINDOW_SIZE],
            prev: [0.0; 2],
            phase: Arc::new([AtomicF32::new(0.0), AtomicF32::new(0.0)]),
        }
    }
}

impl Default for PrismatineParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            phase_gain: FloatParam::new(
                "Phase Gain",
                db_to_gain(0.0),
                FloatRange::Skewed {
                    min: db_to_gain(0.0),
                    max: db_to_gain(60.0),
                    factor: FloatRange::gain_skew_factor(0.0, 60.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            I_c: FloatParam::new(
                "Critical Current",
                db_to_gain(0.0),
                FloatRange::Skewed {
                    min: db_to_gain(-30.0),
                    max: db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            invert_phase: BoolParam::new("Invert Phase", false),
            remove_dc: BoolParam::new("Remove DC", false)
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
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        context.set_latency_samples(WINDOW_SIZE as u32);
        self.filter_spectrum =
            kinetic_spectrum_from_window_size(WINDOW_SIZE, buffer_config.sample_rate);
        //nih_dbg!(&self.filter_spectrum);
        nih_util::window::hann_in_place(&mut self.window_buff);
        nih_dbg!(self.filter_spectrum.iter().map(|c| c.abs()).sum::<f32>());
        //nih_dbg!(self.scratch_buffer);
        //self.prev = vec![0.0, 0.0]; //Two input channels, as specified in the layout, idk if this needs to by dynamic
        true
    }

    fn reset(&mut self) {
        //self.stft.set_block_size(WINDOW_SIZE);
        self.stft.reset();
        for afloat in self.phase.as_ref()
        {
            afloat.store(0.0, std::sync::atomic::Ordering::Release);
        }
        self.prev = [0.0;2];
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        
        //TODO: Play with simd
        for channel_samples in buffer.iter_samples() {
            for (i, sample) in channel_samples.into_iter().enumerate() {
                let diff = self.prev[i] - *sample;
                let dphi = 
                if self.params.invert_phase.value()
                {
                   util::map_range_linear(1.0/(self.prev[i] - *sample), 0.0, 1.0/f32::EPSILON, 0.0, 1.0) * self.params.phase_gain.smoothed.next()
                }
                else {
                    (self.prev[i] - *sample) * self.params.phase_gain.smoothed.next()
                };
                
                self.prev[i] = *sample;
                //prevent NaN poisoning
                if self.prev[i].is_nan()
                {
                    self.prev[i] = 0.0;
                }

                let mut local_phase = self.phase[i].load(std::sync::atomic::Ordering::Acquire);
                //limit maximum phase for numerical precision
                if local_phase + dphi > MAX_PHASE {
                    local_phase += -MAX_PHASE + dphi;
                    
                } else if local_phase + dphi < -MAX_PHASE {
                    local_phase += MAX_PHASE + dphi;
                   
                } else {
                    local_phase += dphi;
                }
                self.phase[i].store(local_phase, std::sync::atomic::Ordering::Release);
                if self.params.invert_phase.value()
                {
                   *sample = diff * self.params.I_c.smoothed.next() * local_phase.sin();
                }
                else {
                    *sample = self.params.I_c.smoothed.next() * local_phase.sin();
                }
                if sample.is_nan()
                {
                    *sample = 0.0;
                }
                //nih_dbg!(&sample);

                
            }
            //TODO: Reset phase when input stops to prevent outputting constant signal
            //reset phase buttons in GUI
            
        }
        // FFT yeet DC component
        if self.params.remove_dc.value()
        {
            self.stft.process(buffer, |_channel_idx, real_fft_buffer| {
            self.r2c_plan
                .process_with_scratch(
                    real_fft_buffer,
                    &mut self.complex_fft_buffer,
                    &mut self.scratch_buffer,
                )
                .unwrap();

            self.complex_fft_buffer[0] = 0.0.into();
            self.complex_fft_buffer.iter_mut().for_each(|c| *c *= GAIN_COMPENSATION);
            self.c2r_plan
                .process_with_scratch(
                    &mut self.complex_fft_buffer,
                    real_fft_buffer,
                    &mut self.scratch_buffer,
                )
                .unwrap();
            });
        }
        

        ProcessStatus::Normal
    }

    fn editor(&mut self, async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            PrismatineEditorParams{
                prismatine_params: self.params.clone(),
                phase: self.phase.clone(),
            },
            self.params.editor_state.clone(),
        )
    }
}

impl ClapPlugin for Prismatine {
    const CLAP_ID: &'static str = "de.royalmustard.prismatine";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A short description of your plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

nih_export_clap!(Prismatine);
