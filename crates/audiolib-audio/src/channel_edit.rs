// crates/audiolib-audio/src/channel_edit.rs
//! Local channel choices shared by audition and the edited WAV.
use audiolib_core::{Error, Result};

/// Whether the edited signal retains the original layout or produces one channel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AudioChannelEdit {
    #[default]
    Original,
    /// Use the given zero-based source channel.
    Single(u16),
    /// Average the channels in a zero-based stereo pair. A final unpaired lane is used as-is.
    MonoPair(u16),
}

impl AudioChannelEdit {
    pub(crate) fn indices(self, channels: u16) -> Result<Option<(usize, Option<usize>)>> {
        let (first, second) = match self {
            Self::Original => return Ok(None),
            Self::Single(channel) => (u32::from(channel), None),
            Self::MonoPair(pair) => {
                let first = u32::from(pair) * 2;
                (
                    first,
                    (first + 1 < u32::from(channels)).then_some(first as usize + 1),
                )
            }
        };
        if first >= u32::from(channels) {
            return Err(Error::Validation(
                "The selected channel is outside the source layout".into(),
            ));
        }
        Ok(Some((first as usize, second)))
    }
}
