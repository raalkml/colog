
pub trait Sink {
    fn sink(&mut self, fd: libc::c_int, buf: &[u8]) -> bool;
}

pub mod regexcolors;
pub mod externalfilter;
pub mod stdout;
