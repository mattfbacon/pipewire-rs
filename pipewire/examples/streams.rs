// Copyright The pipewire-rs Contributors.
// SPDX-License-Identifier: MIT

//! This file is a rustic interpretation of the the [PipeWire Tutorial 5][tut]
//!
//! tut: https://docs.pipewire.org/page_tutorial5.html

use pipewire as pw;
use pw::prelude::*;
use pw::{properties, spa};

use clap::Parser;

struct UserData {
    format: spa::param::video::VideoInfoRaw,
}

#[derive(Parser)]
#[clap(name = "streams", about = "Stream example")]
struct Opt {
    #[clap(short, long, help = "The target object id to connect to")]
    target: Option<u32>,
}

pub fn main() -> Result<(), pw::Error> {
    pw::init();

    let opt = Opt::parse();

    let mainloop = pw::MainLoop::new()?;

    let data = UserData {
        format: Default::default(),
    };

    let stream = pw::stream::Stream::<UserData>::with_user_data(
        &mainloop,
        "video-test",
        properties! {
            *pw::keys::MEDIA_TYPE => "Video",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Camera",
        },
        data,
    )
    .state_changed(|old, new| {
        println!("State changed: {:?} -> {:?}", old, new);
    })
    .param_changed(|_, id, user_data, param| {
        if param.is_null() || id != pw::spa::param::ParamType::Format.as_raw() {
            return;
        }

        let (media_type, media_subtype) = unsafe {
            match pw::spa::param::format_utils::spa_parse_format(param) {
                Ok(v) => v,
                Err(_) => return,
            }
        };

        if media_type != pw::spa::format::MediaType::Video
            || media_subtype != pw::spa::format::MediaSubtype::Raw
        {
            return;
        }

        unsafe {
            user_data
                .format
                .parse(param)
                .expect("Failed to parse param changed to VideoInfoRaw")
        };

        println!("got video format:");
        println!(
            "  format: {} ({:?})",
            user_data.format.format().as_raw(),
            user_data.format.format()
        );
        println!(
            "  size: {}x{}",
            user_data.format.size().width,
            user_data.format.size().height
        );
        println!(
            "  framerate: {}/{}",
            user_data.format.framerate().num,
            user_data.format.framerate().denom
        );

        // prepare to render video of this size
    })
    .process(|stream, _| {
        match stream.dequeue_buffer() {
            None => println!("out of buffers"),
            Some(mut buffer) => {
                let datas = buffer.datas_mut();
                if datas.is_empty() {
                    return;
                }

                // copy frame data to screen
                let data = &mut datas[0];
                println!("got a frame of size {}", data.chunk().size());
            }
        }
    })
    .create()?;

    println!("Created stream {:#?}", stream);

    let obj = pw::spa::pod::object!(
        pw::spa::utils::SpaTypes::ObjectParamFormat,
        pw::spa::param::ParamType::EnumFormat,
        pw::spa::pod::property!(
            pw::spa::format::FormatProperties::MediaType,
            Id,
            pw::spa::format::MediaType::Video
        ),
        pw::spa::pod::property!(
            pw::spa::format::FormatProperties::MediaSubtype,
            Id,
            pw::spa::format::MediaSubtype::Raw
        ),
        pw::spa::pod::property!(
            pw::spa::format::FormatProperties::VideoFormat,
            Choice,
            Enum,
            Id,
            pw::spa::param::video::VideoFormat::RGB,
            pw::spa::param::video::VideoFormat::RGB,
            pw::spa::param::video::VideoFormat::RGBA,
            pw::spa::param::video::VideoFormat::RGBx,
            pw::spa::param::video::VideoFormat::BGRx,
            pw::spa::param::video::VideoFormat::YUY2,
            pw::spa::param::video::VideoFormat::I420,
        ),
        pw::spa::pod::property!(
            pw::spa::format::FormatProperties::VideoSize,
            Choice,
            Range,
            Rectangle,
            pw::spa::utils::Rectangle {
                width: 320,
                height: 240
            },
            pw::spa::utils::Rectangle {
                width: 1,
                height: 1
            },
            pw::spa::utils::Rectangle {
                width: 4096,
                height: 4096
            }
        ),
        pw::spa::pod::property!(
            pw::spa::format::FormatProperties::VideoFramerate,
            Choice,
            Range,
            Fraction,
            pw::spa::utils::Fraction { num: 25, denom: 1 },
            pw::spa::utils::Fraction { num: 0, denom: 1 },
            pw::spa::utils::Fraction {
                num: 1000,
                denom: 1
            }
        ),
    );
    let values = pw::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pw::spa::pod::Value::Object(obj),
    )
    .unwrap()
    .0
    .into_inner();

    let mut params = [values.as_ptr() as *const spa_sys::spa_pod];

    stream.connect(
        spa::Direction::Input,
        opt.target,
        pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
        &mut params,
    )?;

    println!("Connected stream");

    mainloop.run();

    unsafe { pw::deinit() };

    Ok(())
}
