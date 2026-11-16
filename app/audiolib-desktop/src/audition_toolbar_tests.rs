// app/audiolib-desktop/src/audition_toolbar_tests.rs
//! Offscreen interaction checks for the Classic player controls; no native input/audio.
use crate::{AppWindow, Theme};
use i_slint_backend_testing::ElementHandle;
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

fn settle() {
    i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(1));
}
fn control(ui: &AppWindow, label: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(ui, label)
        .next()
        .unwrap_or_else(|| panic!("Missing {label}"))
}
fn player() -> AppWindow {
    let ui = AppWindow::new().unwrap();
    ui.global::<Theme>().set_modern_style(false);
    ui.global::<Theme>().set_modern_player(false);
    ui.window()
        .set_size(slint::LogicalSize::new(1920.0, 1000.0));
    ui.set_audio_ready(true);
    ui.set_has_selection(true);
    ui.set_reaper_supported(true);
    ui.set_waveform_ready(true);
    ui.set_play_duration(20.0);
    ui.set_view_end(20.0);
    ui.set_wave_selection_identity("owned:synthetic".into());
    ui.show().unwrap();
    settle();
    ui
}

#[test]
fn transform_sliders_and_typed_values_stay_synchronized_and_obey_readiness() {
    i_slint_backend_testing::init_no_event_loop();
    let ui = player();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let observed = calls.clone();
    ui.on_transform_requested(move |p, r, m, l| observed.borrow_mut().push((p, r, m, l)));
    let pitch = control(&ui, "Pitch semitones; double-click to reset");
    let rate = control(&ui, "Rate; double-click to reset");
    pitch.set_accessible_value("7.25");
    rate.set_accessible_value("1.37");
    settle();
    assert_eq!(ui.get_pitch_text(), "7.25");
    assert_eq!(ui.get_speed_text(), "1.37");
    let last = calls.borrow().last().unwrap().clone();
    assert_eq!((last.0.as_str(), last.1.as_str()), ("7.25", "1.37"));
    // Engine publication and numeric reset must still move a previously dragged slider.
    ui.set_pitch_text("-3.00".into());
    ui.set_speed_text("0.75".into());
    settle();
    assert_eq!(pitch.accessible_value().as_deref(), Some("-3"));
    assert_eq!(rate.accessible_value().as_deref(), Some("0.75"));
    pitch.set_accessible_value("100");
    rate.set_accessible_value("-2");
    settle();
    assert_eq!(ui.get_pitch_text(), "24.00");
    assert_eq!(ui.get_speed_text(), "0.25");
    ui.set_audio_ready(false);
    settle();
    let count = calls.borrow().len();
    pitch.set_accessible_value("0");
    rate.invoke_accessible_increment_action();
    settle();
    assert_eq!(calls.borrow().len(), count);
    assert_eq!(ui.get_pitch_text(), "24.00");
    assert_eq!(ui.get_speed_text(), "0.25");
    ui.hide().unwrap();
}

#[test]
fn display_gain_is_vertical_visual_only_and_synchronized_with_peak_settings() {
    i_slint_backend_testing::init_no_event_loop();
    let ui = player();
    ui.set_volume_db(-12.0);
    ui.set_pitch_text("2.00".into());
    ui.set_speed_text("1.25".into());
    let changes = Rc::new(Cell::new(0));
    let observed = changes.clone();
    ui.on_peak_style_requested(move || observed.set(observed.get() + 1));
    ui.on_transform_requested(|_, _, _, _| panic!("Display Gain must not change audition"));
    ui.on_volume_db_requested(|_| panic!("Display Gain must not change audio gain"));
    let gain = control(&ui, "Display Gain; visual only; double-click to reset");
    assert!(gain.size().height > gain.size().width);
    gain.set_accessible_value("18");
    settle();
    assert_eq!(ui.get_peak_gain(), 18.0);
    assert_eq!(changes.get(), 1);
    ui.set_peak_gain(-6.0);
    settle();
    assert_eq!(gain.accessible_value().as_deref(), Some("-6"));
    assert_eq!(ui.get_volume_db(), -12.0);
    assert_eq!(ui.get_pitch_text(), "2.00");
    assert_eq!(ui.get_speed_text(), "1.25");
    ui.hide().unwrap();
}

#[test]
fn reaper_controls_share_the_player_row_at_fhd_and_keep_busy_guards() {
    i_slint_backend_testing::init_no_event_loop();
    let ui = player();
    let sends = Rc::new(Cell::new(0));
    let observed = sends.clone();
    ui.on_reaper_send_requested(move || observed.set(observed.get() + 1));
    let send = control(&ui, "Send to REAPER");
    let connect = control(&ui, "Connect REAPER");
    let pitch = control(&ui, "Pitch semitones; double-click to reset");
    for button in [&send, &connect] {
        let pos = button.absolute_position();
        assert!(pos.x >= 0.0 && pos.x + button.size().width <= 1920.0);
        assert!((pos.y - pitch.absolute_position().y).abs() < 32.0);
    }
    send.mock_single_click(PointerEventButton::Left);
    settle();
    assert_eq!(sends.get(), 1);
    let focus = control(&ui, "Focus REAPER after send");
    focus.mock_single_click(PointerEventButton::Left);
    settle();
    assert!(ui.get_reaper_auto_focus());
    ui.set_reaper_busy(true);
    settle();
    send.mock_single_click(PointerEventButton::Left);
    settle();
    assert_eq!(sends.get(), 1);
    focus.mock_single_click(PointerEventButton::Left);
    settle();
    assert!(ui.get_reaper_auto_focus());
    ui.hide().unwrap();
}

#[test]
fn shift_wheel_pans_without_zooming_or_changing_the_selection() {
    i_slint_backend_testing::init_no_event_loop();
    let ui = player();
    ui.set_wave_zoom(4.0);
    ui.set_wave_scroll(0.5);
    settle();
    let pan = Rc::new(Cell::new(-1.0));
    let observed = pan.clone();
    ui.on_waveform_pan_requested(move |v| observed.set(v));
    ui.on_waveform_wheel_zoom_requested(|_, _| panic!("Shift-wheel must pan"));
    ui.on_wave_selection_committed(|_, _| panic!("Panning must not create a region"));
    let wave = control(&ui, "Waveform timeline");
    let origin = wave.absolute_position();
    let size = wave.size();
    let position = LogicalPosition::new(origin.x + size.width * 0.5, origin.y + size.height * 0.5);
    ui.window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    ui.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    ui.window().dispatch_event(WindowEvent::PointerScrolled {
        position,
        delta_x: 0.0,
        delta_y: -120.0,
    });
    ui.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    settle();
    assert!((pan.get() - (0.5 + 0.25 / 3.0)).abs() < 0.001);
    ui.hide().unwrap();
}

#[test]
fn reverse_button_and_r_preserve_editor_modal_busy_and_send_acknowledgment_guards() {
    i_slint_backend_testing::init_no_event_loop();
    let ui = player();
    let changes = Rc::new(RefCell::new(Vec::new()));
    let observed = changes.clone();
    let weak = ui.as_weak();
    ui.on_transform_requested(move |_, _, _, _| {
        observed
            .borrow_mut()
            .push(weak.upgrade().unwrap().get_audition_reverse())
    });
    let sends = Rc::new(Cell::new(0));
    let observed = sends.clone();
    ui.on_reaper_send_requested(move || observed.set(observed.get() + 1));
    let key = |text: slint::SharedString| {
        ui.window()
            .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
        ui.window()
            .dispatch_event(WindowEvent::KeyReleased { text });
        settle();
    };
    key("r".into());
    assert!(ui.get_audition_reverse());
    assert_eq!(&*changes.borrow(), &[true]);
    key("s".into());
    assert_eq!(sends.get(), 0, "Send waits for the accepted direction");
    ui.set_confirmed_reverse(true);
    key("s".into());
    assert_eq!(sends.get(), 1);
    control(&ui, "Reverse audio").mock_single_click(PointerEventButton::Left);
    settle();
    assert!(!ui.get_audition_reverse());
    assert_eq!(&*changes.borrow(), &[true, false]);
    let search = ElementHandle::find_by_element_id(&ui, "AppWindow::search-input")
        .next()
        .unwrap();
    search.mock_single_click(PointerEventButton::Left);
    key("r".into());
    assert!(ui.get_search_entry_text().contains('r'));
    assert_eq!(changes.borrow().len(), 2);
    ui.set_reaper_setup_visible(true);
    key("r".into());
    assert_eq!(changes.borrow().len(), 2);
    key(Key::Escape.into());
    for gate in 0..5 {
        ui.set_wave_selection_preparing(false);
        ui.set_reaper_busy(false);
        ui.set_reverse_supported(true);
        ui.set_selected_missing(false);
        ui.set_audio_ready(true);
        match gate {
            0 => ui.set_wave_selection_preparing(true),
            1 => ui.set_reaper_busy(true),
            2 => ui.set_reverse_supported(false),
            3 => ui.set_selected_missing(true),
            _ => ui.set_audio_ready(false),
        }
        key("r".into());
        assert_eq!(changes.borrow().len(), 2, "blocked gate {gate}");
    }
    ui.hide().unwrap();
}
