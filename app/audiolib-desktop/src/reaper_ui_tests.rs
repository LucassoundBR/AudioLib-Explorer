// app/audiolib-desktop/src/reaper_ui_tests.rs
//! REAPER controls use the offscreen Slint backend, never native input or audio.
use crate::{AppWindow, Theme};
use i_slint_backend_testing::ElementHandle;
use slint::{
    ComponentHandle,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{cell::Cell, rc::Rc};

fn key(ui: &AppWindow, text: slint::SharedString) {
    ui.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    ui.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}

#[test]
fn reaper_shortcut_respects_search_editing_busy_state_and_setup_modal() {
    i_slint_backend_testing::init_no_event_loop();
    for modern in [false, true] {
        let ui = AppWindow::new().unwrap();
        ui.global::<Theme>().set_modern_style(modern);
        ui.global::<Theme>().set_modern_player(modern);
        ui.window()
            .set_size(slint::LogicalSize::new(1440.0, 1000.0));
        ui.set_reaper_supported(true);
        ui.set_has_selection(true);
        ui.set_waveform_ready(true);
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        ui.on_reaper_send_requested(move || observed.set(observed.get() + 1));
        ui.show().unwrap();
        key(&ui, "s".into());
        assert_eq!(calls.get(), 1, "S sends from the non-editing focus scope");
        ui.set_reaper_busy(true);
        key(&ui, "s".into());
        assert_eq!(calls.get(), 1, "An in-flight send cannot be duplicated");
        ui.set_reaper_busy(false);
        ui.set_reaper_setup_visible(true);
        key(&ui, "s".into());
        assert_eq!(calls.get(), 1);
        key(&ui, Key::Escape.into());
        assert!(!ui.get_reaper_setup_visible());
        let input = ElementHandle::find_by_element_id(
            &ui,
            if modern {
                "AppWindow::studio-search-input"
            } else {
                "AppWindow::search-input"
            },
        )
        .next()
        .unwrap();
        input.mock_single_click(PointerEventButton::Left);
        key(&ui, "s".into());
        assert_eq!(
            calls.get(),
            1,
            "Typing s in a search field must not send audio"
        );
        assert!(ui.get_search_entry_text().contains('s'));
        ui.hide().unwrap();
    }
}

/// Render the real controls and selection without creating native windows.
#[test]
#[ignore = "explicit offscreen PNG export for REAPER and waveform visual review"]
fn export_reaper_waveform_visual_previews() {
    use crate::waveform_view::{WaveView, render_waveform, render_waveform_mask};
    use audiolib_audio::{WaveformData, WaveformPeak};
    use audiolib_core::config::{DEFAULT_THEME, STUDIO_THEME};
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    let directory = std::path::PathBuf::from(
        std::env::var_os("AUDIOLIB_REAPER_SNAPSHOT_DIR").expect("Set owned snapshot directory"),
    );
    assert!(directory.is_absolute());
    std::fs::create_dir_all(&directory).unwrap();
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        threading: false,
        renderer_name: Some("software".into()),
    })))
    .unwrap();
    let waveform = WaveformData {
        sample_rate: 48_000,
        channels: 2,
        frames: 960_000,
        samples_per_pixel: 938,
        peaks: (0..2)
            .map(|channel| {
                (0..1024)
                    .map(|i| {
                        let amplitude =
                            0.08 + (i as f32 * 0.06 + channel as f32).sin().abs() * 0.65;
                        WaveformPeak {
                            min: -amplitude,
                            max: amplitude,
                            rms: amplitude * 0.5,
                            dominant_hz: 80.0 * 2.0_f32.powf(i as f32 / 160.0),
                            tonality: 0.8,
                            ..Default::default()
                        }
                    })
                    .collect()
            })
            .collect(),
    };
    for (modern, theme, name) in [
        (false, DEFAULT_THEME, "classic"),
        (true, STUDIO_THEME, "studio"),
    ] {
        let ui = AppWindow::new().unwrap();
        crate::theme::apply_theme(&ui, theme);
        ui.global::<Theme>().set_modern_style(modern);
        ui.global::<Theme>().set_modern_player(modern);
        ui.global::<Theme>().set_motion(0.0);
        ui.window()
            .set_size(slint::LogicalSize::new(1440.0, 1000.0));
        ui.set_reaper_supported(true);
        ui.set_has_selection(true);
        ui.set_waveform_ready(true);
        ui.set_playback_name("Synthetic ambience · 48 kHz / 24-bit.wav".into());
        ui.set_waveform_info("2 ch · 48 kHz · 0:20.000".into());
        ui.set_play_duration(20.0);
        ui.set_audio_ready(true);
        ui.set_playback_state(2);
        ui.set_view_start(0.0);
        ui.set_view_end(20.0);
        ui.set_selected_id("visual-fixture".into());
        ui.set_wave_selection_identity("visual:source:1".into());
        ui.set_audition_expanded(true);
        ui.set_audition_preferred_height(286.0);
        ui.set_wave_selection_active(true);
        ui.set_wave_selection_start(4.0);
        ui.set_wave_selection_end(13.0);
        ui.set_waveform_mask(slint::Image::from_rgba8(
            slint::SharedPixelBuffer::clone_from_slice(
                &render_waveform_mask(&waveform, 1440, 160, WaveView::default()),
                1440,
                160,
            ),
        ));
        ui.show().unwrap();
        for spectral in [false, true] {
            ui.set_peak_mode(if spectral { 1 } else { 0 });
            let pixels = render_waveform(
                &waveform,
                1440,
                160,
                WaveView {
                    spectral,
                    ..WaveView::default()
                },
            );
            ui.set_waveform_image(slint::Image::from_rgba8(
                slint::SharedPixelBuffer::clone_from_slice(&pixels, 1440, 160),
            ));
            let pixels = ui.window().take_snapshot().unwrap();
            image::save_buffer(
                directory.join(format!(
                    "{name}-{}.png",
                    if spectral { "spectral" } else { "normal" }
                )),
                pixels.as_bytes(),
                pixels.width(),
                pixels.height(),
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
        ui.set_reaper_script_path("C:\\Users\\Test\\AppData\\Local\\AudioLibExplorer\\v1\\reaper-bridge\\AudioLib_Explorer_Bridge.lua".into());
        ui.set_reaper_setup_visible(true);
        let pixels = ui.window().take_snapshot().unwrap();
        image::save_buffer(
            directory.join(format!("{name}-setup.png")),
            pixels.as_bytes(),
            pixels.width(),
            pixels.height(),
            image::ColorType::Rgba8,
        )
        .unwrap();
        ui.hide().unwrap();
    }
}
