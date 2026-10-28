/*
 * A log colorizer.
 */

#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, allow(unused))]

use core::ffi;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

extern crate libc;
mod externalfilter;
mod regexcolors;
mod pty;
mod helpers;
use helpers::BadVec;
mod iobuf;
mod fdprint;

static COMMAND_PID : AtomicI32 = AtomicI32::new(0);
static EXTERNALFILTER_PID : AtomicI32 = AtomicI32::new(0);
static VERBOSE: AtomicBool = AtomicBool::new(false);

fn exit(code: i32) -> ! {
    unsafe { libc::exit(code); }
}

fn kill_subprocs(sig: libc::c_int) {
    unsafe {
        let pid = COMMAND_PID.load(Ordering::Relaxed);
        if pid != 0 {
            if VERBOSE.load(Ordering::Relaxed) {
                eprintsln!("command process [", pid, "] killed");
            }
            libc::kill(-pid, sig);
        }
        let pid = EXTERNALFILTER_PID.load(Ordering::Relaxed);
        if pid != 0 {
            if VERBOSE.load(Ordering::Relaxed) {
                eprintsln!("filter process [", pid, "] killed");
            }
            libc::kill(-pid, sig);
        }
    }
}

extern "C" fn interrupt_handler(sig: libc::c_int) {
    eprintsln!("\n", helpers::strsignal(sig));
    kill_subprocs(libc::SIGHUP);
}

extern "C" fn terminate_handler(sig: libc::c_int) {
    eprintsln!("\n", helpers::strsignal(sig));
    kill_subprocs(sig);
}

fn usage() {
    printsln!("colog [-v|--verbose -c <filter>] PROGRAM ARGS...

<filter> is a shell command which will receive all lines of the
PROGRAM output, which were not colorized by the built-in rules.");
}

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, argv: *const *const libc::c_char) -> i32 {
    let mut filter_command = None;
    let mut command = BadVec::<*const libc::c_char, 20>::new();
    {
        let mut filter_command_arg = false;
        let mut command_arg = false;
        for arg in 1 .. argc {
            let argz = unsafe { ffi::CStr::from_ptr(*argv.add(arg as usize)) };
            let arg = argz.to_str().expect("Valid UTF-8 in argument expected");
            if command_arg {
                command.push(argz.as_ptr());
            } else if filter_command_arg {
                filter_command_arg = false;
                filter_command = Some(argz);
            } else if arg == "--" {
                command_arg = true;
            } else if arg == "--help" {
                usage();
                return 0;
            } else if arg == "-v" || arg == "--verbose" {
                VERBOSE.store(true, Ordering::Relaxed);
            } else if arg.starts_with("-c") {
                if arg.len() > 2 {
                    filter_command = Some(unsafe {
                        ffi::CStr::from_ptr(argz.as_ptr().add(2))
                    });
                } else {
                    filter_command_arg = true;
                }
            } else {
                command.push(argz.as_ptr());
                command_arg = true;
            }
        }
    }
    if command.len() == 0 {
        usage();
        return 1;
    }
    command.push(ptr::null());
    if VERBOSE.load(Ordering::Relaxed) {
        eprints!("Running:");
        for a in command.iter() {
            if a.is_null() { break; }
            eprints!(" '", a, "'");
        }
        eprints!("\n");
        if let Some(ref filter) = filter_command {
            eprintsln!("Filter: \"", filter, "\"");
        }
    }
    helpers::signal(libc::SIGINT, interrupt_handler);
    helpers::signal(libc::SIGTERM, terminate_handler);
    helpers::signal(libc::SIGPIPE, terminate_handler);
    helpers::signal(libc::SIGHUP, terminate_handler);
    let mask = [ libc::SIGINT, libc::SIGTERM, libc::SIGPIPE, libc::SIGHUP ];

    let mut splitter = LineSplitter::<{ libc::BUFSIZ as usize }> {
        iob: iobuf::IoBuf::new(),
	markup: &mut regexcolors::RegexColors::new(),
        output: filter_command.map(|f| externalfilter::ExternalFilter::new(f)),
    };

    if let Some(ref mut filter) = splitter.output {
        helpers::block_signals(&mask);
        if let Err(errno) = filter.start() {
            eprintsln!("Starting filter command: ", errno);
            exit(254);
        }
        EXTERNALFILTER_PID.store(filter.pid, Ordering::Relaxed);
        helpers::unblock_signals(&mask);
    }
        helpers::block_signals(&mask);
    let ret = pty::Context::create(command.as_ptr(), &mut splitter);
    if let Ok(mut pty) = ret {
        COMMAND_PID.store(pty.pid, Ordering::Relaxed);
        helpers::unblock_signals(&mask);
        let status = pty.read_until_exit();
        if libc::WIFEXITED(status) {
            libc::WEXITSTATUS(status)
        } else if libc::WIFSIGNALED(status) {
            128 + libc::WTERMSIG(status)
        } else {
            libc::WEXITSTATUS(status)
        }
    } else {
        255
    }
}

struct LineSplitter<'a, const S: usize> {
    iob: iobuf::IoBuf<S>,
    markup: &'a mut regexcolors::RegexColors,
    output: Option<externalfilter::ExternalFilter<'a>>,
}

impl<'a, const S: usize> LineSplitter<'a, S> {
    fn write_out(&self, b: &[u8]) {
        if let Some(ref filter) = self.output {
            filter.filter(b);
        } else {
            fdprint::fdput_bytes(libc::STDOUT_FILENO, b);
        }
    }
}

impl<'a, const S: usize> pty::OutputReceiver for LineSplitter<'a, S> {
    fn get_buf(&mut self, n: usize) -> &mut [u8] {
        self.iob.get_buf(n)
    }
    fn put_buf(&mut self, n: usize) {
        self.iob.put_buf(n);
        loop {
            let ln = self.iob.peek_line();
            if ln.len() == 0 { break; }
            if !self.markup.markup(ln) {
                self.write_out(ln);
            }
            self.iob.consume(ln.len());
        }
    }
    fn eof(&mut self) {
        let b = self.iob.peek();
        if b.len() > 0 {
            if !self.markup.markup(b) {
                self.write_out(b);
            }
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler_fn(info: &core::panic::PanicInfo) -> ! {
    eprintsln!("=== Panic ===");
    if let Some(msg) = info.message().as_str() {
        eprintsln!(msg);
    }
    if let Some(loc) = info.location() {
        eprintsln!(loc.file().as_bytes(), ":", loc.line());
    }
    unsafe { libc::raise(libc::SIGKILL); }
    loop {}
}
