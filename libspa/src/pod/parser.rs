// Copyright The pipewire-rs Contributors.
// SPDX-License-Identifier: MIT

use std::{
    ffi::{c_char, c_double, c_float, c_int, c_long, c_void, CStr},
    marker::PhantomData,
    mem::MaybeUninit,
};

use nix::errno::Errno;

use crate::utils::{Fraction, Id, Rectangle};

/// Low-level wrapper around `spa_pod_parser`.
///
/// Using this may require using `unsafe` and/or working with C types, but
/// is still more safe and rusty than the raw functions and types.
#[repr(transparent)]
pub struct Parser<'d> {
    parser: spa_sys::spa_pod_parser,
    data: PhantomData<&'d [u8]>,
}

impl<'d> Parser<'d> {
    pub fn new(data: &'d [u8]) -> Self {
        unsafe {
            let mut parser: MaybeUninit<spa_sys::spa_pod_parser> = MaybeUninit::uninit();
            spa_sys::spa_pod_parser_init(
                parser.as_mut_ptr(),
                data.as_ptr().cast(),
                data.len()
                    .try_into()
                    .expect("data length does not fit in a u32"),
            );
            Self {
                parser: parser.assume_init(),
                data: PhantomData,
            }
        }
    }

    pub fn from_pod(pod: &'d crate::pod::Pod) -> Self {
        unsafe {
            let mut parser: MaybeUninit<spa_sys::spa_pod_parser> = MaybeUninit::uninit();
            spa_sys::spa_pod_parser_pod(parser.as_mut_ptr(), pod.as_raw_ptr());
            Self {
                parser: parser.assume_init(),
                data: PhantomData,
            }
        }
    }

    pub fn as_raw(&self) -> &spa_sys::spa_pod_parser {
        &self.parser
    }

    pub fn as_raw_ptr(&self) -> *mut spa_sys::spa_pod_parser {
        std::ptr::addr_of!(self.parser).cast_mut()
    }

    pub fn into_raw(self) -> spa_sys::spa_pod_parser {
        self.parser
    }

    /// # Safety
    ///
    /// The parser state may only be used as long as all frames that were pushed
    /// to the parser at the time of this call are alive and not moved
    pub unsafe fn state(&self) -> spa_sys::spa_pod_parser_state {
        let mut state: MaybeUninit<spa_sys::spa_pod_parser_state> = MaybeUninit::uninit();
        spa_sys::spa_pod_parser_get_state(self.as_raw_ptr(), state.as_mut_ptr());
        state.assume_init()
    }

    /// # Safety
    ///
    /// TODO: Constraints unknown, use at own risk
    pub unsafe fn reset(&mut self, state: *mut spa_sys::spa_pod_parser_state) {
        spa_sys::spa_pod_parser_reset(self.as_raw_ptr(), state)
    }

    /// # Safety
    ///
    /// TODO: Constraints unknown, use at own risk
    pub unsafe fn deref(&mut self, offset: u32, size: u32) -> *mut spa_sys::spa_pod {
        spa_sys::spa_pod_parser_deref(self.as_raw_ptr(), offset, size)
    }

    /// # Safety
    ///
    /// TODO: Constraints unknown, use at own risk
    pub unsafe fn frame(&mut self, frame: *mut spa_sys::spa_pod_frame) -> *mut spa_sys::spa_pod {
        spa_sys::spa_pod_parser_frame(self.as_raw_ptr(), frame)
    }

    /// # Safety
    ///
    /// TODO: Constraints unknown, use at own risk
    pub unsafe fn push(
        &mut self,
        frame: *mut spa_sys::spa_pod_frame,
        pod: *const spa_sys::spa_pod,
        offset: u32,
    ) {
        spa_sys::spa_pod_parser_push(self.as_raw_ptr(), frame, pod, offset)
    }

    pub fn current(&mut self) -> *mut spa_sys::spa_pod {
        unsafe { spa_sys::spa_pod_parser_current(self.as_raw_ptr()) }
    }

    /// # Safety
    ///
    /// Pod pointed to must we valid, well aligned, and contained in the current frame
    ///
    /// TODO: Any other constraints? Use at own risk
    pub unsafe fn advance(&mut self, pod: *const spa_sys::spa_pod) {
        spa_sys::spa_pod_parser_advance(self.as_raw_ptr(), pod)
    }

    /// # Safety
    ///
    /// TODO: Constraints unknown, use at own risk
    pub unsafe fn next(&mut self) -> *mut spa_sys::spa_pod {
        spa_sys::spa_pod_parser_next(self.as_raw_ptr())
    }

    /// # Safety
    ///
    /// Only the last added frame may be popped
    pub unsafe fn pop(&mut self, frame: &mut spa_sys::spa_pod_frame) -> Result<(), Errno> {
        let res = spa_sys::spa_pod_parser_pop(self.as_raw_ptr(), frame as *mut _);

        if res >= 0 {
            Ok(())
        } else {
            Err(Errno::from_i32(-res))
        }
    }

    pub fn get_bool(&mut self) -> Result<bool, Errno> {
        unsafe {
            let mut b: MaybeUninit<bool> = MaybeUninit::uninit();
            let res = spa_sys::spa_pod_parser_get_bool(self.as_raw_ptr(), b.as_mut_ptr());
            if res >= 0 {
                Ok(b.assume_init())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn get_id(&mut self) -> Result<Id, Errno> {
        unsafe {
            let mut id: MaybeUninit<u32> = MaybeUninit::uninit();
            let res = spa_sys::spa_pod_parser_get_id(self.as_raw_ptr(), id.as_mut_ptr());
            if res >= 0 {
                Ok(Id(id.assume_init()))
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn get_int(&mut self) -> Result<c_int, Errno> {
        unsafe {
            let mut int: MaybeUninit<c_int> = MaybeUninit::uninit();
            let res = spa_sys::spa_pod_parser_get_int(self.as_raw_ptr(), int.as_mut_ptr());
            if res >= 0 {
                Ok(int.assume_init())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn get_long(&mut self) -> Result<c_long, Errno> {
        unsafe {
            let mut long: MaybeUninit<c_long> = MaybeUninit::uninit();
            let res = spa_sys::spa_pod_parser_get_long(self.as_raw_ptr(), long.as_mut_ptr());
            if res >= 0 {
                Ok(long.assume_init())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn get_float(&mut self) -> Result<c_float, Errno> {
        unsafe {
            let mut float: MaybeUninit<c_float> = MaybeUninit::uninit();
            let res = spa_sys::spa_pod_parser_get_float(self.as_raw_ptr(), float.as_mut_ptr());
            if res >= 0 {
                Ok(float.assume_init())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn get_double(&mut self) -> Result<c_double, Errno> {
        unsafe {
            let mut double: MaybeUninit<c_double> = MaybeUninit::uninit();
            let res = spa_sys::spa_pod_parser_get_double(self.as_raw_ptr(), double.as_mut_ptr());
            if res >= 0 {
                Ok(double.assume_init())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn get_string_raw(&mut self) -> Result<&CStr, Errno> {
        unsafe {
            let mut string: MaybeUninit<*const c_char> = MaybeUninit::uninit();
            let res = spa_sys::spa_pod_parser_get_string(self.as_raw_ptr(), string.as_mut_ptr());
            if res >= 0 {
                let string = string.assume_init();
                // FIXME: Do we need to check string for null?
                let string = CStr::from_ptr(string);
                Ok(string)
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn get_bytes(&mut self) -> Result<&[u8], Errno> {
        unsafe {
            let mut bytes: MaybeUninit<*const u8> = MaybeUninit::uninit();
            let mut len: MaybeUninit<u32> = MaybeUninit::uninit();
            let res = spa_sys::spa_pod_parser_get_bytes(
                self.as_raw_ptr(),
                bytes.as_mut_ptr().cast(),
                len.as_mut_ptr(),
            );
            if res >= 0 {
                let bytes = bytes.assume_init();
                let len = len.assume_init();
                // TODO: Do we need to check bytes for null?
                let bytes = std::slice::from_raw_parts(bytes, len.try_into().unwrap());
                Ok(bytes)
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn get_pointer(&mut self) -> Result<(*const c_void, Id), Errno> {
        unsafe {
            let mut ptr: MaybeUninit<*const c_void> = MaybeUninit::uninit();
            let mut type_: MaybeUninit<u32> = MaybeUninit::uninit();
            let res = spa_sys::spa_pod_parser_get_pointer(
                self.as_raw_ptr(),
                type_.as_mut_ptr(),
                ptr.as_mut_ptr(),
            );
            if res >= 0 {
                Ok((ptr.assume_init(), Id(type_.assume_init())))
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn get_fd(&mut self) -> Result<i64, Errno> {
        unsafe {
            let mut fd: MaybeUninit<i64> = MaybeUninit::uninit();
            let res = spa_sys::spa_pod_parser_get_fd(self.as_raw_ptr(), fd.as_mut_ptr());
            if res >= 0 {
                Ok(fd.assume_init())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn get_rectangle(&mut self) -> Result<Rectangle, Errno> {
        unsafe {
            let mut rect: MaybeUninit<spa_sys::spa_rectangle> = MaybeUninit::uninit();
            let res = spa_sys::spa_pod_parser_get_rectangle(self.as_raw_ptr(), rect.as_mut_ptr());
            if res >= 0 {
                Ok(rect.assume_init())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn get_fraction(&mut self) -> Result<Fraction, Errno> {
        unsafe {
            let mut frac: MaybeUninit<spa_sys::spa_fraction> = MaybeUninit::uninit();
            let res = spa_sys::spa_pod_parser_get_fraction(self.as_raw_ptr(), frac.as_mut_ptr());
            if res >= 0 {
                Ok(frac.assume_init())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    /// # Safety
    ///
    /// TOOD: Constraints unknown, use at own risk
    pub unsafe fn get_pod(&mut self) -> Result<*mut spa_sys::spa_pod, Errno> {
        let mut pod: MaybeUninit<*mut spa_sys::spa_pod> = MaybeUninit::uninit();
        let res = spa_sys::spa_pod_parser_get_pod(self.as_raw_ptr(), pod.as_mut_ptr());
        if res >= 0 {
            Ok(pod.assume_init())
        } else {
            Err(Errno::from_i32(-res))
        }
    }

    /// # Safety
    /// The provided frame must not be moved or destroyed before it is popped again.
    ///
    /// The frame may only be assumed as initialized if this method returns `Ok`.
    pub unsafe fn push_struct(
        &mut self,
        frame: &mut MaybeUninit<spa_sys::spa_pod_frame>,
    ) -> Result<(), Errno> {
        let res = spa_sys::spa_pod_parser_push_struct(self.as_raw_ptr(), frame.as_mut_ptr());

        if res >= 0 {
            Ok(())
        } else {
            Err(Errno::from_i32(-res))
        }
    }

    /// # Safety
    /// The provided frame must not be moved or destroyed before it is popped again.
    ///
    /// The frame may only be assumed as initialized if this method returns `Ok`.
    pub unsafe fn push_object(
        &mut self,
        frame: &mut MaybeUninit<spa_sys::spa_pod_frame>,
        _type: u32,
    ) -> Result<Id, Errno> {
        let mut id: MaybeUninit<u32> = MaybeUninit::uninit();
        let res = spa_sys::spa_pod_parser_push_object(
            self.as_raw_ptr(),
            frame.as_mut_ptr(),
            _type,
            id.as_mut_ptr(),
        );

        if res >= 0 {
            Ok(Id(id.assume_init()))
        } else {
            Err(Errno::from_i32(-res))
        }
    }
}
