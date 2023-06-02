// Copyright The pipewire-rs Contributors.
// SPDX-License-Identifier: MIT

use crate::format::{MediaSubtype, MediaType};
use crate::{Error, SpaResult};

/// helper function to parse format properties type
///
/// # Safety
///
/// `format` is not validated to be a valid SPA Pod, which may lead to undefined behaviour.
pub unsafe fn spa_parse_format(
    format: *const spa_sys::spa_pod,
) -> Result<(MediaType, MediaSubtype), Error> {
    let mut media_type: u32 = 0;
    let mut media_subtype: u32 = 0;

    if format.is_null() {
        return Err(SpaResult::from_c(-libc::EINVAL).into_result().unwrap_err());
    }

    let res = spa_sys::spa_format_parse(format.cast_mut(), &mut media_type, &mut media_subtype);
    match SpaResult::from_c(res).into_sync_result() {
        Err(e) => Err(e),
        Ok(_) => Ok((
            MediaType::from_raw(media_type),
            MediaSubtype::from_raw(media_subtype),
        )),
    }
}
