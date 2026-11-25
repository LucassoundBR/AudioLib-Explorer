// crates/audiolib-audio/src/reverse_export.rs
//! Precision-preserving reverse export with bounded RAM and owned compressed spooling.
use super::*;
use std::{
    path::PathBuf,
    sync::atomic::AtomicU64,
    time::{SystemTime, UNIX_EPOCH},
};

/// Export original frames in descending order, retaining each frame's channel order,
/// precision and source sample rate. Timing-dependent Metadata is omitted just as in
/// `export_audio_slice`. PCM sources need one bounded copying pass; compressed sources
/// spool a precise forward WAV in a private child of the caller-owned destination
/// directory. Cancellation/errors remove only files/directories created by this call.
/// All file I/O runs on the caller's worker, never in a realtime audio callback.
pub fn export_reversed_audio_slice(
    source: &Path,
    destination: &Path,
    range: AudioFrameRange,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64, u64),
) -> Result<()> {
    export_audio_slice_ordered(source, destination, Some(range), true, cancel, progress)
}

/// Reverse the entire original, deriving its frame range from the locked source
/// header so a separate probe cannot race a source replacement. Ownership,
/// precision, cancellation and worker-thread rules match reversed slice export.
pub fn export_reversed_audio_file(
    source: &Path,
    destination: &Path,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64, u64),
) -> Result<()> {
    export_audio_slice_ordered(source, destination, None, true, cancel, progress)
}

struct Spool {
    directory: PathBuf,
    file: PathBuf,
}
impl Spool {
    fn new(destination: &Path) -> Result<Self> {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let parent = destination
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(audio_error)?
            .as_nanos();
        for _ in 0..32 {
            let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let directory = parent.join(format!(
                ".audiolib-reverse-{}-{stamp:x}-{sequence:x}",
                std::process::id()
            ));
            match std::fs::create_dir(&directory) {
                Ok(()) => {
                    let file = directory.join("forward.wav");
                    return Ok(Self { directory, file });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Err(invalid("Unable to allocate owned reverse spool"))
    }
}
impl Drop for Spool {
    fn drop(&mut self) {
        // Never recursively remove or traverse the destination directory.
        let _ = std::fs::remove_file(&self.file);
        let _ = std::fs::remove_dir(&self.directory);
    }
}

pub(super) fn export_via_pcm(
    source: &Path,
    destination: &Path,
    range: AudioFrameRange,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64, u64),
) -> Result<()> {
    let spool = Spool::new(destination)?;
    // Preserve the existing integer FLAC decoder rather than rounding through f32.
    export_audio_slice(source, &spool.file, range, cancel, &mut |_, _| {})?;
    cancelled(cancel)?;
    export_audio_slice_ordered(
        &spool.file,
        destination,
        Some(AudioFrameRange {
            start_frame: 0,
            end_frame: range.end_frame - range.start_frame,
            sample_rate: range.sample_rate,
        }),
        true,
        cancel,
        progress,
    )
}
