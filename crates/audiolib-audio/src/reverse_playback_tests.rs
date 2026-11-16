// crates/audiolib-audio/src/reverse_playback_tests.rs
//! Reverse decoder/transport proofs use real PCM and worker state without devices.
use super::*;

fn collect(mut decoder: AuditionDecoder) -> Vec<f32> {
    let mut result = Vec::new();
    while !decoder.is_finished() {
        let block = decoder.next().unwrap();
        assert!(block.len() <= 4096);
        result.extend(block);
    }
    result
}

#[test]
fn reverse_preview_matches_reversed_original_frames_before_resampling_and_pitch() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.wav");
    let reference = temp.path().join("reference.wav");
    let frames = 7003;
    let samples: Vec<f32> = (0..frames)
        .flat_map(|i| {
            let value = (i as f32 * 0.037).sin() * (0.1 + i as f32 / 40000.0);
            [value, -value * 0.3, value * 0.7 + 0.01, -value * 0.2 - 0.03]
        })
        .collect();
    let bytes: Vec<u8> = samples.iter().flat_map(|v| v.to_le_bytes()).collect();
    crate::tests::write_wav(&source, 48000, 4, 32, 3, &bytes, false, false);
    let start = 123usize;
    let end = 6551usize;
    let reversed: Vec<u8> = samples[start * 4..end * 4]
        .chunks_exact(4)
        .rev()
        .flatten()
        .flat_map(|v| v.to_le_bytes())
        .collect();
    crate::tests::write_wav(&reference, 48000, 4, 32, 3, &reversed, false, false);
    for (output_rate, rate, pitch, mono, pair) in [
        (48000, 1.0, 0.0, false, 0),
        (48000, 1.0, 0.0, false, 1),
        (44100, 0.5, 7.0, false, 1),
        (48000, 1.5, -5.0, true, 0),
    ] {
        let reverse = AuditionSettings {
            rate,
            pitch_semitones: pitch,
            mono,
            reverse: true,
            ..Default::default()
        };
        let actual = collect(
            AuditionDecoder::new_bounded(
                Decoder::open(&source).unwrap(),
                output_rate,
                start as u64,
                end as u64,
                pair,
                reverse,
            )
            .unwrap(),
        );
        let expected = collect(
            AuditionDecoder::new(
                Decoder::open(&reference).unwrap(),
                output_rate,
                0,
                pair,
                AuditionSettings {
                    reverse: false,
                    ..reverse
                },
            )
            .unwrap(),
        );
        assert_eq!(actual.len(), expected.len());
        assert_eq!(
            actual.len() / 2,
            (((end - start) as f64 * f64::from(output_rate) / 48000.0 / f64::from(rate)).round())
                as usize
        );
        assert!(
            actual
                .iter()
                .zip(&expected)
                .all(|(a, b)| (a - b).abs() < 1e-6)
        );
    }
    // A seek resumes backwards at its source cursor, rather than reversing a tail.
    let actual = collect(
        AuditionDecoder::new(
            Decoder::open(&source).unwrap(),
            48000,
            4321,
            1,
            AuditionSettings {
                reverse: true,
                ..Default::default()
            },
        )
        .unwrap(),
    );
    let expected: Vec<f32> = samples[..4321 * 4]
        .chunks_exact(4)
        .rev()
        .flat_map(|frame| [frame[2], frame[3]])
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn compressed_reverse_preview_matches_forward_decoded_frames_across_blocks() {
    let temp = tempfile::tempdir().unwrap();
    for (name, bytes) in [
        (
            "source.mp3",
            include_bytes!("fixtures/mp3/vbr-xing.mp3").as_slice(),
        ),
        (
            "source.ogg",
            include_bytes!("fixtures/vorbis/comments.ogg").as_slice(),
        ),
        (
            "source.flac",
            include_bytes!("fixtures/vorbis/comments.flac").as_slice(),
        ),
    ] {
        let path = temp.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        let mut forward = Decoder::open(&path).unwrap();
        let info = forward.info.metadata.clone();
        let start = info.frames / 5;
        let end = (start + 8003).min(info.frames);
        forward.seek(start).unwrap();
        let samples = forward.read((end - start) as usize).unwrap();
        let channels = usize::from(info.channels);
        let expected: Vec<f32> = samples
            .chunks_exact(channels)
            .rev()
            .flat_map(|frame| [frame[0], frame[usize::from(channels > 1)]])
            .collect();
        let actual = collect(
            AuditionDecoder::new_bounded(
                Decoder::open(&path).unwrap(),
                info.sample_rate,
                start,
                end,
                0,
                AuditionSettings {
                    reverse: true,
                    ..Default::default()
                },
            )
            .unwrap(),
        );
        assert_eq!(actual.len(), expected.len(), "{name}");
        assert!(
            actual
                .iter()
                .zip(&expected)
                .all(|(a, b)| (a - b).abs() < 1e-6),
            "{name}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn reverse_transport_retains_source_range_session_and_directional_bounds() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("range.wav");
    let output = tests::prepared_output(&path);
    let mut worker = tests::worker(PlaybackStatus::default());
    worker.output = Some(output);
    worker.execute_command(AudioCommand::Load(path)).unwrap();
    worker
        .execute_command(AudioCommand::SetAudition(AuditionSettings {
            reverse: true,
            ..Default::default()
        }))
        .unwrap();
    assert_eq!(worker.snapshot().position, 1.0);
    let range = AudioFrameRange {
        start_frame: 200,
        end_frame: 600,
        sample_rate: 1000,
    };
    worker
        .execute_command(AudioCommand::PlayRange {
            range,
            expected_generation: worker.snapshot().generation,
        })
        .unwrap();
    let first = worker.snapshot();
    assert_eq!(first.position, 0.6);
    assert_eq!(first.playback_range, Some(range));
    assert_eq!(playback_start(&first), 0.6);
    assert_eq!(playback_end(&first), 0.2);
    worker
        .output
        .as_ref()
        .unwrap()
        .realtime
        .consumed
        .store(100, Ordering::Relaxed);
    assert!((worker.output.as_ref().unwrap().position() - 0.5).abs() < 1e-9);
    worker.execute_command(AudioCommand::Pause).unwrap();
    assert!((worker.snapshot().position - 0.5).abs() < 1e-9);
    for reverse in [false, true] {
        worker
            .execute_command(AudioCommand::SetAudition(AuditionSettings {
                reverse,
                ..Default::default()
            }))
            .unwrap();
        assert!((worker.snapshot().position - 0.5).abs() < 1e-9);
        assert_eq!(worker.snapshot().play_session_id, first.play_session_id);
        assert_eq!(worker.snapshot().playback_range, Some(range));
    }
    worker.execute_command(AudioCommand::Stop).unwrap();
    assert_eq!(worker.snapshot().position, 0.6);
    worker.execute_command(AudioCommand::Play).unwrap();
    assert_eq!(worker.snapshot().position, 0.6);
    assert_eq!(worker.snapshot().play_session_id, first.play_session_id + 1);
    worker.execute_command(AudioCommand::Seek(-1.0)).unwrap();
    assert_eq!(worker.snapshot().position, 0.2);
    worker.execute_command(AudioCommand::Pause).unwrap();
    assert_eq!(worker.snapshot().state, PlaybackState::Ended);
    worker.execute_command(AudioCommand::Play).unwrap();
    assert_eq!(worker.snapshot().position, 0.6);
    worker.execute_command(AudioCommand::ClearRange).unwrap();
    assert_eq!(worker.snapshot().playback_range, None);
    assert!(worker.snapshot().audition.reverse);
}

#[test]
fn reverse_settings_are_backward_compatible_and_local_load_starts_at_the_end() {
    let old: AuditionSettings = serde_json::from_str(
        r#"{"pitch_semitones":3,"rate":1.25,"mono":true,"loop_enabled":true}"#,
    )
    .unwrap();
    assert!(!old.reverse);
    let reverse = AuditionSettings {
        reverse: true,
        ..old
    };
    assert_eq!(
        serde_json::from_value::<AuditionSettings>(serde_json::to_value(reverse).unwrap()).unwrap(),
        reverse
    );
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("loaded.wav");
    let output = tests::prepared_output(&path);
    let mut worker = tests::worker(PlaybackStatus::default());
    worker.output = Some(output);
    worker.config.audition.reverse = true;
    worker.execute_command(AudioCommand::Load(path)).unwrap();
    let state = worker.snapshot();
    assert_eq!(state.position, state.duration);
    assert!(state.audition.reverse);
}

#[test]
fn buffered_source_direction_is_rejected_without_sticking_reverse_on() {
    use crate::source::{
        ByteSource, SourceOpener, register_stream_source, unregister_stream_source,
    };
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("device.wav");
    let output = tests::prepared_output(&path);
    let bytes = std::fs::read(&path).unwrap();
    let virtual_path = temp.path().join("registered.wav");
    let opener: SourceOpener = Arc::new(move || {
        let source: Box<dyn ByteSource> = Box::new(std::io::Cursor::new(bytes.clone()));
        Ok((source, bytes.len() as u64))
    });
    register_stream_source(&virtual_path, opener);
    let mut worker = tests::worker(PlaybackStatus::default());
    worker.output = Some(output);
    worker
        .execute_command(AudioCommand::Load(virtual_path.clone()))
        .unwrap();
    assert!(
        worker
            .execute_command(AudioCommand::SetAudition(AuditionSettings {
                reverse: true,
                ..Default::default()
            }))
            .is_err()
    );
    assert!(!worker.snapshot().audition.reverse);
    assert!(!worker.config.audition.reverse);
    unregister_stream_source(&virtual_path);
}
