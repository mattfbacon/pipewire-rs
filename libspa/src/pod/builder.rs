use std::{
    ffi::{c_int, c_void, CString},
    mem::MaybeUninit,
};

use nix::errno::Errno;

use crate::utils::{Fraction, Id, Rectangle};

static CALLBACKS: spa_sys::spa_pod_builder_callbacks = spa_sys::spa_pod_builder_callbacks {
    version: spa_sys::SPA_VERSION_POD_BUILDER_CALLBACKS,
    overflow: Some(Builder::overflow),
};

struct BuilderInner<'d> {
    builder: spa_sys::spa_pod_builder,
    data: &'d mut Vec<u8>,
}

pub struct Builder<'d> {
    // Keep the actual state in a box, so that
    // we can be sure that it does not move while the builder is in use
    // This lets us access it via pointer in the overflow callback
    inner: Box<BuilderInner<'d>>,
}

impl<'d> Builder<'d> {
    unsafe extern "C" fn overflow(data: *mut c_void, size: u32) -> c_int {
        let this: *mut BuilderInner = data.cast();

        assert!(!this.is_null());
        assert!(size as usize > (*this).data.len());

        // Resize the vec to be `size` longer, so that the new value fits,
        // then update the builders internal data size and also the data pointer
        // in case the vec had to reallocate
        (*this).data.resize(size as usize, 0);
        (*this).builder.data = (*this).data.as_mut_ptr().cast::<c_void>();
        (*this).builder.size = (*this)
            .data
            .len()
            .try_into()
            .expect("data length does not fit in a u32");

        // Return zero to indicate that we successfully resized our data
        0
    }

    pub fn new(data: &'d mut Vec<u8>) -> Self {
        unsafe {
            let mut builder: MaybeUninit<spa_sys::spa_pod_builder> = MaybeUninit::uninit();

            spa_sys::spa_pod_builder_init(
                builder.as_mut_ptr(),
                data.as_mut_ptr().cast(),
                data.len()
                    .try_into()
                    .expect("data length does not fit in a u32"),
            );

            let inner = Box::new(BuilderInner {
                builder: builder.assume_init(),
                data,
            });

            spa_sys::spa_pod_builder_set_callbacks(
                std::ptr::addr_of!(inner.builder).cast_mut(),
                std::ptr::addr_of!(CALLBACKS),
                std::ptr::addr_of!(*inner).cast::<c_void>().cast_mut(),
            );

            Self { inner }
        }
    }

    pub fn as_raw(&self) -> &spa_sys::spa_pod_builder {
        &self.inner.builder
    }

    pub fn as_raw_ptr(&self) -> *mut spa_sys::spa_pod_builder {
        std::ptr::addr_of!(self.inner.builder).cast_mut()
    }

    /// # Safety
    ///
    /// The builder state may only be used as long as all frames that were pushed
    /// to the builder at the time of this call are alive and not moved
    pub unsafe fn state(&self) -> spa_sys::spa_pod_builder_state {
        let mut state: MaybeUninit<spa_sys::spa_pod_builder_state> = MaybeUninit::uninit();
        spa_sys::spa_pod_builder_get_state(self.as_raw_ptr(), state.as_mut_ptr());
        state.assume_init()
    }

    // not bound: set_callbacks
    // we set those ourselves to resize the Vec

    /// # Safety
    ///
    /// TODO: Constraints unknown, use at own risk
    pub unsafe fn reset(&mut self, state: *mut spa_sys::spa_pod_builder_state) {
        spa_sys::spa_pod_builder_reset(self.as_raw_ptr(), state)
    }

    /// # Safety
    ///
    /// TODO: Constraints unknown, use at own risk
    pub unsafe fn deref(&mut self, offset: u32) -> *mut spa_sys::spa_pod {
        spa_sys::spa_pod_builder_deref(self.as_raw_ptr(), offset)
    }

    /// # Safety
    ///
    /// TODO: Constraints unknown, use at own risk
    pub unsafe fn frame(&mut self, frame: *mut spa_sys::spa_pod_frame) -> *mut spa_sys::spa_pod {
        spa_sys::spa_pod_builder_frame(self.as_raw_ptr(), frame)
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
        spa_sys::spa_pod_builder_push(self.as_raw_ptr(), frame, pod, offset)
    }

    // TODO: raw, pad

    pub fn raw_padded(&mut self, data: &[u8]) -> Result<(), Errno> {
        let res = unsafe {
            spa_sys::spa_pod_builder_raw_padded(
                self.as_raw_ptr(),
                data.as_ptr().cast::<c_void>(),
                data.len().try_into().unwrap(),
            )
        };

        if res >= 0 {
            Ok(())
        } else {
            Err(Errno::from_i32(-res))
        }
    }

    /// # Safety
    ///
    /// Only the last added frame may be popped
    pub unsafe fn pop(&mut self, frame: &mut spa_sys::spa_pod_frame) {
        unsafe {
            spa_sys::spa_pod_builder_pop(self.as_raw_ptr(), frame as *mut _);
        }
    }

    // TODO: primitive

    pub fn add_none(&mut self) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_none(self.as_raw_ptr());

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    // todo: child

    pub fn add_bool(&mut self, val: bool) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_bool(self.as_raw_ptr(), val);

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn add_id(&mut self, val: Id) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_id(self.as_raw_ptr(), val.0);

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn add_int(&mut self, val: i32) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_int(self.as_raw_ptr(), val);

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn add_long(&mut self, val: i64) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_long(self.as_raw_ptr(), val);

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn add_float(&mut self, val: f32) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_float(self.as_raw_ptr(), val);

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn add_double(&mut self, val: f64) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_double(self.as_raw_ptr(), val);

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    // TODO: write_string, string_len,
    // TODO: add_string_raw variant?

    /// # Panics
    ///
    /// If `string` contains an interior null byte
    pub fn add_string(&mut self, string: &str) -> Result<(), Errno> {
        let c_str = CString::new(string).expect("string should not contain an interior null byte");

        let res = unsafe { spa_sys::spa_pod_builder_string(self.as_raw_ptr(), c_str.as_ptr()) };

        if res >= 0 {
            Ok(())
        } else {
            Err(Errno::from_i32(-res))
        }
    }

    // TODO: raw bytes variant?

    pub fn add_bytes(&mut self, bytes: &[u8]) -> Result<(), Errno> {
        let res = unsafe {
            spa_sys::spa_pod_builder_bytes(
                self.as_raw_ptr(),
                bytes.as_ptr().cast::<c_void>(),
                bytes.len().try_into().unwrap(),
            )
        };

        if res >= 0 {
            Ok(())
        } else {
            Err(Errno::from_i32(-res))
        }
    }

    // TODO: reserve_bytes

    /// # Safety
    ///
    /// The pointer must be pointing to valid, well-aligned data which has the type as specified by `type_`.
    pub unsafe fn add_pointer(&mut self, type_: Id, val: *const c_void) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_pointer(self.as_raw_ptr(), type_.0, val);

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn add_fd(&mut self, val: std::os::fd::RawFd) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_fd(self.as_raw_ptr(), val.into());

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn add_rectangle(&mut self, val: Rectangle) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_rectangle(self.as_raw_ptr(), val.width, val.height);

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn add_fraction(&mut self, val: Fraction) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_fraction(self.as_raw_ptr(), val.num, val.denom);

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    /// # Safety
    /// The provided frame must not be moved or destroyed before it is popped again.
    ///
    /// The frame may only be assumed as initialized if this method returns `Ok`.
    pub unsafe fn push_array(
        &mut self,
        frame: &mut MaybeUninit<spa_sys::spa_pod_frame>,
    ) -> Result<(), Errno> {
        let res = spa_sys::spa_pod_builder_push_array(self.as_raw_ptr(), frame.as_mut_ptr());

        if res >= 0 {
            Ok(())
        } else {
            Err(Errno::from_i32(-res))
        }
    }

    /// # Safety
    ///
    /// `elems` must point to a valid array containing at least `n_elems`
    /// with each child having exactly the size as specified by `child_size` and the type `child_type`.
    pub unsafe fn add_array(
        &mut self,
        child_size: u32,
        child_type: u32,
        n_elems: u32,
        elems: *const c_void,
    ) -> Result<(), Errno> {
        let res = spa_sys::spa_pod_builder_array(
            self.as_raw_ptr(),
            child_size,
            child_type,
            n_elems,
            elems,
        );

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
    pub unsafe fn push_choice(
        &mut self,
        frame: &mut MaybeUninit<spa_sys::spa_pod_frame>,
        type_: u32,
        flags: u32, // FIXME: Make dedicated flag type
    ) -> Result<(), Errno> {
        let res = spa_sys::spa_pod_builder_push_choice(
            self.as_raw_ptr(),
            frame.as_mut_ptr(),
            type_,
            flags,
        );

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
    pub unsafe fn push_struct(
        &mut self,
        frame: &mut MaybeUninit<spa_sys::spa_pod_frame>,
    ) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_push_struct(self.as_raw_ptr(), frame.as_mut_ptr());

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    /// # Safety
    /// The provided frame must not be moved or destroyed before it is popped again.
    ///
    /// The frame may only be assumed as initialized if this method returns `Ok`.
    pub unsafe fn push_object(
        &mut self,
        frame: &mut MaybeUninit<spa_sys::spa_pod_frame>,
        type_: u32,
        id: u32,
    ) -> Result<(), Errno> {
        unsafe {
            let res = spa_sys::spa_pod_builder_push_object(
                self.as_raw_ptr(),
                frame.as_mut_ptr(),
                type_,
                id,
            );

            if res >= 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(-res))
            }
        }
    }

    pub fn add_prop(&mut self, key: u32, flags: u32) -> Result<(), Errno> {
        let res = unsafe { spa_sys::spa_pod_builder_prop(self.as_raw_ptr(), key, flags) };

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
    pub unsafe fn push_sequence(
        &mut self,
        frame: &mut MaybeUninit<spa_sys::spa_pod_frame>,
        unit: u32,
    ) -> Result<(), Errno> {
        let res =
            spa_sys::spa_pod_builder_push_sequence(self.as_raw_ptr(), frame.as_mut_ptr(), unit);

        if res >= 0 {
            Ok(())
        } else {
            Err(Errno::from_i32(-res))
        }
    }

    // FIXME: For some reason the raw function returns a u32 instead of a c_int like all the others,
    // Change this when https://gitlab.freedesktop.org/pipewire/pipewire/-/merge_requests/1679 is merged and released
    pub fn add_control(&mut self, offset: u32, type_: u32) -> u32 {
        unsafe { spa_sys::spa_pod_builder_control(self.as_raw_ptr(), offset, type_) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn struct_pod() {
        let mut data = Vec::new();

        let mut builder = Builder::new(&mut data);
        unsafe {
            let mut struct_frame: MaybeUninit<spa_sys::spa_pod_frame> = MaybeUninit::uninit();
            println!("Pushing struct");
            builder.push_struct(&mut struct_frame).unwrap();
            println!("Pushing int");
            builder.add_int(3).unwrap();
            println!("Popping struct");
            builder.pop(struct_frame.assume_init_mut());
        }

        let other: Vec<u8> = [
            16u32.to_ne_bytes(), // body has size 16
            14u32.to_ne_bytes(), // struct type is 14
            4u32.to_ne_bytes(),  // child body size is 4
            4u32.to_ne_bytes(),  // Int child type is 4
            3i32.to_ne_bytes(),  // the integer
            [0, 0, 0, 0],        // padding
        ]
        .iter()
        .copied()
        .flatten()
        .collect();

        assert_eq!(&data, &other)
    }
}
