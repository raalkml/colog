#![allow(unused)]
use core::mem;
use core::ffi::c_str::CStr;
extern crate libc;

pub const fn count_of<T, const N: usize>(_: &[T; N]) -> usize { N }

//
// Bad replacement for Vec
//
pub struct BadVec<T, const N: usize> {
    storage: *mut T,
    count: usize,
    size: usize,
}

impl<T, const N: usize> BadVec<T, N> {
    pub fn new() -> Self {
        Self {
            storage: unsafe { libc::malloc(mem::size_of::<T>() * N) as *mut T },
            count: 0,
            size: N,
        }
    }
    fn realloc(&mut self) {
        self.size += N;
        self.storage = unsafe {
            libc::realloc(self.storage as *mut libc::c_void,
                          mem::size_of::<T>() * self.size) as *mut T
        };
    }
    pub fn push(&mut self, e: T) {
        assert!(self.count < N);
        if self.count == self.size { self.realloc() }
        unsafe { *self.storage.add(self.count) = e; }
        self.count += 1;
    }
    pub const fn len(&self) -> usize { self.count }
    pub const fn size(&self) -> usize { N }
    pub fn as_ptr(&self) -> *const T { self.storage }
    pub fn iter(&self) -> BadVecIter<'_, T, N> {
        BadVecIter { inner: self, off: 0 }
    }
}

impl<T, const N: usize> Drop for BadVec<T, N> {
    fn drop(&mut self) {
        unsafe { libc::free(self.storage as *mut libc::c_void); }
    }
}

impl<T, const N: usize> core::ops::Index<usize> for BadVec<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.storage.add(index) }
    }
}

#[derive(Clone)]
pub struct BadVecIter<'a, T, const N: usize> {
    inner: &'a BadVec<T, N>,
    off: usize,
}
impl<'a, T: Clone + Copy, const N: usize> Iterator for BadVecIter<'a, T, N> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.off < self.inner.len() {
            let off = self.off;
            self.off += 1;
            Some(unsafe { *self.inner.storage.add(off) })
        } else {
            None
        }
    }
}

pub fn strsignal(sig: libc::c_int) -> &'static CStr {
    // SAFETY: Returns a non-NULL pointer to a static string in libc
    unsafe { CStr::from_ptr(libc::strsignal(sig)) }
}

pub fn signal(sig: libc::c_int, f: extern "C" fn(i32)) -> libc::c_int {
    let mut empty_sigset = mem::MaybeUninit::<libc::sigset_t>::zeroed();
    unsafe { libc::sigemptyset(empty_sigset.as_mut_ptr()) };
    let act = libc::sigaction {
        sa_sigaction: f as libc::sighandler_t,
        sa_mask: unsafe { *empty_sigset.as_mut_ptr() },
        sa_flags: 0,
        sa_restorer: None
    };
    unsafe { libc::sigaction(sig, &raw const act, core::ptr::null_mut()) }
}

#[derive(Copy,Clone)]
pub struct Errno(pub libc::c_int);

impl Errno {
    pub fn current() -> Self {
        // SAFETY: libc guarantees existence of errno.
        Errno(unsafe { *libc::__errno_location() })
    }

    pub fn cstrerror(&self) -> &'static CStr {
        unsafe {
            let p = libc::strerror(self.0);
            CStr::from_ptr(p as *const i8)
        }
    }

    pub fn strerror(&self) -> &'static str {
        let cs = self.cstrerror();
        if let Ok(s) = cs.to_str() { s } else { &"Invalid encoding" }
    }
}

impl crate::fdprint::FdPuts for Errno {
    fn fdputs(self, fd: libc::c_int) {
        let err = unsafe { libc::strerror(self.0) };
        crate::fdprint::fdput_c_str(fd, err)
    }
}

