/*
 * Linux-only implementation of running a program with its stdout/stderr connected
 * to a pseudo-terminal (PTY) multiplexor device.
 */
extern crate libc;
use libc::{open, close, dup2, O_RDWR, O_CLOEXEC, O_NONBLOCK};
use super::helpers::Errno;

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
            eprintln!("posix_openpt failure: {}", Errno::current());
            fds[0] = open(b"/dev/ptmx".as_ptr() as *const libc::c_char,
                          O_NONBLOCK | O_RDWR | O_CLOEXEC);
        }
        if fds[0] != -1 {
            libc::grantpt(fds[0]);
            libc::unlockpt(fds[0]);
            let ptsname = libc::ptsname(fds[0]);
            fds[1] = if ptsname.is_null() { -1 } else { open(ptsname, O_RDWR) };
            if fds[1] == -1 {
                eprintln!("open PTY slave failure: {}", Errno::current());
                close(fds[0]);
                fds[0] = -1;
            } else {
                // Copy terminal window size into the pty so that the child
                // formats its output correspondingly.
                let mut ws : libc::winsize = std::mem::MaybeUninit::zeroed().assume_init();
                if libc::ioctl(1, libc::TIOCGWINSZ, &raw mut ws) == 0 {
                    if libc::ioctl(fds[1], libc::TIOCSWINSZ, &raw const ws) != 0 {
                        eprintln!("PTY setup: {}", Errno::current());
                    }
                }
            }
        }
        if fds[0] == -1 {
            if libc::pipe((&mut fds[..]).as_mut_ptr()) == -1 {
                eprintln!("pipe failure: {}", Errno::current());
                fds = [-1, -1];
            } else {
                libc::fcntl(fds[0], libc::F_SETFL, O_NONBLOCK);
            }
        }
    }
    fds
}

fn exec_command(command: Vec<String>, out_fd: libc::c_int) -> ! {
    let command: Vec<std::ffi::CString> = command
        .iter()
        .map(|v| std::ffi::CString::new(v.as_str()).unwrap())
        .collect();
    let mut args = Vec::new();
    for a in command.iter() {
        args.push(a.as_ptr());
    }
    args.push(std::ptr::null());
    unsafe {
        if libc::setsid() == -1 {
            eprintln!("failure to allocate new session: {}",
                      Errno::current());
        }
        dup2(out_fd, 1); // tty stdout
        dup2(out_fd, 2); // tty stderr
        close(out_fd);
        libc::execvp(args[0] as *const libc::c_char,
                     args.as_ptr() as *const *const libc::c_char);
        eprintln!("{:?}: exec failure: {}",
                  std::ffi::CStr::from_ptr(args[0] as *const i8),
                  Errno::current());
        libc::_exit(2)
    }
}

fn process_command(pid: libc::pid_t, fd: libc::c_int, out: &mut impl OutputReceiver) -> libc::c_int {
    loop {
        let buf = out.get_buf(libc::BUFSIZ as usize);
        let rd = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if rd == 0 { break; }
        if rd > 0 {
            out.put_buf(rd as usize);
        } else if rd < 0 {
            let errno = Errno::current();
            if errno.value == libc::EAGAIN {
                let mut fds = libc::pollfd {
                    fd,
                    events: libc::POLLIN | libc::POLLPRI,
                    revents: 0
                };
                unsafe { libc::poll(&raw mut fds, 1, -1) };
                continue;
            } else {
                if errno.value != libc::EIO {
                    eprintln!("read failure: {}", errno);
                }
                break;
            }
        }
    }
    out.eof();
    loop {
        let mut status : libc::c_int = 0;
        let ret = unsafe { libc::waitpid(pid, &mut status, 0) };
        if ret > 0 { return status; }
        if ret == -1 {
            let errno = Errno::current();
            if errno.value == libc::EINTR { continue; }
            break;
        }
    }
    -1
}

pub fn ptyrun(command: Vec<String>, out: &mut impl OutputReceiver) -> libc::c_int {
    let fds = get_pipes();
    if fds[0] == -1 { return -1; }
    match unsafe { libc::fork() } {
        -1 => {
            eprintln!("Subprocess spawning failure: {}", Errno::current());
            return -1
        },
        0 => { // child process
            unsafe { close(fds[0]); } // master
            exec_command(command, fds[1])
        },
        pid => {
            unsafe { close(fds[1]); } // tty
            process_command(pid, fds[0], out)
        }
    }
}

