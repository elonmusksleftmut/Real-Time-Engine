//! egui editor: a native port of the browser prototype's studio layout.

use crate::shared::{Shared, Snapshot, N_OUTPUTS, OUTPUT_NAMES};
use crate::HostaParams;
use hosta_core::spectral::{Layout, BI_H, BI_W, RTA_BINS, SCOPE_LEN, TRI_H, TRI_W};
use hosta_core::Verdict;
use nih_plug::prelude::{Editor, Param, ParamSetter};
use nih_plug_egui::egui::{
    self, pos2, vec2, Align2, Color32, ColorImage, CornerRadius, FontId, Frame, Mesh, Pos2, Rect,
    RichText, Sense, Shape, Stroke, StrokeKind, TextureHandle, TextureOptions, Ui, Vec2,
};
use nih_plug_egui::{create_egui_editor, resizable_window::ResizableWindow, widgets, EguiState};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const WIDTH: u32 = 1200;
pub const HEIGHT: u32 = 900;

// Prototype theme.
const BG0: Color32 = Color32::from_rgb(0x06, 0x09, 0x0f);
const BG1: Color32 = Color32::from_rgb(0x0c, 0x12, 0x1d);
const BG2: Color32 = Color32::from_rgb(0x12, 0x1b, 0x2b);
const BG3: Color32 = Color32::from_rgb(0x19, 0x25, 0x3a);
const BORDER: Color32 = Color32::from_rgb(0x23, 0x32, 0x4d);
const BORDER_HI: Color32 = Color32::from_rgb(0x35, 0x4c, 0x73);
const TEXT0: Color32 = Color32::from_rgb(0xe8, 0xee, 0xf7);
const TEXT1: Color32 = Color32::from_rgb(0x9f, 0xb0, 0xc9);
const TEXT2: Color32 = Color32::from_rgb(0x64, 0x78, 0x96);
const CYAN: Color32 = Color32::from_rgb(0x22, 0xd3, 0xee);
const AMBER: Color32 = Color32::from_rgb(0xf5, 0x9e, 0x0b);
const CRIMSON: Color32 = Color32::from_rgb(0xef, 0x44, 0x44);
const EMERALD: Color32 = Color32::from_rgb(0x10, 0xb9, 0x81);
const VIOLET: Color32 = Color32::from_rgb(0xa8, 0x55, 0xf7);
const SKY: Color32 = Color32::from_rgb(0x38, 0xbd, 0xf8);

struct EditorState {
    layout: Layout,
    snap: Snapshot,
    bi_tex: Option<TextureHandle>,
    tri_tex: Option<TextureHandle>,
    asym: bool,
    rta_ref: Option<Vec<f32>>,
    last_seq: u64,
    last_change: Instant,
}

pub fn create(params: Arc<HostaParams>, shared: Arc<Shared>) -> Option<Box<dyn Editor>> {
    let egui_state: Arc<EguiState> = params.editor_state.clone();
    let layout = Layout::new();
    let n = layout.n_triads();
    let state = EditorState {
        layout,
        snap: Snapshot::new(n),
        bi_tex: None,
        tri_tex: None,
        asym: false,
        rta_ref: None,
        last_seq: 0,
        last_change: Instant::now(),
    };
    create_egui_editor(
        params.editor_state.clone(),
        state,
        |ctx, _| {
            let mut v = egui::Visuals::dark();
            v.panel_fill = BG0;
            v.window_fill = BG1;
            v.extreme_bg_color = BG0;
            v.widgets.inactive.bg_fill = BG2;
            v.widgets.inactive.weak_bg_fill = BG2;
            v.widgets.inactive.bg_stroke = Stroke::new(1.0f32, BORDER);
            v.widgets.hovered.bg_fill = BG3;
            v.widgets.hovered.weak_bg_fill = BG3;
            v.widgets.hovered.bg_stroke = Stroke::new(1.0f32, BORDER_HI);
            v.selection.bg_fill = Color32::from_rgba_unmultiplied(0x22, 0xd3, 0xee, 60);
            v.selection.stroke = Stroke::new(1.0f32, CYAN);
            ctx.set_visuals(v);
        },
        move |ctx, setter, st| {
            {
                let s = shared.snap.lock();
                st.snap.copy_from(&s);
            }
            if st.snap.seq != st.last_seq {
                st.last_seq = st.snap.seq;
                st.last_change = Instant::now();
            }
            ResizableWindow::new("hosta-window")
                .min_size(Vec2::new(980.0, 720.0))
                .show(ctx, egui_state.as_ref(), |ui| {
                    draw(ui, setter, &params, &shared, st);
                });
            ctx.request_repaint();
        },
    )
}

fn draw(
    ui: &mut Ui,
    setter: &ParamSetter,
    params: &HostaParams,
    shared: &Shared,
    st: &mut EditorState,
) {
    ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
    let live = st.last_change.elapsed() < Duration::from_millis(400);
    header(ui, shared, st, live);

    // Controls + automation outputs.
    {
        let st_ref = &mut *st;
        let snap_outputs = st_ref.snap.outputs;
        let verdict = st_ref.snap.readout.verdict;
        two_columns(
            ui,
            0.56,
            |ui| controls_panel(ui, setter, params, shared, st_ref),
            |ui| outputs_panel(ui, setter, params, &snap_outputs, verdict),
        );
    }

    // Budget the remaining height: heatmaps flex, HUD and scope are fixed.
    let hud_h = 86.0;
    let scope_h = 140.0;
    let chrome = 3.0 * 8.0 + 2.0 * 64.0 + 40.0;
    let heat_h = (ui.available_height() - hud_h - scope_h - chrome).clamp(140.0, 460.0);

    let r = st.snap.readout;
    let df = st.snap.df;
    let EditorState {
        layout,
        snap,
        bi_tex,
        tri_tex,
        asym,
        ..
    } = &mut *st;
    let (layout, snap, asym) = (&*layout, &*snap, *asym);
    two_columns(
        ui,
        0.5,
        |ui| {
            let meta = format!(
                "Bonf crit {:.3} | hits {} / {}",
                r.crit_bi,
                r.bi_hits,
                layout.n_triads()
            );
            panel(
                ui,
                "3rd-order bicoherence b²(f1, f2) — even / IMD",
                &meta,
                |ui| {
                    let size = vec2(ui.available_width(), heat_h);
                    let hover = heatmap(
                        ui,
                        bi_tex,
                        "bi",
                        BI_W,
                        BI_H,
                        size,
                        asym,
                        |x, y| {
                            let f = layout.bi_cell_to_flat[y * BI_W + x];
                            (f >= 0).then(|| {
                                let k = f as usize;
                                let b2 = snap.b2[k];
                                (
                                    b2,
                                    b2 as f64 >= snap.readout.crit_bi,
                                    snap.biphase[k].sin().abs(),
                                )
                            })
                        },
                        (
                            format!("f2 ↑ 0..{:.0} Hz", BI_H as f64 * df),
                            format!(
                                "f1 → 0..{:.0} Hz  (f1+f2 ≤ {:.0} Hz)",
                                BI_W as f64 * df,
                                64.0 * df
                            ),
                        ),
                    );
                    let text = match hover.and_then(|(x, y)| {
                        let f = layout.bi_cell_to_flat[y * BI_W + x];
                        (f >= 0).then_some(f as usize)
                    }) {
                        Some(k) => {
                            let l = layout;
                            let psi = (snap.biphase[k] as f64).to_degrees();
                            let b2 = snap.b2[k] as f64;
                            format!(
                                "({:.0} + {:.0} → {:.0} Hz)  b²={:.3}  ψ={:+.0}°  |sinψ|={:.2}  {}",
                                l.bi_i1[k] as f64 * df,
                                l.bi_i2[k] as f64 * df,
                                l.bi_i3[k] as f64 * df,
                                b2,
                                psi,
                                psi.to_radians().sin().abs(),
                                if b2 >= r.crit_bi { "BONF SIG" } else { "noise" }
                            )
                        }
                        None => match r.top_triad {
                            Some(t) => format!(
                                "Top: ({:.0} + {:.0} → {:.0} Hz)  b²={:.3}  ψ={:+.0}°",
                                t.f1, t.f2, t.f3, t.b2, t.psi_deg
                            ),
                            None => "Hover to inspect a triad".into(),
                        },
                    };
                    ui.label(RichText::new(text).monospace().size(11.0).color(TEXT1));
                },
            );
        },
        |ui| {
            let meta = format!(
                "Bonf crit {:.3} | hits {} / {}",
                r.crit_tri,
                r.tri_hits,
                layout.n_quartets()
            );
            panel(
                ui,
                "4th-order tricoherence max t²(f1, f2, f3) — odd clip",
                &meta,
                |ui| {
                    let size = vec2(ui.available_width(), heat_h);
                    let hover = heatmap(
                        ui,
                        tri_tex,
                        "tri",
                        TRI_W,
                        TRI_H,
                        size,
                        asym,
                        |x, y| {
                            let c = y * TRI_W + x;
                            layout.tri_valid_proj[c].then(|| {
                                (
                                    snap.tri_val[c],
                                    snap.tri_sig[c],
                                    snap.tri_phase[c].sin().abs(),
                                )
                            })
                        },
                        (
                            format!("f2 ↑ 0..{:.0} Hz", TRI_H as f64 * df),
                            format!(
                                "f1 → 0..{:.0} Hz  (f1+f2+f3 ≤ {:.0} Hz)",
                                TRI_W as f64 * df,
                                36.0 * df
                            ),
                        ),
                    );
                    let text = match hover.filter(|&(x, y)| layout.tri_valid_proj[y * TRI_W + x]) {
                        Some((x, y)) => {
                            let c = y * TRI_W + x;
                            let (i1, i2) = (x + 1, y + 1);
                            let i3 = snap.tri_best_i3[c].max(1) as usize;
                            format!(
                                "({:.0} + {:.0} + {:.0} → {:.0} Hz)  t²={:.3}  ψ={:+.0}°  {}",
                                i1 as f64 * df,
                                i2 as f64 * df,
                                i3 as f64 * df,
                                (i1 + i2 + i3) as f64 * df,
                                snap.tri_val[c],
                                (snap.tri_phase[c] as f64).to_degrees(),
                                if snap.tri_sig[c] { "BONF SIG" } else { "noise" }
                            )
                        }
                        None => match r.top_quartet {
                            Some(q) => format!(
                                "Top: ({:.0} + {:.0} + {:.0} → {:.0} Hz)  t²={:.3}",
                                q.f1, q.f2, q.f3, q.f4, q.t2
                            ),
                            None => "Hover to inspect a quartet".into(),
                        },
                    };
                    ui.label(RichText::new(text).monospace().size(11.0).color(TEXT1));
                },
            );
        },
    );

    hud(ui, st, hud_h);

    let (snap, rta_ref) = (&st.snap, st.rta_ref.as_deref());
    two_columns(
        ui,
        0.5,
        |ui| {
            panel(
                ui,
                "Peak-triggered oscilloscope (decimated)",
                "pre-onset left · decay right",
                |ui| {
                    scope(ui, snap, scope_h);
                },
            );
        },
        |ui| {
            let meta = format!("0..{:.0} Hz  (df {:.1} Hz)", snap.fs_a * 0.5, df);
            panel(
                ui,
                "2nd-order RTA power spectrum (phase-blind)",
                &meta,
                |ui| {
                    rta(ui, snap, rta_ref, scope_h);
                },
            );
        },
    );
}

/// Lay out two columns in explicit, clipped rectangles (left gets `frac` of the width).
/// Returns nothing; advances the parent cursor past the taller column.
fn two_columns(ui: &mut Ui, frac: f32, left: impl FnOnce(&mut Ui), right: impl FnOnce(&mut Ui)) {
    let gap = 8.0;
    let avail = ui.available_rect_before_wrap();
    let lw = ((avail.width() - gap) * frac).floor();
    let rects = [
        Rect::from_min_size(avail.min, vec2(lw, avail.height())),
        Rect::from_min_max(pos2(avail.min.x + lw + gap, avail.min.y), avail.max),
    ];
    let mut bottom = avail.min.y;
    let mut run = |rect: Rect, add: &mut dyn FnMut(&mut Ui)| {
        let r = ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                ui.set_max_width(rect.width());
                add(ui);
            },
        );
        bottom = bottom.max(r.response.rect.bottom());
    };
    let mut left = Some(left);
    let mut right = Some(right);
    run(rects[0], &mut |ui| (left.take().unwrap())(ui));
    run(rects[1], &mut |ui| (right.take().unwrap())(ui));
    ui.advance_cursor_after_rect(Rect::from_min_max(avail.min, pos2(avail.max.x, bottom)));
}

// ---- header ------------------------------------------------------------------

fn verdict_colors(v: Verdict) -> (Color32, Color32) {
    match v {
        Verdict::EvenImd => (AMBER, Color32::from_rgb(0xfd, 0xe6, 0x8a)),
        Verdict::OddClip => (VIOLET, Color32::from_rgb(0xe9, 0xd5, 0xff)),
        Verdict::CausalPunch => (EMERALD, Color32::from_rgb(0xa7, 0xf3, 0xd0)),
        Verdict::ReversedArrow => (CRIMSON, Color32::from_rgb(0xfe, 0xca, 0xca)),
        Verdict::PhaseSmeared => (CYAN, Color32::from_rgb(0xcf, 0xfa, 0xfe)),
        _ => (
            Color32::from_rgb(0x33, 0x41, 0x55),
            Color32::from_rgb(0x94, 0xa3, 0xb8),
        ),
    }
}

fn header(ui: &mut Ui, shared: &Shared, st: &EditorState, live: bool) {
    let r = st.snap.readout;
    let w = ui.available_width() - (2.0 * 14.0 + 2.0);
    Frame::new()
        .fill(BG2)
        .stroke(Stroke::new(1.0f32, BORDER))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_width(w);
            ui.horizontal(|ui| {
                let (dot, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                let dot_col = if !live || r.active_hops == 0 {
                    TEXT2
                } else if r.gated {
                    AMBER
                } else {
                    EMERALD
                };
                ui.painter().circle_filled(dot.center(), 4.5, dot_col);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.label(RichText::new("HOSTA · HIGHER-ORDER SPECTRAL & TIME-ARROW ANALYZER").strong().size(14.0).color(TEXT0));
                    ui.label(
                        RichText::new("Rolling-EMA bicoherence (3rd) · tricoherence (4th) · Bandt-Pompe D_JS · increment skew")
                            .monospace()
                            .size(10.5)
                            .color(TEXT2),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let fs = shared.sample_rate();
                    let tele = format!(
                        "fs {:.0} → {:.0} Hz (/{}) | hop {:.0} µs | K_eff {:.0}",
                        fs, st.snap.fs_a, st.snap.decim, r.hop_us, r.k_eff3
                    );
                    Frame::new()
                        .fill(BG0)
                        .stroke(Stroke::new(1.0f32, BORDER))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(egui::Margin::symmetric(10, 6))
                        .show(ui, |ui| ui.label(RichText::new(tele).monospace().size(11.0).color(TEXT1)));

                    let (label, v) = if !live && r.active_hops == 0 {
                        ("IDLE / STANDBY", Verdict::Idle)
                    } else if r.gated {
                        ("GATED (SILENCE)", Verdict::Clean)
                    } else {
                        (r.verdict.label(), r.verdict)
                    };
                    let (border, text) = verdict_colors(v);
                    Frame::new()
                        .fill(border.gamma_multiply(0.18))
                        .stroke(Stroke::new(1.0f32, border))
                        .corner_radius(CornerRadius::same(255))
                        .inner_margin(egui::Margin::symmetric(16, 7))
                        .show(ui, |ui| ui.label(RichText::new(label).monospace().strong().size(12.5).color(text)))
                        .response
                        .on_hover_text(format!(
                            "Browser prototype's original rules would say: {}\n\
                             (they misread the clean mix as smeared and the all-pass kick as reversed)",
                            r.verdict_legacy.label()
                        ));
                });
            });
        });
}

// ---- panels ------------------------------------------------------------------

/// Frame inner margin + stroke on both sides.
const PANEL_CHROME: f32 = 2.0 * 10.0 + 2.0 * 1.0;

fn panel(ui: &mut Ui, title: &str, meta: &str, add: impl FnOnce(&mut Ui)) {
    let w = ui.available_width() - PANEL_CHROME;
    Frame::new()
        .fill(BG1)
        .stroke(Stroke::new(1.0f32, BORDER))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(w);
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(title.to_uppercase())
                        .monospace()
                        .strong()
                        .size(11.0)
                        .color(TEXT0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(meta).monospace().size(10.5).color(TEXT2));
                });
            });
            ui.separator();
            add(ui);
        });
}

fn param_row<P: Param>(ui: &mut Ui, label: &str, p: &P, setter: &ParamSetter, tip: &str) {
    ui.label(RichText::new(label).monospace().size(11.0).color(TEXT1))
        .on_hover_text(tip);
    ui.add(widgets::ParamSlider::for_param(p, setter).with_width(150.0))
        .on_hover_text(tip);
}

fn controls_panel(
    ui: &mut Ui,
    setter: &ParamSetter,
    params: &HostaParams,
    shared: &Shared,
    st: &mut EditorState,
) {
    let meta = format!(
        "K_eff(3) {:.1} | K_eff(4) {:.1}",
        st.snap.readout.k_eff3, st.snap.readout.k_eff4
    );
    panel(ui, "Analysis engine", &meta, |ui| {
        egui::Grid::new("ctl-grid").num_columns(4).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
            param_row(ui, "EMA time τ", &params.tau, setter,
                "Averaging time of all statistics. Longer = steadier and more sensitive to weak coupling, slower to react.");
            param_row(ui, "Input", &params.input, setter,
                "Which signal is analysed. Audio output is always the untouched input.");
            ui.end_row();
            param_row(ui, "Silence gate", &params.gate, setter,
                "Below this level the statistics freeze instead of decaying.");
            param_row(ui, "Coupling floor", &params.floor, setter,
                "Bins quieter than this (relative to the loudest bin) are ignored by the coupling tests. \
                 Stops window leakage of pure tones from reading as distortion.");
            ui.end_row();
        });
        ui.horizontal(|ui| {
            if ui.button("Reset EMA").clicked() {
                shared.reset_request.store(true, Ordering::Release);
            }
            let asym_btn = ui.add(egui::Button::new("Im[B] asymmetry shader").selected(st.asym)).on_hover_text(
                "Weight the heatmaps by sin²(biphase): highlights quadrature (time-asymmetric) coupling \
                 and hides static real-valued distortion coupling.",
            );
            if asym_btn.clicked() {
                st.asym = !st.asym;
            }
            let locked = st.rta_ref.is_some();
            if ui.add(egui::Button::new(if locked { "Clear RTA ref" } else { "Lock RTA ref" }).selected(locked)).clicked() {
                st.rta_ref = if locked { None } else { Some(st.snap.rta_db.clone()) };
            }
        });
        colorbar(ui);
    });
}

fn outputs_panel(
    ui: &mut Ui,
    setter: &ParamSetter,
    params: &HostaParams,
    outputs: &[f32; N_OUTPUTS],
    verdict: Verdict,
) {
    panel(
        ui,
        "Phase-driven automation (MIDI CC out)",
        "map in your DAW",
        |ui| {
            ui.horizontal(|ui| {
                let mut on = params.midi_on.value();
                if ui
                    .checkbox(&mut on, RichText::new("Send").monospace().size(11.0))
                    .changed()
                {
                    setter.begin_set_parameter(&params.midi_on);
                    setter.set_parameter(&params.midi_on, on);
                    setter.end_set_parameter(&params.midi_on);
                }
                ui.label(RichText::new("ch").monospace().size(11.0).color(TEXT1));
                ui.add(
                    widgets::ParamSlider::for_param(&params.midi_channel, setter).with_width(56.0),
                );
                ui.label(RichText::new("CC").monospace().size(11.0).color(TEXT1));
                ui.add(widgets::ParamSlider::for_param(&params.cc_base, setter).with_width(56.0));
                ui.label(RichText::new("smooth").monospace().size(11.0).color(TEXT1));
                ui.add(
                    widgets::ParamSlider::for_param(&params.out_smooth, setter).with_width(80.0),
                );
            });
            let base = params.cc_base.value();
            let on = params.midi_on.value();
            let tips = [
            "How far the strongest 3rd-order (f1+f2→f3) coupling is above the significance threshold. Even distortion, IMD mud.",
            "Same for 4th-order (f1+f2+f3→f4) coupling. Symmetric (odd) clipping.",
            "Transient direction. 64 = none; up = forward (attack first); down = reversed. Needs ordinal confirmation, so noise stays at 64.",
            "Ordinal-pattern irreversibility D_JS, full scale at 0.01 nats.",
            "High when the material is transient (crest > 13 dB) but has lost its time direction: pre-ringing, all-pass smear.",
            "Peak/RMS crest factor, 0–24 dB.",
            "Discrete verdict: value = code × 16. 0 idle, 16 warm-up, 32 clean, 48 even/IMD, 64 odd clip, 80 causal, 96 reversed, 112 smeared.",
        ];
            for k in 0..N_OUTPUTS {
                let v = outputs[k];
                ui.horizontal(|ui| {
                    ui.add_sized(
                        vec2(150.0, 16.0),
                        egui::Label::new(
                            RichText::new(OUTPUT_NAMES[k])
                                .monospace()
                                .size(10.5)
                                .color(TEXT1),
                        ),
                    )
                    .on_hover_text(tips[k]);
                    ui.add_sized(
                        vec2(48.0, 16.0),
                        egui::Label::new(
                            RichText::new(format!("CC{}", base + k as i32))
                                .monospace()
                                .size(10.5)
                                .color(if on { CYAN } else { TEXT2 }),
                        ),
                    );
                    let (rect, resp) = ui.allocate_exact_size(
                        vec2(ui.available_width() - 44.0, 9.0),
                        Sense::hover(),
                    );
                    resp.on_hover_text(tips[k]);
                    let p = ui.painter();
                    p.rect_filled(rect, CornerRadius::same(4), BG3);
                    if k == 2 {
                        // Bipolar arrow bar.
                        let mid = rect.center().x;
                        let x = rect.left() + rect.width() * v;
                        let col = if v >= 0.5 { EMERALD } else { CRIMSON };
                        p.rect_filled(
                            Rect::from_x_y_ranges(mid.min(x)..=mid.max(x), rect.y_range()),
                            CornerRadius::same(4),
                            col,
                        );
                        p.vline(mid, rect.y_range(), Stroke::new(1.0f32, TEXT2));
                    } else {
                        let fill = Rect::from_min_size(
                            rect.min,
                            vec2(rect.width() * v.clamp(0.0, 1.0), rect.height()),
                        );
                        p.rect_filled(
                            fill,
                            CornerRadius::same(4),
                            if k == 6 {
                                verdict_colors(verdict).0
                            } else {
                                CYAN
                            },
                        );
                    }
                    ui.label(
                        RichText::new(format!("{:>3}", (v * 127.0).round() as i32))
                            .monospace()
                            .size(10.5)
                            .color(TEXT0),
                    );
                });
            }
        },
    );
}

fn colorbar(ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("0.0").monospace().size(10.0).color(TEXT2));
        let (rect, _) = ui.allocate_exact_size(
            vec2((ui.available_width() - 260.0).max(60.0), 8.0),
            Sense::hover(),
        );
        let n = 64;
        for i in 0..n {
            let t0 = i as f32 / n as f32;
            let r = Rect::from_x_y_ranges(
                rect.left() + rect.width() * t0
                    ..=rect.left() + rect.width() * (t0 + 1.0 / n as f32) + 0.5,
                rect.y_range(),
            );
            ui.painter()
                .rect_filled(r, CornerRadius::ZERO, colormap(t0));
        }
        ui.label(RichText::new("1.0").monospace().size(10.0).color(TEXT2));
        ui.label(
            RichText::new("[outlined = Bonferroni p<0.05]")
                .monospace()
                .size(10.0)
                .color(Color32::from_rgb(0xfc, 0xa5, 0xa5)),
        );
    });
}

// ---- heatmaps ----------------------------------------------------------------

/// Prototype colormap: obsidian → indigo → cyan → amber → crimson → white.
fn colormap(t: f32) -> Color32 {
    const STOPS: [(f32, [f32; 3]); 6] = [
        (0.00, [8.0, 12.0, 22.0]),
        (0.24, [30.0, 37.0, 107.0]),
        (0.50, [12.0, 166.0, 202.0]),
        (0.74, [245.0, 158.0, 11.0]),
        (0.90, [239.0, 68.0, 68.0]),
        (1.00, [255.0, 247.0, 236.0]),
    ];
    let t = t.clamp(0.0, 1.0);
    for w in STOPS.windows(2) {
        let ((t0, c0), (t1, c1)) = (w[0], w[1]);
        if t <= t1 {
            let f = (t - t0) / (t1 - t0);
            let c = |i: usize| (c0[i] + f * (c1[i] - c0[i])).round() as u8;
            return Color32::from_rgb(c(0), c(1), c(2));
        }
    }
    Color32::from_rgb(255, 247, 236)
}

fn tag(p: &egui::Painter, pos: Pos2, anchor: Align2, text: &str, color: Color32) {
    let galley = p.layout_no_wrap(text.to_owned(), FontId::monospace(10.0), color);
    let rect = anchor
        .anchor_size(pos, galley.size())
        .expand2(vec2(5.0, 2.0));
    p.rect_filled(
        rect,
        CornerRadius::same(4),
        Color32::from_rgba_unmultiplied(6, 9, 15, 190),
    );
    p.galley(rect.min + vec2(5.0, 2.0), galley, color);
}

/// Draw a grid heatmap; `cell(x, y)` returns (coherence, significant, |sin ψ|) or None outside the domain.
/// Row 0 is drawn at the bottom. Returns the hovered cell.
#[allow(clippy::too_many_arguments)]
fn heatmap(
    ui: &mut Ui,
    tex: &mut Option<TextureHandle>,
    name: &str,
    w: usize,
    h: usize,
    size: Vec2,
    asym: bool,
    cell: impl Fn(usize, usize) -> Option<(f32, bool, f32)>,
    axes: (String, String),
) -> Option<(usize, usize)> {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    let mut img = ColorImage::new([w, h], BG0);
    let mut sig_cells = Vec::new();
    for y in 0..h {
        let row = h - 1 - y;
        for x in 0..w {
            img.pixels[row * w + x] = match cell(x, y) {
                Some((coh, sig, s)) => {
                    if sig {
                        sig_cells.push((x, y));
                    }
                    let raw = if asym { coh * s * s } else { coh };
                    colormap(raw.clamp(0.0, 1.0).powf(0.76))
                }
                None => {
                    if (x + 2 * y) % 5 == 0 {
                        Color32::from_rgb(10, 17, 30)
                    } else {
                        Color32::from_rgb(6, 9, 15)
                    }
                }
            };
        }
    }
    let t = tex.get_or_insert_with(|| {
        ui.ctx()
            .load_texture(name, img.clone(), TextureOptions::NEAREST)
    });
    t.set(img, TextureOptions::NEAREST);

    let p = ui.painter_at(rect);
    p.image(
        t.id(),
        rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    let (cw, ch) = (rect.width() / w as f32, rect.height() / h as f32);
    let cell_rect = |x: usize, y: usize| {
        Rect::from_min_size(
            pos2(
                rect.left() + x as f32 * cw,
                rect.bottom() - (y + 1) as f32 * ch,
            ),
            vec2(cw, ch),
        )
    };
    // Faint cell grid.
    let grid = Stroke::new(1.0f32, Color32::from_rgba_unmultiplied(0, 0, 0, 40));
    for x in 1..w {
        p.vline(rect.left() + x as f32 * cw, rect.y_range(), grid);
    }
    for y in 1..h {
        p.hline(rect.x_range(), rect.top() + y as f32 * ch, grid);
    }
    let glow = if asym {
        Color32::from_rgba_unmultiplied(255, 210, 90, 200)
    } else {
        Color32::from_rgba_unmultiplied(250, 245, 255, 190)
    };
    for &(x, y) in &sig_cells {
        p.rect_stroke(
            cell_rect(x, y).shrink(0.5),
            CornerRadius::ZERO,
            Stroke::new(1.2f32, glow),
            StrokeKind::Inside,
        );
    }

    let hover = resp.hover_pos().and_then(|pp| {
        let nx = (pp.x - rect.left()) / rect.width();
        let ny = (rect.bottom() - pp.y) / rect.height();
        ((0.0..1.0).contains(&nx) && (0.0..1.0).contains(&ny))
            .then(|| ((nx * w as f32) as usize, (ny * h as f32) as usize))
    });
    if let Some((hx, hy)) = hover {
        let band = Color32::from_rgba_unmultiplied(0x24, 0xd6, 0xf5, 40);
        let c = cell_rect(hx, hy);
        p.rect_filled(
            Rect::from_x_y_ranges(c.x_range(), rect.y_range()),
            CornerRadius::ZERO,
            band,
        );
        p.rect_filled(
            Rect::from_x_y_ranges(rect.x_range(), c.y_range()),
            CornerRadius::ZERO,
            band,
        );
        p.rect_stroke(
            c,
            CornerRadius::ZERO,
            Stroke::new(1.5f32, Color32::WHITE),
            StrokeKind::Outside,
        );
    }
    tag(
        &p,
        rect.left_top() + vec2(6.0, 6.0),
        Align2::LEFT_TOP,
        &axes.0,
        TEXT0,
    );
    tag(
        &p,
        rect.right_bottom() - vec2(6.0, 6.0),
        Align2::RIGHT_BOTTOM,
        &axes.1,
        TEXT0,
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(1.0f32, BORDER),
        StrokeKind::Outside,
    );
    hover
}

// ---- HUD ---------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn card(
    ui: &mut Ui,
    width: f32,
    height: f32,
    label: &str,
    value: String,
    sub: String,
    tip: &str,
    extra: impl FnOnce(&mut Ui),
) {
    ui.allocate_ui_with_layout(
        vec2(width, height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            Frame::new()
                .fill(BG0)
                .stroke(Stroke::new(1.0f32, BORDER))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.set_width(width - 22.0);
                    ui.set_min_height(height - 16.0);
                    ui.spacing_mut().item_spacing.y = 3.0;
                    ui.label(
                        RichText::new(label.to_uppercase())
                            .monospace()
                            .size(9.5)
                            .color(TEXT2),
                    );
                    ui.label(
                        RichText::new(value)
                            .monospace()
                            .strong()
                            .size(17.0)
                            .color(TEXT0),
                    );
                    extra(ui);
                    ui.label(RichText::new(sub).monospace().size(9.5).color(TEXT2));
                })
                .response
                .on_hover_text(tip);
        },
    );
}

fn hud(ui: &mut Ui, st: &EditorState, height: f32) {
    let r = st.snap.readout;
    let n = 6.0;
    let w = ((ui.available_width() - 8.0 * (n - 1.0)) / n).floor();
    ui.horizontal_top(|ui| {
        card(ui, w, height, "Increment skew φ_rev", format!("{:+.3}", r.phi_rev), "−reverse ← 0 → +causal".into(),
            "Skewness of sample-to-sample increments. Fast attack + slow decay is positive; a reversed hit is negative; \
             smearing pulls it toward 0.",
            |ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 7.0), Sense::hover());
                let p = ui.painter();
                p.rect_filled(rect, CornerRadius::same(4), BG2);
                let c = (r.phi_rev / 1.5).clamp(-1.0, 1.0) as f32;
                let mid = rect.center().x;
                let x = mid + c * rect.width() * 0.5;
                p.rect_filled(
                    Rect::from_x_y_ranges(mid.min(x)..=mid.max(x), rect.y_range()),
                    CornerRadius::same(4),
                    if c >= 0.0 { EMERALD } else { CRIMSON },
                );
                p.vline(mid, rect.y_range(), Stroke::new(1.0f32, TEXT2));
            });
        card(ui, w, height, "Ordinal irrev D_JS", format!("{:.4}", r.d_js), "nats (D=3, 64 transitions)".into(),
            "Jensen-Shannon divergence between forward and time-reversed ordinal-pattern transitions. \
             Zero for any time-symmetric signal, whatever its spectrum.", |_| {});
        card(ui, w, height, "Imag bispec Im_RMS", format!("{:.4}", r.im_rms),
            format!("raw {:.3} · polarity {:+.2}", r.im_rms_raw, r.im_polarity),
            "RMS of b²·sin²ψ over all triads with the noise expectation 1/(2K) removed. Time-asymmetric \
             material has quadrature biphases; static distortion has real ones.", |_| {});
        let crest_db = if r.crest > 1e-6 { 20.0 * r.crest.log10() } else { 0.0 };
        card(ui, w, height, "Peak crest factor", format!("{:.2}", r.crest), format!("{crest_db:.1} dB (peak/RMS)"),
            "0.64 s max-hold peak over EMA RMS. Gates the transient (time-arrow) verdicts.", |_| {});
        card(ui, w, height, "Spectral entropy", format!("{:.3}", r.h_spec), "0 = pure tone · 1 = white".into(),
            "Normalised Shannon entropy of the power spectrum.", |_| {});
        card(ui, w, height, "Peak coupling", format!("{:.2} / {:.2}", r.b2_max, r.t2_max),
            format!("b² ({} sig) / t² ({} sig)", r.bi_hits, r.tri_hits),
            "Strongest bicoherence / tricoherence cell and how many cells pass the Bonferroni threshold.", |_| {});
    });
}

// ---- scope & RTA -------------------------------------------------------------

fn scope(ui: &mut Ui, s: &Snapshot, height: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, CornerRadius::same(6), BG0);
    let grid = Stroke::new(1.0f32, Color32::from_rgba_unmultiplied(53, 76, 115, 70));
    p.hline(rect.x_range(), rect.center().y, grid);
    p.vline(rect.left() + rect.width() * 0.35, rect.y_range(), grid);

    // Peak-centred trigger over the whole ring (~0.5 s, so a beat is always caught):
    // pre-ringing to the left, decay/chirp to the right.
    let n = SCOPE_LEN;
    let view = 512;
    let pre = (view as f32 * 0.35).round() as usize;
    let (mut peak_idx, mut peak) = (n - view + pre, 0.0f32);
    for i in pre..n - (view - pre) {
        if s.scope[i].abs() > peak {
            peak = s.scope[i].abs();
            peak_idx = i;
        }
    }
    let start = peak_idx.saturating_sub(pre).min(n - view);
    let scale = if peak > 0.01 {
        0.42 * rect.height() / peak
    } else {
        0.42 * rect.height()
    };
    let pts: Vec<Pos2> = (0..view)
        .map(|i| {
            pos2(
                rect.left() + i as f32 / (view - 1) as f32 * rect.width(),
                rect.center().y - s.scope[start + i] * scale,
            )
        })
        .collect();
    p.add(Shape::line(pts, Stroke::new(1.4f32, CYAN)));
    let win_ms = view as f64 / s.fs_a * 1000.0;
    p.text(
        rect.left_top() + vec2(8.0, 6.0),
        Align2::LEFT_TOP,
        format!("window {win_ms:.1} ms | peak {peak:.3}"),
        FontId::monospace(10.0),
        TEXT1,
    );
    p.text(
        pos2(rect.left() + rect.width() * 0.35 + 4.0, rect.top() + 6.0),
        Align2::LEFT_TOP,
        "trigger",
        FontId::monospace(10.0),
        TEXT2,
    );
}

fn rta(ui: &mut Ui, s: &Snapshot, reference: Option<&[f32]>, height: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, CornerRadius::same(6), BG0);
    let (db_min, db_max) = (-85.0f32, -5.0f32);
    let map_y = |db: f32| {
        let t = ((db - db_min) / (db_max - db_min)).clamp(0.0, 1.0);
        rect.bottom() - 10.0 - t * (rect.height() - 18.0)
    };
    let map_x = |b: usize| rect.left() + (b - 1) as f32 / (RTA_BINS - 2) as f32 * rect.width();
    let grid = Stroke::new(1.0f32, Color32::from_rgba_unmultiplied(53, 76, 115, 64));
    for db in [-20.0, -40.0, -60.0, -80.0] {
        let y = map_y(db);
        p.hline(rect.x_range(), y, grid);
        p.text(
            pos2(rect.left() + 6.0, y - 2.0),
            Align2::LEFT_BOTTOM,
            format!("{db:.0} dB"),
            FontId::monospace(9.5),
            TEXT2,
        );
    }

    let pts: Vec<Pos2> = (1..RTA_BINS)
        .map(|b| pos2(map_x(b), map_y(s.rta_db[b])))
        .collect();
    let mut mesh = Mesh::default();
    let fill = Color32::from_rgba_unmultiplied(56, 189, 248, 36);
    for w in pts.windows(2) {
        let i = mesh.vertices.len() as u32;
        mesh.colored_vertex(w[0], fill);
        mesh.colored_vertex(w[1], fill);
        mesh.colored_vertex(pos2(w[1].x, rect.bottom()), fill);
        mesh.colored_vertex(pos2(w[0].x, rect.bottom()), fill);
        mesh.add_triangle(i, i + 1, i + 2);
        mesh.add_triangle(i, i + 2, i + 3);
    }
    p.add(Shape::mesh(mesh));
    if let Some(r) = reference {
        let rp: Vec<Pos2> = (1..RTA_BINS).map(|b| pos2(map_x(b), map_y(r[b]))).collect();
        p.extend(Shape::dashed_line(
            &rp,
            Stroke::new(1.4f32, AMBER),
            5.0,
            4.0,
        ));
        p.text(
            rect.right_top() + vec2(-8.0, 6.0),
            Align2::RIGHT_TOP,
            "REF LOCKED (dashed)",
            FontId::monospace(10.0),
            AMBER,
        );
    }
    p.add(Shape::line(pts, Stroke::new(1.6f32, SKY)));
    let nyq = s.fs_a as f32 * 0.5;
    for frac in [0.25f32, 0.5, 0.75] {
        p.text(
            pos2(rect.left() + frac * rect.width(), rect.bottom() - 2.0),
            Align2::CENTER_BOTTOM,
            format!("{:.0} Hz", nyq * frac),
            FontId::monospace(9.5),
            TEXT1,
        );
    }
}
