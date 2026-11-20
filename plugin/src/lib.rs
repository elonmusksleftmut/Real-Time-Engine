//! HOSTA: Higher-Order Spectral & Time-Arrow analyzer (VST3 / CLAP).
//!
//! Audio passes through untouched (zero latency). A ~8 kHz decimated copy is
//! analysed for 3rd/4th-order phase coupling and transient time-direction. The
//! results drive the editor and are sent out as MIDI CCs, so a DAW can map
//! them to any parameter ("phase-driven automation").

use hosta_core::spectral::EngineOptions;
use hosta_core::{Analyzer, Controls};
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use shared::{Shared, N_OUTPUTS};
use std::sync::atomic::Ordering;
use std::sync::Arc;

mod editor;
mod shared;

/// Mono fold scratch size; host blocks are processed in slices of this length.
const SCRATCH: usize = 512;

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    #[name = "Mid (L+R)"]
    Mid,
    #[name = "Left"]
    Left,
    #[name = "Right"]
    Right,
    #[name = "Side (L-R)"]
    Side,
}

#[derive(Params)]
pub struct HostaParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "tau"]
    pub tau: FloatParam,
    #[id = "gate"]
    pub gate: FloatParam,
    #[id = "floor"]
    pub floor: FloatParam,
    #[id = "input"]
    pub input: EnumParam<InputMode>,

    #[id = "midi_on"]
    pub midi_on: BoolParam,
    #[id = "midi_ch"]
    pub midi_channel: IntParam,
    #[id = "cc_base"]
    pub cc_base: IntParam,
    #[id = "out_smooth"]
    pub out_smooth: FloatParam,
}

impl Default for HostaParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(editor::WIDTH, editor::HEIGHT),
            tau: FloatParam::new("EMA Time", 0.6, FloatRange::Linear { min: 0.2, max: 2.5 })
                .with_step_size(0.05)
                .with_unit(" s")
                .with_value_to_string(formatters::v2s_f32_rounded(2)),
            gate: FloatParam::new(
                "Silence Gate",
                -55.0,
                FloatRange::Linear {
                    min: -70.0,
                    max: -30.0,
                },
            )
            .with_step_size(1.0)
            .with_unit(" dBFS")
            .with_value_to_string(formatters::v2s_f32_rounded(0)),
            floor: FloatParam::new(
                "Coupling Floor",
                -70.0,
                FloatRange::Linear {
                    min: -90.0,
                    max: -40.0,
                },
            )
            .with_step_size(1.0)
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(0)),
            input: EnumParam::new("Input", InputMode::Mid),
            midi_on: BoolParam::new("MIDI CC Out", true),
            midi_channel: IntParam::new("MIDI Channel", 1, IntRange::Linear { min: 1, max: 16 }),
            // CC 20-31 are undefined in the MIDI spec, so the default 20..26 collides with nothing.
            cc_base: IntParam::new(
                "First CC",
                20,
                IntRange::Linear {
                    min: 0,
                    max: (127 - N_OUTPUTS + 1) as i32,
                },
            ),
            out_smooth: FloatParam::new(
                "Output Smoothing",
                80.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 1000.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_step_size(1.0)
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(0)),
        }
    }
}

pub struct Hosta {
    params: Arc<HostaParams>,
    shared: Arc<Shared>,
    analyzer: Analyzer,
    scratch: [f32; SCRATCH],
    /// Smoothed 0..1 output values.
    smoothed: [f32; N_OUTPUTS],
    /// Last 7-bit values sent per output (None forces a resend).
    sent: [Option<u8>; N_OUTPUTS],
    last_channel: u8,
    last_cc_base: u8,
    #[cfg(feature = "demo")]
    demo: Option<(Vec<f32>, usize)>,
}

impl Default for Hosta {
    fn default() -> Self {
        let analyzer = Analyzer::new(48000.0, 0.6, -55.0);
        let n_triads = analyzer.engine.layout.n_triads();
        Self {
            params: Arc::new(HostaParams::default()),
            shared: Arc::new(Shared::new(n_triads)),
            analyzer,
            scratch: [0.0; SCRATCH],
            smoothed: [0.0; N_OUTPUTS],
            sent: [None; N_OUTPUTS],
            last_channel: 0,
            last_cc_base: 20,
            #[cfg(feature = "demo")]
            demo: None,
        }
    }
}

impl Hosta {
    fn sync_params(&mut self) {
        let p = &self.params;
        self.analyzer
            .set_params(p.tau.value() as f64, p.gate.value() as f64);
        self.analyzer.set_options(EngineOptions {
            legacy: false,
            floor_db: p.floor.value() as f64,
        });
    }

    fn clear_outputs(&mut self) {
        self.smoothed = [0.0; N_OUTPUTS];
        self.smoothed[2] = 0.5; // neutral time arrow
        self.sent = [None; N_OUTPUTS];
    }
}

impl Plugin for Hosta {
    const NAME: &'static str = "HOSTA";
    const VENDOR: &'static str = "HOSTA";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.shared.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let fs = buffer_config.sample_rate as f64;
        let p = &self.params;
        self.analyzer = Analyzer::with_options(
            fs,
            p.tau.value() as f64,
            p.gate.value() as f64,
            EngineOptions {
                legacy: false,
                floor_db: p.floor.value() as f64,
            },
        );
        self.shared
            .sample_rate
            .store(buffer_config.sample_rate.to_bits(), Ordering::Relaxed);
        self.clear_outputs();
        #[cfg(feature = "demo")]
        {
            self.demo = std::env::var("HOSTA_DEMO_SCENE")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .and_then(|i| hosta_core::synth::Scene::ALL.get(i).copied())
                .map(|scene| (hosta_core::synth::render_scene(scene, fs), 0));
        }
        true
    }

    fn reset(&mut self) {
        self.analyzer.reset();
        self.clear_outputs();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.sync_params();
        if self.shared.reset_request.swap(false, Ordering::AcqRel) {
            self.analyzer.reset();
            self.clear_outputs();
        }

        let midi_on = self.params.midi_on.value();
        let channel = (self.params.midi_channel.value() - 1).clamp(0, 15) as u8;
        let cc_base = self
            .params
            .cc_base
            .value()
            .clamp(0, (128 - N_OUTPUTS) as i32) as u8;
        if channel != self.last_channel || cc_base != self.last_cc_base {
            self.sent = [None; N_OUTPUTS];
            self.last_channel = channel;
            self.last_cc_base = cc_base;
        }
        let smooth_ms = self.params.out_smooth.value() as f64;
        let hop_sec = hosta_core::spectral::HOP as f64 / self.analyzer.fs_a();
        // One-pole coefficient per analysis hop.
        let keep = if smooth_ms <= 0.0 {
            0.0
        } else {
            (-hop_sec / (smooth_ms * 1e-3)).exp() as f32
        };
        let mode = self.params.input.value();

        let n = buffer.samples();
        let decim = self.analyzer.decimator.factor;
        let chans = buffer.as_slice_immutable();
        let stereo = chans.len() >= 2;
        let mut off = 0;
        while off < n {
            let len = (n - off).min(SCRATCH);
            for i in 0..len {
                let l = chans[0][off + i];
                let r = if stereo { chans[1][off + i] } else { l };
                self.scratch[i] = match mode {
                    InputMode::Mid => 0.5 * (l + r),
                    InputMode::Left => l,
                    InputMode::Right => r,
                    InputMode::Side => 0.5 * (l - r),
                };
            }
            #[cfg(feature = "demo")]
            if let Some((demo, pos)) = &mut self.demo {
                for v in &mut self.scratch[..len] {
                    *v = demo[*pos];
                    *pos = (*pos + 1) % demo.len();
                }
            }

            let smoothed = &mut self.smoothed;
            let sent = &mut self.sent;
            let shared = &self.shared;
            self.analyzer
                .process_engine(&self.scratch[..len], |i, engine| {
                    let r = &engine.latest;
                    let c = Controls::from_readout(r).as_array();
                    let mut target = [0.0f32; N_OUTPUTS];
                    for k in 0..Controls::COUNT {
                        target[k] = c[k] as f32;
                    }
                    for k in 0..Controls::COUNT {
                        smoothed[k] = keep * smoothed[k] + (1.0 - keep) * target[k];
                    }
                    // Verdict is a discrete code: never smoothed. Value = code × 16 (0..112).
                    smoothed[Controls::COUNT] = (r.verdict.code() as f32 * 16.0) / 127.0;

                    if midi_on {
                        for k in 0..N_OUTPUTS {
                            let v7 = (smoothed[k].clamp(0.0, 1.0) * 127.0).round() as u8;
                            if sent[k] != Some(v7) {
                                sent[k] = Some(v7);
                                context.send_event(NoteEvent::MidiCC {
                                    timing: (off + i) as u32,
                                    channel,
                                    cc: cc_base + k as u8,
                                    value: v7 as f32 / 127.0,
                                });
                            }
                        }
                    }
                    if let Some(mut snap) = shared.snap.try_lock() {
                        snap.fill(engine, decim, smoothed);
                    }
                });
            off += len;
        }

        // Pass-through: the analyzer never touches the audio.
        ProcessStatus::Normal
    }
}

impl ClapPlugin for Hosta {
    const CLAP_ID: &'static str = "audio.hosta.analyzer";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Higher-order spectral (bi/tricoherence) and transient time-arrow analyzer with MIDI CC outputs");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Analyzer,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for Hosta {
    const VST3_CLASS_ID: [u8; 16] = *b"HOSTAanalyzer001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Analyzer,
        Vst3SubCategory::Tools,
    ];
}

nih_export_clap!(Hosta);
nih_export_vst3!(Hosta);
