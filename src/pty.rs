/*
 * Linux-only implementation of running a program with its stdout/stderr connected
 * to a pseudo-terminal (PTY) multiplexor device.
 */
extern crate libc;
use libc::{open, close, dup2, O_RDWR, O_CLOEXEC, O_NONBLOCK};
use super::helpers::Errno;
use crate::eprintsln;
use core::mem;
use core::ffi::c_str::CStr;

pub trait OutputReceiver {
    // It is normal for get_buf to return less then requested,
    // but should provide at least some space.
    fn get_buf(&mut self, n: usize) -> &mut [u8];
    fn put_buf(&mut self, n: usize);
    // Called when the child process closes  its stdout
    fn eof(&mut self) {}
}

fn get_pipes() -> [libc::c_int; 2] {
    let mut fds = [ -1 as libc::c_int, -1 ];
    unsafe {
        fds[0] = libc::posix_openpt(O_NONBLOCK | O_RDWR);
        if fds[0] == -1 {
            eprintsln!("openpt failure: {}", Errno::current());
            fds[0] = open(b"/dev/ptmx".as_ptr() as *const libc::c_char,
                          O_NONBLOCK | O_RDWR | O_CLOEXEC);
        }
        if fds[0] != -1 {
            libc::grantpt(fds[0]);
            libc::unlockpt(fds[0]);
            let ptsname = libc::ptsname(fds[0]);
            fds[1] = if ptsname.is_null() { -1 } else { open(ptsname, O_RDWR) };
            if fds[1] == -1 {
                eprintsln!("open PTY slave failure: ", Errno::current());
                close(fds[0]);
                fds[0] = -1;
            } else {
                // Copy terminal window size into the pty so that the child
                // formats its output correspondingly.
                let mut ws : libc::winsize = mem::MaybeUninit::zeroed().assume_init();
                if libc::ioctl(1, libc::TIOCGWINSZ, &raw mut ws) == 0 {
                    if libc::ioctl(fds[1], libc::TIOCSWINSZ, &raw const ws) != 0 {
                        eprintsln!("PTY setup: ", Errno::current());
                    }
                }
            }
        }
        if fds[0] == -1 {
            if libc::pipe((&mut fds[..]).as_mut_ptr()) == -1 {
                eprintsln!("pipe failure: ", Errno::current());
                fds = [-1, -1];
            } else {
                libc::fcntl(fds[0], libc::F_SETFL, O_NONBLOCK);
            }
        }
    }
    fds
}

fn exec_command(args: *const *const libc::c_char, out_fd: libc::c_int) -> ! {
    unsafe {
        if libc::setsid() == -1 {
            eprintsln!("failure to allocate new session: ", Errno::current());
        }
        dup2(out_fd, 1); // tty stdout
        dup2(out_fd, 2); // tty stderr
        close(out_fd);
        libc::execvp(*args, args);
        eprintsln!(CStr::from_ptr(*args), ": exec failure: ", Errno::current());
        libc::_exit(2)
    }
}

pub struct Context<'a> {
    pub pid: libc::pid_t,
    fd: libc::c_int,
    out: &'a mut dyn OutputReceiver
}

impl<'a> Context<'a> {

    pub fn create(command: *const *const libc::c_char, out: &'a mut dyn OutputReceiver) ->
        Result<Context<'a>,Errno>
    {
        let fds = get_pipes();
        if fds[0] == -1 { return Err(Errno(libc::ENOTTY)); }
        match unsafe { libc::fork() } {
            -1 => {
                let errno = Errno::current();
                eprintsln!("Subprocess spawning failure: ", errno);
                return Err(errno)
            },
            0 => { // child process
                unsafe { close(fds[0]); } // master
                exec_command(command, fds[1])
            },
            pid => {
                unsafe { close(fds[1]); } // tty
                Ok(Context { pid, fd: fds[0], out })
            }
        }
    }

    pub fn read_until_exit(&mut self) -> libc::c_int {
        loop {
            let buf = self.out.get_buf(libc::BUFSIZ as usize);
            let rd = unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if rd == 0 { break; }
            if rd > 0 {
                self.out.put_buf(rd as usize);
            } else if rd < 0 {
                let errno = Errno::current();
                if errno.0 == libc::EAGAIN {
                    let mut fds = libc::pollfd {
                        fd: self.fd,
                        events: libc::POLLIN | libc::POLLPRI,
                        revents: 0
                    };
                    unsafe { libc::poll(&raw mut fds, 1, -1) };
                    continue;
                } else {
                    if errno.0 != libc::EIO {
                        eprintsln!("read failure: ", errno);
                    }
                    break;
                }
            }
        }
        self.out.eof();
        loop {
            let mut status : libc::c_int = 0;
            let ret = unsafe { libc::waitpid(self.pid, &mut status, 0) };
            if ret > 0 { return status; }
            if ret == -1 {
                let errno = Errno::current();
                if errno.0 == libc::EINTR { continue; }
                break;
            }
        }
        -1
    }

}

