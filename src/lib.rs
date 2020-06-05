#![no_std]

use core::convert::Infallible;
use core::mem::MaybeUninit;
use cortex_m::interrupt::free as interrupt_free;

const BUF_SIZE: usize = 16;
pub struct RingBuf<T> {
    front: usize,
    back: usize,
    data: [MaybeUninit<T>; BUF_SIZE],
}

impl<T: Copy> RingBuf<T> {
    pub fn new() -> Self {
        interrupt_free(|_| Self {
            front: usize::default(),
            back: usize::default(),
            data: [MaybeUninit::uninit(); BUF_SIZE],
        })
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        BUF_SIZE
    }

    pub fn free(&self) -> usize {
        self.capacity() - self.len()
    }

    pub fn len(&self) -> usize {
        if self.front <= self.back {
            self.back - self.front
        } else {
            self.capacity() - self.front + self.back
        }
    }

    pub fn get(&self, i: usize) -> nb::Result<T, Infallible> {
        if self.len() <= i {
            return Err(nb::Error::WouldBlock);
        }

        Ok(unsafe { self.data[self.front + i].assume_init() })
    }

    pub fn push_back(&mut self, item: T) -> nb::Result<(), Infallible> {
        if self.is_full() {
            return Err(nb::Error::WouldBlock);
        }
        unsafe { self.data[self.back].as_mut_ptr().write(item) };
        if self.back == self.capacity() - 1 {
            self.back = 0;
        } else {
            self.back += 1;
        }
        Ok(())
    }

    pub fn push_front(&mut self, item: T) -> nb::Result<(), Infallible> {
        if self.is_full() {
            return Err(nb::Error::WouldBlock);
        }
        if self.front == 0 {
            self.front = self.capacity() - 1;
        } else {
            self.front -= 1;
        }
        unsafe { self.data[self.front].as_mut_ptr().write(item) };
        Ok(())
    }

    pub fn pop_back(&mut self) -> nb::Result<T, Infallible> {
        if self.is_empty() {
            return Err(nb::Error::WouldBlock);
        }

        if self.back == 0 {
            self.back = self.capacity() - 1;
        } else {
            self.back -= 1;
        }
        let d = self.data[self.back];

        Ok(unsafe { d.assume_init() })
    }

    pub fn pop_front(&mut self) -> nb::Result<T, Infallible> {
        if self.is_empty() {
            return Err(nb::Error::WouldBlock);
        }
        let d = self.data[self.front];
        if self.front == self.capacity() {
            self.front = 0;
        } else {
            self.front += 1;
        }
        Ok(unsafe { d.assume_init() })
    }
}

impl<T: Copy> core::ops::Index<usize> for RingBuf<T> {
    type Output = T;
    fn index(&self, i: usize) -> &Self::Output {
        if self.len() <= i {
            panic!("Out of bounds")
        }

        let d = unsafe { self.data[self.front + i].assume_init() };
        unsafe { &*((&d) as *const T) }
    }
}

impl<T: Copy> core::ops::IndexMut<usize> for RingBuf<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        if self.len() <= i {
            panic!("Out of bounds")
        }

        let mut d = unsafe { self.data[self.front + i].assume_init() };
        unsafe { &mut *(&mut d as *mut T) }
    }
}

impl<T: Copy> core::ops::Index<core::ops::Range<usize>> for RingBuf<T> {
    type Output = [T];

    fn index(&self, i: core::ops::Range<usize>) -> &Self::Output {
        if self.len() < i.end {
            panic!("Out of bounds")
        }
        let d = core::ptr::slice_from_raw_parts(
            &self.data[self.front + i.start] as *const _ as *const T,
            i.end - i.start,
        );
        unsafe { &*d }
    }
}

impl<T: Copy> core::ops::IndexMut<core::ops::Range<usize>> for RingBuf<T> {
    fn index_mut(&mut self, i: core::ops::Range<usize>) -> &mut Self::Output {
        if self.len() < i.end {
            panic!("Out of bounds")
        }
        let d = core::ptr::slice_from_raw_parts_mut(
            &mut self.data[self.front + i.start] as *mut _ as *mut T,
            i.end - i.start,
        );
        unsafe { &mut *d }
    }
}