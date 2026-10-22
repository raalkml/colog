extern crate libc;

pub const fn count_of<T, const N: usize>(_: &[T; N]) -> usize { N }

#[allow(unused)]
pub fn strsignal(sig: libc::c_int) -> &'static std::ffi::CStr {
    // SAFETY: Returns a non-NULL pointer to a static string in libc
    unsafe { std::ffi::CStr::from_ptr(libc::strsignal(sig)) }
}

pub fn signal(sig: libc::c_int, f: extern "C" fn(i32)) -> libc::c_int {
    let mut empty_sigset = std::mem::MaybeUninit::<libc::sigset_t>::zeroed();
    unsafe { libc::sigemptyset(empty_sigset.as_mut_ptr()) };
    let act = libc::sigaction {
        sa_sigaction: f as libc::sighandler_t,
        sa_mask: unsafe { *empty_sigset.as_mut_ptr() },
        sa_flags: 0,
        sa_restorer: None
    };
    unsafe { libc::sigaction(sig, &raw const act, std::ptr::null_mut()) }
}

pub struct Errno {
    pub value: libc::c_int
}

impl Errno {
    pub fn current() -> Self {
        // SAFETY: libc guarantees existence of errno.
        Errno { value: unsafe { *libc::__errno_location() } }
    }
    pub fn cstrerror(&self) -> &'static std::ffi::CStr {
        unsafe {
            let p = libc::strerror(self.value);
            std::ffi::CStr::from_ptr(p as *const i8)
        }
    }
    pub fn strerror(&self) -> &'static str {
        let cs = self.cstrerror();
        if let Ok(s) = cs.to_str() { s } else { &"Invalid encoding" }
    }
}

use std::fmt;

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.strerror(), self.value)
    }
}
