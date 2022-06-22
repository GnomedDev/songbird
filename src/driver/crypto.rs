//! Encryption schemes supported by Discord's secure RTP negotiation.
use byteorder::{NetworkEndian, WriteBytesExt};
use discortp::{rtp::RtpPacket, MutablePacket};
use rand::Rng;
use std::num::Wrapping;
use xsalsa20poly1305::{
    aead::{AeadInPlace, Error as CryptoError},
    Nonce,
    XSalsa20Poly1305 as Cipher,
    NONCE_SIZE,
    TAG_SIZE,
};

/// Variants of the `XSalsa20Poly1305` encryption scheme.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CryptoMode {
    /// The RTP header is used as the source of nonce bytes for the packet.
    ///
    /// Equivalent to a nonce of at most 48b (6B) at no extra packet overhead:
    /// the RTP sequence number and timestamp are the varying quantities.
    Normal,
    /// An additional random 24B suffix is used as the source of nonce bytes for the packet.
    /// This is regenerated randomly for each packet.
    ///
    /// Full nonce width of 24B (192b), at an extra 24B per packet (~1.2 kB/s).
    Suffix,
    /// An additional random 4B suffix is used as the source of nonce bytes for the packet.
    /// This nonce value increments by `1` with each packet.
    ///
    /// Nonce width of 4B (32b), at an extra 4B per packet (~0.2 kB/s).
    Lite,
}

impl From<CryptoState> for CryptoMode {
    fn from(val: CryptoState) -> Self {
        match val {
            CryptoState::Normal => Self::Normal,
            CryptoState::Suffix => Self::Suffix,
            CryptoState::Lite(_) => Self::Lite,
        }
    }
}

impl CryptoMode {
    /// Returns the name of a mode as it will appear during negotiation.
    #[must_use]
    pub fn to_request_str(self) -> &'static str {
        match self {
            Self::Normal => "xsalsa20_poly1305",
            Self::Suffix => "xsalsa20_poly1305_suffix",
            Self::Lite => "xsalsa20_poly1305_lite",
        }
    }

    /// Returns the number of bytes each nonce is stored as within
    /// a packet.
    #[must_use]
    pub fn nonce_size(self) -> usize {
        match self {
            Self::Normal => RtpPacket::minimum_packet_size(),
            Self::Suffix => NONCE_SIZE,
            Self::Lite => 4,
        }
    }

    /// Returns the number of bytes occupied by the encryption scheme
    /// which fall before the payload.
    #[must_use]
    pub fn payload_prefix_len() -> usize {
        TAG_SIZE
    }

    /// Returns the number of bytes occupied by the encryption scheme
    /// which fall after the payload.
    #[must_use]
    pub fn payload_suffix_len(self) -> usize {
        match self {
            Self::Normal => 0,
            Self::Suffix | Self::Lite => self.nonce_size(),
        }
    }

    /// Calculates the number of additional bytes required compared
    /// to an unencrypted payload.
    #[must_use]
    pub fn payload_overhead(self) -> usize {
        Self::payload_prefix_len() + self.payload_suffix_len()
    }

    /// Extracts the byte slice in a packet used as the nonce, and the remaining mutable
    /// portion of the packet.
    fn nonce_slice<'a>(
        self,
        header: &'a [u8],
        body: &'a mut [u8],
    ) -> Result<(&'a [u8], &'a mut [u8]), CryptoError> {
        match self {
            Self::Normal => Ok((header, body)),
            Self::Suffix | Self::Lite => {
                let len = body.len();
                if len < self.payload_suffix_len() {
                    Err(CryptoError)
                } else {
                    let (body_left, nonce_loc) = body.split_at_mut(len - self.payload_suffix_len());
                    Ok((&nonce_loc[..self.nonce_size()], body_left))
                }
            },
        }
    }

    /// Encrypts a Discord RT(C)P packet using the given key.
    ///
    /// Use of this requires that the input packet has had a nonce generated in the correct location,
    /// and `payload_len` specifies the number of bytes after the header including this nonce.
    #[inline]
    pub fn encrypt_in_place(
        self,
        packet: &mut impl MutablePacket,
        cipher: &Cipher,
        payload_len: usize,
    ) -> Result<(), CryptoError> {
        let header_len = packet.packet().len() - packet.payload().len();
        let (header, body) = packet.packet_mut().split_at_mut(header_len);
        let (slice_to_use, body_remaining) = self.nonce_slice(header, &mut body[..payload_len])?;

        let mut nonce = Nonce::default();
        let nonce_slice = if slice_to_use.len() == NONCE_SIZE {
            Nonce::from_slice(&slice_to_use[..NONCE_SIZE])
        } else {
            nonce[..self.nonce_size()].copy_from_slice(slice_to_use);
            &nonce
        };

        // body_remaining is now correctly truncated by this point.
        // the true_payload to encrypt follows after the first TAG_LEN bytes.
        let tag =
            cipher.encrypt_in_place_detached(nonce_slice, b"", &mut body_remaining[TAG_SIZE..])?;
        body_remaining[..TAG_SIZE].copy_from_slice(&tag[..]);

        Ok(())
    }
}

/// State used in nonce generation for the `XSalsa20Poly1305` encryption variants
/// in [`CryptoMode`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CryptoState {
    /// The RTP header is used as the source of nonce bytes for the packet.
    ///
    /// No state is required.
    Normal,
    /// An additional random 24B suffix is used as the source of nonce bytes for the packet.
    /// This is regenerated randomly for each packet.
    ///
    /// No state is required.
    Suffix,
    /// An additional random 4B suffix is used as the source of nonce bytes for the packet.
    /// This nonce value increments by `1` with each packet.
    ///
    /// The last used nonce is stored.
    Lite(Wrapping<u32>),
}

impl From<CryptoMode> for CryptoState {
    fn from(val: CryptoMode) -> Self {
        match val {
            CryptoMode::Normal => CryptoState::Normal,
            CryptoMode::Suffix => CryptoState::Suffix,
            CryptoMode::Lite => CryptoState::Lite(Wrapping(rand::random::<u32>())),
        }
    }
}

impl CryptoState {
    /// Writes packet nonce into the body, if required, returning the new length.
    pub fn write_packet_nonce(
        &mut self,
        packet: &mut impl MutablePacket,
        payload_end: usize,
    ) -> usize {
        let mode = self.kind();
        let endpoint = payload_end + mode.payload_suffix_len();

        match self {
            Self::Suffix => {
                rand::thread_rng().fill(&mut packet.payload_mut()[payload_end..endpoint]);
            },
            Self::Lite(mut i) => {
                (&mut packet.payload_mut()[payload_end..endpoint])
                    .write_u32::<NetworkEndian>(i.0)
                    .expect(
                        "Nonce size is guaranteed to be sufficient to write u32 for lite tagging.",
                    );
                i += Wrapping(1);
            },
            _ => {},
        }

        endpoint
    }

    /// Returns the underlying (stateless) type of the active crypto mode.
    pub fn kind(self) -> CryptoMode {
        CryptoMode::from(self)
    }
}
