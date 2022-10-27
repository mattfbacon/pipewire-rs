// Copyright The pipewire-rs Contributors.
// SPDX-License-Identifier: MIT

use crate::pod::{Property, Value, ValueArray};
use crate::utils;
use std::ops::Range;

pub const MAX_CHANNELS: usize = spa_sys::SPA_AUDIO_MAX_CHANNELS as usize;

#[repr(transparent)]
#[derive(PartialEq, PartialOrd, Eq, Clone, Copy)]
pub struct AudioFormat(u32);

impl From<AudioFormat> for u32 {
    fn from(value: AudioFormat) -> Self {
        value.0
    }
}

impl From<u32> for AudioFormat {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl AudioFormat {
    pub const UNKNOWN: Self = Self(spa_sys::SPA_AUDIO_FORMAT_UNKNOWN);
    pub const ENCODED: Self = Self(spa_sys::SPA_AUDIO_FORMAT_ENCODED);
    pub const S8: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S8);
    pub const U8: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U8);
    pub const S16_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S16_LE);
    pub const S16_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S16_BE);
    pub const U16_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U16_LE);
    pub const U16_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U16_BE);
    pub const S24_32_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S24_32_LE);
    pub const S24_32_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S24_32_BE);
    pub const U24_32_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U24_32_LE);
    pub const U24_32_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U24_32_BE);
    pub const S32_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S32_LE);
    pub const S32_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S32_BE);
    pub const U32_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U32_LE);
    pub const U32_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U32_BE);
    pub const S24_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S24_LE);
    pub const S24_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S24_BE);
    pub const U24_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U24_LE);
    pub const U24_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U24_BE);
    pub const S20_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S20_LE);
    pub const S20_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S20_BE);
    pub const U20_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U20_LE);
    pub const U20_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U20_BE);
    pub const S18_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S18_LE);
    pub const S18_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_S18_BE);
    pub const U18_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U18_LE);
    pub const U18_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_U18_BE);
    pub const F32_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_F32_LE);
    pub const F32_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_F32_BE);
    pub const F64_LE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_F64_LE);
    pub const F64_BE: Self = Self(spa_sys::SPA_AUDIO_FORMAT_F64_BE);

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
}

/// Rust representation of [`spa_sys::spa_audio_info_raw`].
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct AudioInfoRaw {
    pub format: AudioFormat,
    pub channels: u32,
    pub rate: u32,
    pub position: Option<[u32; spa_sys::SPA_AUDIO_MAX_CHANNELS as _]>,
}

impl Default for AudioInfoRaw {
    fn default() -> Self {
        Self {
            format: AudioFormat::UNKNOWN,
            channels: 0,
            rate: 0,
            position: None,
        }
    }
}

impl From<AudioInfoRaw> for spa_sys::spa_audio_info_raw {
    fn from(value: AudioInfoRaw) -> Self {
        let AudioInfoRaw {
            channels,
            format,
            rate,
            position,
        } = value;
        let (flags, position) = if let Some(position) = position {
            (spa_sys::SPA_AUDIO_FLAG_NONE, position)
        } else {
            (
                spa_sys::SPA_AUDIO_FLAG_UNPOSITIONED,
                [0; spa_sys::SPA_AUDIO_MAX_CHANNELS as _],
            )
        };
        spa_sys::spa_audio_info_raw {
            channels,
            format: format.into(),
            rate,
            flags,
            position,
        }
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

        let AudioInfoRaw {
            format,
            position,
            rate,
            channels,
        } = value;

        if format != AudioFormat::UNKNOWN {
            props.push(Property::new(
                spa_sys::SPA_FORMAT_AUDIO_format,
                Value::Id(utils::Id(format.into())),
            ));
        }

        if rate != 0 {
            props.push(Property::new(
                spa_sys::SPA_FORMAT_AUDIO_rate,
                Value::Int(rate as i32),
            ));
        }

        if channels != 0 {
            props.push(Property::new(
                spa_sys::SPA_FORMAT_AUDIO_channels,
                Value::Int(channels as i32),
            ));
            if let Some(position) = position {
                let array = position[0..channels as usize]
                    .iter()
                    .copied()
                    .map(|p| utils::Id(p))
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
