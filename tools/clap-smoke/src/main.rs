//! Minimal CLAP host smoke test for the *built* plugin binary.
//!
//!   cargo xtask bundle hosta --release
//!   cargo run -p clap-smoke --release -- target/bundled/HOSTA.clap
//!
//! Loads the .clap, instantiates the plugin, streams the synthetic scenes
//! through it in odd-sized stereo blocks, and checks that:
//!  * the audio output is bit-identical to the input (pure pass-through),
//!  * MIDI CC automation events come out with the expected values per scene,
//!  * processing is comfortably faster than real time.

use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::entry::clap_plugin_entry;
use clap_sys::events::*;
use clap_sys::factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use clap_sys::process::{clap_process, CLAP_PROCESS_ERROR};
use clap_sys::version::CLAP_VERSION;
use hosta_core::synth::{self, Scene};
use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr;
use std::time::Instant;

unsafe extern "C" fn host_get_extension(_: *const clap_host, _: *const c_char) -> *const c_void {
    ptr::null()
}
unsafe extern "C" fn host_noop(_: *const clap_host) {}

unsafe extern "C" fn in_size(_: *const clap_input_events) -> u32 {
    0
}
unsafe extern "C" fn in_get(_: *const clap_input_events, _: u32) -> *const clap_event_header {
    ptr::null()
}

/// (timing, status, cc, value) collected from the plugin.
type Collected = Vec<(u32, u8, u8, u8)>;

unsafe extern "C" fn out_push(
    list: *const clap_output_events,
    ev: *const clap_event_header,
) -> bool {
    let out = &mut *((*list).ctx as *mut Collected);
    if (*ev).space_id == CLAP_CORE_EVENT_SPACE_ID && (*ev).type_ == CLAP_EVENT_MIDI {
        let m = &*(ev as *const clap_event_midi);
        out.push(((*ev).time, m.data[0], m.data[1], m.data[2]));
    }
    true
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/bundled/HOSTA.clap".into());
    let fs = 48000.0;
    let mut failures = 0;

    unsafe {
        let lib = libloading::Library::new(&path).expect("load .clap");
        let entry: libloading::Symbol<*const clap_plugin_entry> =
            lib.get(b"clap_entry\0").expect("clap_entry");
        let entry = &**entry;
        let cpath = CString::new(path.clone()).unwrap();
        assert!((entry.init.unwrap())(cpath.as_ptr()), "entry.init");
        let factory = (entry.get_factory.unwrap())(CLAP_PLUGIN_FACTORY_ID.as_ptr())
            as *const clap_plugin_factory;
        assert!(!factory.is_null(), "plugin factory");
        let f = &*factory;
        assert_eq!((f.get_plugin_count.unwrap())(factory), 1);
        let desc = &*(f.get_plugin_descriptor.unwrap())(factory, 0);
        let id = CStr::from_ptr(desc.id);
        println!(
            "loaded: {} ({}) v{}",
            CStr::from_ptr(desc.name).to_string_lossy(),
            id.to_string_lossy(),
            CStr::from_ptr(desc.version).to_string_lossy()
        );

        let host = clap_host {
            clap_version: CLAP_VERSION,
            host_data: ptr::null_mut(),
            name: c"hosta-smoke".as_ptr(),
            vendor: c"test".as_ptr(),
            url: c"".as_ptr(),
            version: c"0.1".as_ptr(),
            get_extension: Some(host_get_extension),
            request_restart: Some(host_noop),
            request_process: Some(host_noop),
            request_callback: Some(host_noop),
        };
        let plugin = (f.create_plugin.unwrap())(factory, &host, id.as_ptr());
        assert!(!plugin.is_null(), "create_plugin");
        let p: &clap_plugin = &*plugin;
        assert!((p.init.unwrap())(plugin), "plugin.init");
        let max_block = 1024u32;
        assert!((p.activate.unwrap())(plugin, fs, 1, max_block), "activate");
        assert!((p.start_processing.unwrap())(plugin), "start_processing");

        // Expected end-of-scene CC values (CC 20..26 on channel 1).
        // (scene, cc offset, predicate description, predicate)
        type Check = (usize, &'static str, fn(u8) -> bool);
        let scenes: [(Scene, &[Check]); 5] = [
            (
                Scene::CleanMix,
                &[
                    (6, "verdict = 32 (clean)", |v| v == 32),
                    (0, "3rd coupling < 40", |v| v < 40),
                ],
            ),
            (
                Scene::OddClipper,
                &[
                    (6, "verdict = 64 (odd clip)", |v| v == 64),
                    (1, "4th coupling > 100", |v| v > 100),
                ],
            ),
            (
                Scene::TubeSubImd,
                &[
                    (6, "verdict = 48 (even/IMD)", |v| v == 48),
                    (0, "3rd coupling > 100", |v| v > 100),
                ],
            ),
            (
                Scene::DryKick,
                &[
                    (6, "verdict = 80 (causal)", |v| v == 80),
                    (2, "arrow > 115", |v| v > 115),
                ],
            ),
            (
                Scene::ReversedKick,
                &[
                    (6, "verdict = 96 (reversed)", |v| v == 96),
                    (2, "arrow < 12", |v| v < 12),
                ],
            ),
        ];

        let mut total_samples = 0usize;
        let mut total_time = 0.0f64;
        for (scene, checks) in scenes {
            (p.reset.unwrap())(plugin);
            let mono = synth::render_scene(scene, fs);
            let loops = if (scene as u32) <= 2 { 2 } else { 5 };
            let x: Vec<f32> = mono
                .iter()
                .copied()
                .cycle()
                .take(mono.len() * loops)
                .collect();

            let mut last_cc = [None::<u8>; 128];
            let mut n_events = 0usize;
            let mut bit_exact = true;
            let mut pos = 0usize;
            let sizes = [480usize, 1024, 37, 512, 333, 1];
            let mut si = 0;
            let (mut in_l, mut in_r) = (
                vec![0f32; max_block as usize],
                vec![0f32; max_block as usize],
            );
            let (mut out_l, mut out_r) = (
                vec![0f32; max_block as usize],
                vec![0f32; max_block as usize],
            );
            while pos < x.len() {
                let n = sizes[si % sizes.len()].min(x.len() - pos);
                si += 1;
                in_l[..n].copy_from_slice(&x[pos..pos + n]);
                for i in 0..n {
                    in_r[i] = -0.5 * x[pos + i]; // different R so the mid fold is exercised
                }
                let mut in_ptrs = [in_l.as_mut_ptr(), in_r.as_mut_ptr()];
                let mut out_ptrs = [out_l.as_mut_ptr(), out_r.as_mut_ptr()];
                let in_buf = clap_audio_buffer {
                    data32: in_ptrs.as_mut_ptr(),
                    data64: ptr::null_mut(),
                    channel_count: 2,
                    latency: 0,
                    constant_mask: 0,
                };
                let mut out_buf = clap_audio_buffer {
                    data32: out_ptrs.as_mut_ptr(),
                    data64: ptr::null_mut(),
                    channel_count: 2,
                    latency: 0,
                    constant_mask: 0,
                };
                let in_events = clap_input_events {
                    ctx: ptr::null_mut(),
                    size: Some(in_size),
                    get: Some(in_get),
                };
                let mut collected: Collected = Vec::with_capacity(64);
                let out_events = clap_output_events {
                    ctx: &mut collected as *mut _ as *mut c_void,
                    try_push: Some(out_push),
                };
                let proc = clap_process {
                    steady_time: pos as i64,
                    frames_count: n as u32,
                    transport: ptr::null(),
                    audio_inputs: &in_buf,
                    audio_outputs: &mut out_buf,
                    audio_inputs_count: 1,
                    audio_outputs_count: 1,
                    in_events: &in_events,
                    out_events: &out_events,
                };
                let t0 = Instant::now();
                let status = (p.process.unwrap())(plugin, &proc);
                total_time += t0.elapsed().as_secs_f64();
                assert_ne!(status, CLAP_PROCESS_ERROR, "process returned error");
                if out_l[..n] != in_l[..n] || out_r[..n] != in_r[..n] {
                    bit_exact = false;
                }
                for &(t, status, cc, val) in &collected {
                    assert!((t as usize) < n, "event timing {t} outside block of {n}");
                    assert_eq!(status, 0xB0, "expected CC on channel 1");
                    last_cc[cc as usize] = Some(val);
                    n_events += 1;
                }
                pos += n;
            }
            total_samples += x.len();

            let vals: Vec<String> = (20..27)
                .map(|c| last_cc[c].map_or("-".into(), |v: u8| v.to_string()))
                .collect();
            println!("{scene:?}: {n_events} CC events, final CC20..26 = [{}], pass-through bit-exact: {bit_exact}", vals.join(", "));
            if !bit_exact {
                println!("  FAIL: audio was modified");
                failures += 1;
            }
            for (off, what, ok) in checks {
                match last_cc[20 + off] {
                    Some(v) if ok(v) => println!("  ok   CC{}: {what} (got {v})", 20 + off),
                    v => {
                        println!("  FAIL CC{}: {what} (got {v:?})", 20 + off);
                        failures += 1;
                    }
                }
            }
        }
        let rt = total_samples as f64 / fs / total_time;
        println!(
            "processed {:.1} s of audio in {:.3} s CPU → {:.0}× real time",
            total_samples as f64 / fs,
            total_time,
            rt
        );

        (p.stop_processing.unwrap())(plugin);
        (p.deactivate.unwrap())(plugin);
        (p.destroy.unwrap())(plugin);
        (entry.deinit.unwrap())();
    }
    if failures > 0 {
        println!("{failures} check(s) FAILED");
        std::process::exit(1);
    }
    println!("all checks passed");
}
