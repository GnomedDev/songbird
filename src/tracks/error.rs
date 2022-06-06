use crate::input::AudioStreamError;
use std::{
    error::Error,
    fmt::{self, Display},
    sync::Arc,
};
use symphonia_core::errors::Error as SymphoniaError;

/// Errors associated with control and manipulation of tracks.
///
/// Unless otherwise stated, these don't invalidate an existing track,
/// but do advise on valid operations and commands.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ControlError {
    /// The operation failed because the track has ended, has been removed
    /// due to call closure, or some error within the driver.
    Finished,
    /// The supplied event listener can never be fired by a track, and should
    /// be attached to the driver instead.
    InvalidTrackEvent,
    /// The track's underlying [`Input`] doesn't support seeking operations.
    ///
    /// [`Input`]: crate::input::Input
    SeekUnsupported,
}

impl Display for ControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to operate on track (handle): ")?;
        match self {
            ControlError::Finished => write!(f, "track ended"),
            ControlError::InvalidTrackEvent => {
                write!(f, "given event listener can't be fired on a track")
            },
            ControlError::SeekUnsupported => write!(f, "track did not support seeking"),
        }
    }
}

impl Error for ControlError {}

/// Alias for most calls to a [`TrackHandle`].
///
/// [`TrackHandle`]: super::TrackHandle
pub type TrackResult<T> = Result<T, ControlError>;

/// Errors reported by the mixer while attempting to play (or ready) a [`Track`].
///
/// [`Track`]: super::Track
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum PlayError {
    /// Failed to create a live bytestream from the lazy [`Compose`].
    ///
    /// [`Compose`]: crate::input::Compose
    Create(Arc<AudioStreamError>),
    /// Failed to read headers, codecs, or a valid stream from an [`Input`].
    ///
    /// [`Input`]: crate::input::Input
    Parse(Arc<SymphoniaError>),
    /// Failed to decode a frame received from an [`Input`].
    ///
    /// [`Input`]: crate::input::Input
    Decode(Arc<SymphoniaError>),
    /// Failed to seek to the requested location.
    Seek(Arc<SymphoniaError>),
}

impl Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("runtime error while playing track: ")?;
        match self {
            Self::Create(c) => {
                f.write_str("input creation [")?;
                f.write_fmt(format_args!("{}", &c))?;
                f.write_str("]")
            },
            Self::Parse(p) => {
                f.write_str("parsing formats/codecs [")?;
                f.write_fmt(format_args!("{}", &p))?;
                f.write_str("]")
            },
            Self::Decode(d) => {
                f.write_str("decoding packets [")?;
                f.write_fmt(format_args!("{}", &d))?;
                f.write_str("]")
            },
            Self::Seek(s) => {
                f.write_str("seeking along input [")?;
                f.write_fmt(format_args!("{}", &s))?;
                f.write_str("]")
            },
        }
    }
}

impl Error for PlayError {}
