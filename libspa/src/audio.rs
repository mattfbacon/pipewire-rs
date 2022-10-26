use crate::pod::{
    self,
    serialize::{GenError, ObjectPodSerializer, PodSerialize, PodSerializer, SerializeSuccess},
};
use crate::utils;
use std::{io, ops::Range};

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

struct AudioPosition {
    position: [u32; spa_sys::SPA_AUDIO_MAX_CHANNELS as _],
    channels: u32,
}

impl AudioPosition {
    fn new(pos: &[u32]) -> Self {
        let mut position = [0; spa_sys::SPA_AUDIO_MAX_CHANNELS as _];
        let channels = pos.len();
        position[0..channels].copy_from_slice(pos);
        Self {
            position,
            channels: channels as u32,
        }
    }
}
impl PodSerialize for AudioPosition {
    fn serialize<O: io::Write + io::Seek>(
        &self,
        serializer: PodSerializer<O>,
    ) -> Result<SerializeSuccess<O>, GenError> {
        let mut serializer = serializer.serialize_array::<utils::Id>(self.channels)?;
        for p in &self.position[0..self.channels as usize] {
            serializer.serialize_element(&utils::Id(*p))?;
        }
        serializer.end()
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

impl AudioInfoRaw {
    /// Serialize [`Self`] into an [`ObjectPodSerializer`].
    pub fn serialize<O: io::Write + io::Seek>(
        &self,
        mut serializer: ObjectPodSerializer<O>,
    ) -> Result<SerializeSuccess<O>, GenError> {
        serializer.serialize_property(
            spa_sys::SPA_FORMAT_mediaType,
            &utils::Id(spa_sys::SPA_MEDIA_TYPE_audio),
            pod::PropertyFlags::empty(),
        )?;

        serializer.serialize_property(
            spa_sys::SPA_FORMAT_mediaSubtype,
            &utils::Id(spa_sys::SPA_MEDIA_SUBTYPE_raw),
            pod::PropertyFlags::empty(),
        )?;

        if self.format != AudioFormat::UNKNOWN {
            serializer.serialize_property(
                spa_sys::SPA_FORMAT_AUDIO_format,
                &utils::Id(self.format.into()),
                pod::PropertyFlags::empty(),
            )?;
        }

        if self.rate != 0 {
            serializer.serialize_property(
                spa_sys::SPA_FORMAT_AUDIO_rate,
                &(self.rate as i32),
                pod::PropertyFlags::empty(),
            )?;
        }

        if self.channels != 0 {
            serializer.serialize_property(
                spa_sys::SPA_FORMAT_AUDIO_channels,
                &(self.channels as i32),
                pod::PropertyFlags::empty(),
            )?;
            if let Some(position) = self.position {
                serializer.serialize_property(
                    spa_sys::SPA_FORMAT_AUDIO_position,
                    &AudioPosition::new(&position[0..self.channels as usize]),
                    pod::PropertyFlags::empty(),
                )?;
            }
        }

        serializer.end()
    }
}
