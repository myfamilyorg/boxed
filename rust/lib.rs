#![no_std]
#![feature(coerce_unsized)]
#![feature(unsize)]

extern crate error;
extern crate ffi;
extern crate ptr;
extern crate result;

use core::marker::Unsize;
use core::ops::CoerceUnsized;
use core::ptr::{null_mut, write};
use error::prelude::*;
use ptr::Ptr;
use result::Result;

errors!(Alloc);

pub struct Box<T: ?Sized> {
    ptr: Ptr<T>,
}

impl<T, U> CoerceUnsized<Box<U>> for Box<T>
where
    T: Unsize<U> + ?Sized,
    U: ?Sized,
{
}

impl<T> Box<T> {
    pub fn new(t: T) -> Result<Self> {
        let size = size_of::<T>();
        let ptr = if size == 0 {
            let mut ptr: Ptr<T> = Ptr::new(null_mut());
            ptr.set_bit(true);
            ptr
        } else {
            let mut ptr = unsafe {
                let rptr = ffi::alloc(size) as *mut T;
                if rptr.is_null() {
                    return err!(Alloc);
                }
                write(rptr, t);
                Ptr::new(rptr)
            };
            ptr.set_bit(false);
            ptr
        };
        Ok(Box { ptr })
    }
}

impl<T: ?Sized> Box<T> {
    pub unsafe fn leak(&mut self) {
        self.ptr.set_bit(true);
    }

    pub unsafe fn unleak(&mut self) {
        self.ptr.set_bit(false);
    }

    pub unsafe fn from_raw(ptr: Ptr<T>) -> Box<T> {
        Box { ptr }
    }

    pub unsafe fn into_raw(mut self) -> Ptr<T> {
        self.leak();
        Ptr::new(self.ptr.raw())
    }
}
