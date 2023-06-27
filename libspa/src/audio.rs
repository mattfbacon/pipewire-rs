// Copyright The pipewire-rs Contributors.
// SPDX-License-Identifier: MIT

use crate::pod::{Property, Value, ValueArray};
use crate::utils;
use std::ops::Range;

pub const MAX_CHANNELS: usize = spa_sys::SPA_AUDIO_MAX_CHANNELS as usize;

#[repr(transparent)]
#[derive(PartialEq, PartialOrd, Eq, Clone, Copy)]
pub struct AudioFormat(pub spa_sys::spa_audio_format);

#[allow(non_upper_case_globals)]
impl AudioFormat {
    pub const Unknown: Self = Self(spa_sys::SPA_AUDIO_FORMAT_UNKNOWN);
    pub const Encoded: Self = Self(spa_sys::SPA_AUDIO_FORMAT_ENCODED);
    pub const S8: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S8);
    pub const U8: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U8);
    pub const S16LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S16_LE);
    pub const S16BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S16_BE);
    pub const U16LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U16_LE);
    pub const U16BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U16_BE);
    pub const S24_32LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S24_32_LE);
    pub const S24_32BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S24_32_BE);
    pub const U24_32LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U24_32_LE);
    pub const U24_32BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U24_32_BE);
    pub const S32LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S32_LE);
    pub const S32BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S32_BE);
    pub const U32LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U32_LE);
    pub const U32BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U32_BE);
    pub const S24LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S24_LE);
    pub const S24BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S24_BE);
    pub const U24LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U24_LE);
    pub const U24BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U24_BE);
    pub const S20LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S20_LE);
    pub const S20BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S20_BE);
    pub const U20LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U20_LE);
    pub const U20BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U20_BE);
    pub const S18LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S18_LE);
    pub const S18BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S18_BE);
    pub const U18LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U18_LE);
    pub const U18BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U18_BE);
    pub const F32LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_F32_LE);
    pub const F32BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_F32_BE);
    pub const F64LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_F64_LE);
    pub const F64BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_F64_BE);

    pub const U8P: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U8P);
    pub const S16P: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S16P);
    pub const S24_32P: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S24_32P);
    pub const S32P: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S32P);
    pub const S24P: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S24P);
    pub const F32P: Self = Self(spa_sys::SPA_AUDIO_FORMAT_F32P);
    pub const F64P: Self = Self(spa_sys::SPA_AUDIO_FORMAT_F64P);
    pub const S8P: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S8P);

    const INTERLEAVED_RANGE: Range<Self> = Self::S8..Self(spa_sys::SPA_AUDIO_FORMAT_START_Planar);
    const PLANAR_RANGE: Range<Self> = Self::U8P..Self(spa_sys::SPA_AUDIO_FORMAT_START_Other);

    pub fn is_interleaved(&self) -> bool {
        Self::INTERLEAVED_RANGE.contains(self)
    }

    pub fn is_planar(&self) -> bool {
        Self::PLANAR_RANGE.contains(self)
    }

    /// Obtain an [`AudioFormat`] from a raw `spa_audio_format` variant.
    pub fn from_raw(raw: spa_sys::spa_audio_format) -> Self {
        Self(raw)
    }

    /// Get the raw [`spa_sys::spa_audio_format`] representing this `AudioFormat`.
    pub fn as_raw(&self) -> spa_sys::spa_audio_format {
        self.0
    }
}

bitflags::bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct AudioInfoRawFlags: u32 {
        /// the position array explicitly contains unpositioned channels.
        const UNPOSITIONED = 1<<0;
    }
}

/// Rust representation of [`spa_sys::spa_audio_info_raw`].
#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct AudioInfoRaw(spa_sys::spa_audio_info_raw);

impl AudioInfoRaw {
    pub fn new() -> Self {
        Self(spa_sys::spa_audio_info_raw {
            format: AudioFormat::Unknown.as_raw(),
            flags: AudioInfoRawFlags::UNPOSITIONED.bits(),
            rate: 0,
            channels: 0,
            position: [0; 64usize],
        })
    }

    pub fn set_format(&mut self, format: AudioFormat) {
        self.0.format = format.as_raw();
    }

    pub fn format(&self) -> AudioFormat {
        AudioFormat::from_raw(self.0.format)
    }

    pub fn set_flags(&mut self, flags: AudioInfoRawFlags) {
        self.0.flags = flags.bits();
    }

    pub fn flags(&self) -> AudioInfoRawFlags {
        AudioInfoRawFlags::from_bits_retain(self.0.flags)
    }

    pub fn set_rate(&mut self, rate: u32) {
        self.0.rate = rate;
    }

    pub fn rate(&self) -> u32 {
        self.0.rate
    }

    pub fn set_channels(&mut self, channels: u32) {
        self.0.channels = channels;
    }

    pub fn channels(&self) -> u32 {
        self.0.channels
    }

    pub fn set_position(&mut self, position: [u32; 64usize]) {
        self.0.position = position;
        if position[0] == 0 {
            self.0.flags |= AudioInfoRawFlags::UNPOSITIONED.bits();
        } else {
            self.0.flags &= AudioInfoRawFlags::UNPOSITIONED.complement().bits();
        };
    }

    pub fn position(&self) -> [u32; 64usize] {
        self.0.position
    }

    /// Obtain an [`AudioInfoRaw`] from a raw `spa_audio_info_raw` variant.
    pub fn from_raw(raw: spa_sys::spa_audio_info_raw) -> Self {
        Self(raw)
    }

    /// Get the raw [`spa_sys::spa_audio_info_raw`] representing this `AudioInfoRaw`.
    pub fn as_raw(&self) -> spa_sys::spa_audio_info_raw {
        self.0
    }
}

impl Default for AudioInfoRaw {
    fn default() -> Self {
        Self::new()
    }
}

impl From<AudioInfoRaw> for Vec<Property> {
    fn from(value: AudioInfoRaw) -> Self {
        let mut props = Vec::with_capacity(6);
        props.push(Property::new(
            spa_sys::SPA_FORMAT_mediaType,
            Value::Id(utils::Id(spa_sys::SPA_MEDIA_TYPE_audio)),
        ));
        props.push(Property::new(
            spa_sys::SPA_FORMAT_mediaSubtype,
            Value::Id(utils::Id(spa_sys::SPA_MEDIA_SUBTYPE_raw)),
        ));

        if value.format() != AudioFormat::Unknown {
            props.push(Property::new(
                spa_sys::SPA_FORMAT_AUDIO_format,
                Value::Id(utils::Id(value.format().as_raw())),
            ));
        }

        if value.rate() != 0 {
            props.push(Property::new(
                spa_sys::SPA_FORMAT_AUDIO_rate,
                Value::Int(value.rate() as i32),
            ));
        }

        if value.channels() != 0 {
            props.push(Property::new(
                spa_sys::SPA_FORMAT_AUDIO_channels,
                Value::Int(value.channels() as i32),
            ));
            if !value.flags().contains(AudioInfoRawFlags::UNPOSITIONED) {
                let array = value.position()[0..value.channels() as usize]
                    .iter()
                    .copied()
                    .map(utils::Id)
                    .collect();
                props.push(Property::new(
                    spa_sys::SPA_FORMAT_AUDIO_position,
                    Value::ValueArray(ValueArray::Id(array)),
                ));
            }
        }

        props
    }
}
