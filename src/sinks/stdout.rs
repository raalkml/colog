extern crate libc;
pub struct StdoutSink {}

impl super::Sink for StdoutSink {
    fn sink(&mut self, fd: libc::c_int, buf: &[u8]) -> bool {
        unsafe { libc::write(fd, buf.as_ptr() as *const libc::c_void, buf.len()); }
        false
    }
}
