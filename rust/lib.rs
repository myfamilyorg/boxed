#![no_std]
#![feature(coerce_unsized)]
#![feature(unsize)]

extern crate error;
extern crate ffi;
extern crate ptr;
extern crate raw;
extern crate result;
extern crate try_clone;

use core::marker::Unsize;
use core::mem::size_of;
use core::ops::{CoerceUnsized, Deref, DerefMut};
use core::ptr::{drop_in_place, null_mut, write};
use error::prelude::*;
use ptr::Ptr;
use raw::{AsRaw, AsRawMut};
use result::Result;
use try_clone::TryClone;

errors!(
    // A memory allocation error occurred
    Alloc
);

pub struct Box<T: ?Sized> {
    ptr: Ptr<T>,
}

impl<T, U> CoerceUnsized<Box<U>> for Box<T>
where
    T: Unsize<U> + ?Sized,
    U: ?Sized,
{
}

impl<T: ?Sized + Clone> TryClone for Box<T> {
    fn try_clone(&self) -> Result<Self> {
        Box::new((*(self.ptr)).clone())
    }
}

impl<T: ?Sized> Drop for Box<T> {
    fn drop(&mut self) {
        if !self.ptr.get_bit() {
            let value_ptr = self.ptr.as_mut_ptr();
            if !value_ptr.is_null() {
                unsafe {
                    drop_in_place(value_ptr);
                    ffi::release(value_ptr as *const u8);
                }
                self.ptr.set_bit(true);
            }
        }
    }
}

impl<T: ?Sized> AsRaw<T> for Box<T>
where
    Self: Sized,
{
    fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr() as *const T
    }
}

impl<T: ?Sized> AsRawMut<T> for Box<T>
where
    Self: Sized,
{
    fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr() as *mut T
    }
}

impl<T> Deref for Box<T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr.as_ptr() }
    }
}

impl<T> DerefMut for Box<T>
where
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr.as_mut_ptr() }
    }
}

impl<T> AsRef<T> for Box<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.ptr.as_ptr() }
    }
}

impl<T> AsMut<T> for Box<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr.as_mut_ptr() }
    }
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
        Ptr::new(self.ptr.as_ptr())
    }
}
