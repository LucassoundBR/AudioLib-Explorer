// crates/audiolib-audio/src/fade.rs
//! A shared source-frame envelope for audition and WAV export.

/// Linear fade lengths in source frames, measured from the two playback ends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AudioFade {
    pub fade_in: u64,
    pub fade_out: u64,
}

impl AudioFade {
    /// Whether this envelope preserves every sample.
    pub fn is_empty(self) -> bool {
        self == Self::default()
    }

    /// Fit the fades into the clip, retaining the leading fade first.
    pub fn bounded(self, frames: u64) -> Self {
        let fade_in = self.fade_in.min(frames);
        Self {
            fade_in,
            fade_out: self.fade_out.min(frames - fade_in),
        }
    }

    /// Ratio at a frame in playback order. The envelope must fit the clip.
    pub(crate) fn gain(self, frames: u64, elapsed: u64) -> (u64, u64) {
        let trailing = frames - 1 - elapsed;
        if elapsed < self.fade_in {
            (elapsed, self.fade_in)
        } else if trailing < self.fade_out {
            (trailing, self.fade_out)
        } else {
            (1, 1)
        }
    }
}
