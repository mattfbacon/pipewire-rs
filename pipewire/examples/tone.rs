// Copyright The pipewire-rs Contributors.
// SPDX-License-Identifier: MIT

//! This file is a rustic interpretation of the the [PipeWire Tutorial 4][tut]
//!
//! tut: https://docs.pipewire.org/page_tutorial4.html

use pipewire as pw;
use pw::prelude::*;
use pw::{properties, spa};

pub const DEFAULT_RATE: u32 = 44100;
pub const DEFAULT_CHANNELS: u32 = 2;
pub const DEFAULT_VOLUME: f64 = 0.7;
pub const PI_2: f64 = std::f64::consts::PI + std::f64::consts::PI;
pub const CHAN_SIZE: usize = std::mem::size_of::<i16>();

pub fn main() -> Result<(), pw::Error> {
    pw::init();
    let mainloop = pw::MainLoop::new()?;

    let stream = pw::stream::Stream::<f64>::with_user_data(
        &mainloop,
        "audio-src",
        properties! {
            *pw::keys::MEDIA_TYPE => "Audio",
            *pw::keys::MEDIA_ROLE => "Music",
            *pw::keys::MEDIA_CATEGORY => "Playback",
        },
        0.0,
    )
    .process(|stream, acc| match stream.dequeue_buffer() {
        None => println!("No buffer received"),
        Some(mut buffer) => {
            let datas = buffer.datas_mut();
            let stride = CHAN_SIZE * DEFAULT_CHANNELS as usize;
            let data = &mut datas[0];
            let n_frames = if let Some(slice) = data.data() {
                let n_frames = slice.len() / stride;
                for i in 0..n_frames {
                    *acc += PI_2 * 440.0 / DEFAULT_RATE as f64;
                    if *acc >= PI_2 {
                        *acc -= PI_2
                    }
                    let val = (f64::sin(*acc) * DEFAULT_VOLUME * 16767.0) as i16;
                    for c in 0..DEFAULT_CHANNELS {
                        let start = i * stride + (c as usize * CHAN_SIZE);
                        let end = start + CHAN_SIZE;
                        let chan = &mut slice[start..end];
                        chan.copy_from_slice(&i16::to_le_bytes(val));
                    }
                }
                n_frames
            } else {
                0
            };
            let chunk = data.chunk_mut();
            *chunk.offset_mut() = 0;
            *chunk.stride_mut() = stride as _;
            *chunk.size_mut() = (stride * n_frames) as _;
        }
    })
    .create()?;

    let audio_info = pw::spa::audio::AudioInfoRaw {
        channels: DEFAULT_CHANNELS,
        rate: DEFAULT_RATE,
        format: pw::spa::audio::AudioFormat::S16_LE,
        ..Default::default()
    };

    let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pw::spa::pod::Value::Object(pw::spa::pod::Object {
            type_: spa_sys::SPA_TYPE_OBJECT_Format,
            id: spa_sys::SPA_PARAM_EnumFormat,
            properties: audio_info.into(),
        }),
    )
    .unwrap()
    .0
    .into_inner();

    let mut params = [values.as_ptr() as *const spa_sys::spa_pod];

    stream.connect(
        spa::Direction::Output,
        None,
        pw::stream::StreamFlags::AUTOCONNECT
            | pw::stream::StreamFlags::MAP_BUFFERS
            | pw::stream::StreamFlags::RT_PROCESS,
        &mut params,
    )?;

    mainloop.run();

    unsafe { pw::deinit() };

    Ok(())
}
