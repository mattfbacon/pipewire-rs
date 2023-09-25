// Copyright The pipewire-rs Contributors.
// SPDX-License-Identifier: MIT

use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ptr;
use std::rc::{Rc, Weak};

use crate::{error::Error, Properties};
use crate::{AsLoop, LoopRef};
use spa::ReadableDict;

/// A wrapper around the pipewire threaded loop interface. ThreadLoops are a higher level
/// of abstraction around the loop interface. A ThreadLoop can be used to spawn a new thread
/// that runs the wrapped loop.
#[derive(Debug, Clone)]
pub struct ThreadLoop {
    inner: Rc<ThreadLoopInner>,
}

impl ThreadLoop {
    /// Initialize Pipewire and create a new `ThreadLoop` with the given `name`
    ///
    /// # Safety
    pub unsafe fn new(name: Option<&str>) -> Result<Self, Error> {
        super::init();
        let inner = ThreadLoopInner::new::<Properties>(name, None)?;
        Ok(Self {
            inner: Rc::new(inner),
        })
    }

    /// Create a new ThreadLoop with the given `name` and `properties`.
    pub fn with_properties<T: ReadableDict>(
        name: Option<&str>,
        properties: &T,
    ) -> Result<Self, Error> {
        let inner = ThreadLoopInner::new(name, Some(properties))?;
        Ok(Self {
            inner: Rc::new(inner),
        })
    }

    pub fn downgrade(&self) -> WeakThreadLoop {
        let weak = Rc::downgrade(&self.inner);
        WeakThreadLoop { weak }
    }
}

impl AsLoop for ThreadLoop {
    type Target = ThreadLoopInner;

    fn as_loop(&self) -> &Rc<Self::Target> {
        &self.inner
    }
}

impl std::convert::AsRef<LoopRef> for ThreadLoop {
    fn as_ref(&self) -> &LoopRef {
        self.deref()
    }
}

impl Deref for ThreadLoop {
    type Target = ThreadLoopInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct WeakThreadLoop {
    weak: Weak<ThreadLoopInner>,
}

impl WeakThreadLoop {
    pub fn upgrade(&self) -> Option<ThreadLoop> {
        self.weak.upgrade().map(|inner| ThreadLoop { inner })
    }
}

pub struct ThreadLoopLockGuard<'a> {
    thread_loop: &'a ThreadLoopInner,
}

impl<'a> ThreadLoopLockGuard<'a> {
    fn new(thread_loop: &'a ThreadLoopInner) -> Self {
        unsafe {
            pw_sys::pw_thread_loop_lock(thread_loop.as_ptr());
        }
        ThreadLoopLockGuard { thread_loop }
    }

    /// Unlock the loop
    ///
    /// Unlocking the loop will call `drop()`
    pub fn unlock(self) {
        drop(self);
    }
}

impl<'a> Drop for ThreadLoopLockGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            pw_sys::pw_thread_loop_unlock(self.thread_loop.as_ptr());
        }
    }
}

#[derive(Debug)]
pub struct ThreadLoopInner {
    ptr: ptr::NonNull<pw_sys::pw_thread_loop>,
}

impl ThreadLoopInner {
    fn new<T: ReadableDict>(name: Option<&str>, properties: Option<&T>) -> Result<Self, Error> {
        unsafe {
            let props = properties.map_or(ptr::null(), |props| props.get_dict_ptr()) as *mut _;
            let l = pw_sys::pw_thread_loop_new(
                name.map_or(ptr::null(), |p| p.as_ptr() as *const _),
                props,
            );
            let ptr = ptr::NonNull::new(l).ok_or(Error::CreationFailed)?;

            Ok(ThreadLoopInner { ptr })
        }
    }

    fn as_ptr(&self) -> *mut pw_sys::pw_thread_loop {
        self.ptr.as_ptr()
    }

    /// Lock the Loop
    ///
    /// This ensures that the loop thread will not access objects associated
    /// with the loop while the lock is held, `lock()` can be used multiple times
    /// from the same thread.
    ///
    /// The lock needs to be held whenever you call any PipeWire function that
    /// uses an object associated with this loop. Make sure to not hold
    /// on to the lock more than necessary though, as the threaded loop stops
    /// while the lock is held.
    pub fn lock(&self) -> ThreadLoopLockGuard {
        ThreadLoopLockGuard::new(self)
    }

    /// Start the ThreadLoop
    pub fn start(&self) {
        unsafe {
            pw_sys::pw_thread_loop_start(self.as_ptr());
        }
    }

    /// Stop the ThreadLoop
    ///
    /// Stopping the ThreadLoop must be called without the lock
    pub fn stop(&self) {
        unsafe {
            pw_sys::pw_thread_loop_stop(self.as_ptr());
        }
    }

    /// Signal all threads waiting with [`wait()`](`Self::wait`)
    pub fn signal(&self, signal: bool) {
        unsafe {
            pw_sys::pw_thread_loop_signal(self.as_ptr(), signal);
        }
    }

    /// Release the lock and wait
    ///
    /// Release the lock and wait until some thread calls [`signal()`](`Self::signal`)
    pub fn wait(&self) {
        unsafe {
            pw_sys::pw_thread_loop_wait(self.as_ptr());
        }
    }

    /// Release the lock and wait a maximum of `wait_max_sec` seconds
    /// until some thread calls [`signal()`](`Self::signal`) or time out
    pub fn timed_wait(&self, wait_max_sec: std::time::Duration) {
        unsafe {
            let wait_max_sec: i32 = wait_max_sec
                .as_secs()
                .try_into()
                .expect("Provided timeout does not fit in a i32");
            pw_sys::pw_thread_loop_timed_wait(self.as_ptr(), wait_max_sec);
        }
    }

    /// Get a timespec suitable for [`timed_wait_full()`](`Self::timed_wait_full`)
    pub fn get_time(&self, timeout: i64) -> nix::sys::time::TimeSpec {
        unsafe {
            let mut abstime: MaybeUninit<pw_sys::timespec> = std::mem::MaybeUninit::uninit();
            pw_sys::pw_thread_loop_get_time(self.as_ptr(), abstime.as_mut_ptr(), timeout);
            let abstime = abstime.assume_init();
            nix::sys::time::TimeSpec::new(abstime.tv_sec, abstime.tv_nsec)
        }
    }

    /// Release the lock and wait up to abs seconds until some
    /// thread calls [`signal()`](`Self::signal`). Use [`get_time()`](`Self::get_time`)
    /// to get a suitable timespec
    pub fn timed_wait_full(&self, abstime: nix::sys::time::TimeSpec) {
        unsafe {
            let abstime = pw_sys::timespec {
                tv_sec: abstime.tv_sec(),
                tv_nsec: abstime.tv_nsec(),
            };
            pw_sys::pw_thread_loop_timed_wait_full(
                self.as_ptr(),
                &abstime as *const pw_sys::timespec,
            );
        }
    }

    /// Signal all threads executing [`signal()`](`Self::signal`) with `wait_for_accept`
    pub fn accept(&self) {
        unsafe {
            pw_sys::pw_thread_loop_accept(self.as_ptr());
        }
    }

    /// Check if inside the thread
    pub fn in_thread(&self) {
        unsafe {
            pw_sys::pw_thread_loop_in_thread(self.as_ptr());
        }
    }
}

impl std::convert::AsRef<LoopRef> for ThreadLoopInner {
    fn as_ref(&self) -> &LoopRef {
        self.deref()
    }
}

impl std::ops::Deref for ThreadLoopInner {
    type Target = LoopRef;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(pw_sys::pw_thread_loop_get_loop(self.ptr.as_ptr()) as *mut LoopRef) }
    }
}

impl Drop for ThreadLoopInner {
    fn drop(&mut self) {
        unsafe { pw_sys::pw_thread_loop_destroy(self.ptr.as_ptr()) }
    }
}
