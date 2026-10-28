extern crate libc;
use crate::helpers::Errno;
use crate::eprintsln;
use core::ptr;
use core::ffi;

pub struct ExternalFilter<'a> {
    filter: &'a ffi::CStr,
    pub pid: libc::pid_t,
    fd: libc::c_int,
}

impl<'a> ExternalFilter<'a> {
    pub fn new(filter: &'a ffi::CStr) -> Self {
        Self { filter, pid: 0, fd: -1 }
    }
    pub fn start(&mut self) -> Result<(), Errno> {
        let mut fd = [ -1 as libc::c_int, -1 ];
        if unsafe { libc::pipe((&mut fd[..]).as_mut_ptr()) } == -1 {
            let errno = Errno::current();
            eprintsln!("pipe failure: ", &errno);
            return Err(errno);
        }
        match unsafe { libc::fork() } {
            -1 => {
                let errno = Errno::current();
                eprintsln!("External filter spawning failure: ", &errno);
                Err(errno)
            },
            0  => {
                let args = [
                    c"sh".as_ptr(),
                    c"-c".as_ptr(),
                    self.filter.as_ptr(),
                    ptr::null()
                ];
                unsafe {
                    libc::close(fd[1]);
                    libc::dup2(fd[0], 0);
                    libc::execvp(args[0] as *const libc::c_char,
                                 args.as_ptr() as *const *const libc::c_char);
                    eprintsln!("sh -c ", self.filter, ": exec failure: ", Errno::current());
                    libc::_exit(254);
                }
            },
            pid => {
                unsafe { libc::close(fd[0]); }
                self.fd = fd[1];
                self.pid = pid;
                Ok(())
            }
        }
    }

    pub fn filter(&self, buf: &[u8]) -> bool {
        let _ = unsafe { libc::write(self.fd, buf.as_ptr() as *const libc::c_void, buf.len()) };
        false
    }
}

impl<'a> Drop for ExternalFilter<'a> {
    fn drop(&mut self) {
        let mut status : libc::c_int = 0;
        unsafe { libc::waitpid(self.pid, &mut status, 0); }
    }
}
