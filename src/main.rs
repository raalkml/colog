/*
 * A log colorizer.
 */
extern crate libc;
mod externalfilter;
mod regexcolors;
mod pty;
mod helpers;
mod iobuf;

extern "C" fn interrupt_handler(sig: libc::c_int) {
    eprintln!("\n{:?}", helpers::strsignal(sig));
    std::process::exit(1);
}

extern "C" fn terminate_handler(sig: libc::c_int) {
    eprintln!("\n{:?}", helpers::strsignal(sig));
    std::process::exit(1);
}

fn usage() {
    println!("colog [-v|--verbose -c <filter>] PROGRAM ARGS...

<filter> is a shell command which will receive all lines of the
PROGRAM output, which were not colorized by the built-in rules.");
}

fn main() {
    let mut verbose = false;
    let mut filter_command = None;
    let mut command = Vec::new();
    {
        let mut filter_command_arg = false;
        let mut command_arg = false;
        for arg in std::env::args().skip(1) {
            if command_arg {
                command.push(arg);
            } else if filter_command_arg {
                filter_command_arg = false;
                filter_command = Some(arg);
            } else if arg == "--" {
                command_arg = true;
            } else if arg == "--help" {
                usage();
                return;
            } else if arg == "-v" || arg == "--verbose" {
                verbose = true;
            } else if arg.starts_with("-c") {
                if arg.len() > 2 {
                    filter_command = Some(arg[2..].to_string());
                } else {
                    filter_command_arg = true;
                }
            } else {
                command.push(arg);
                command_arg = true;
            }
        }
    }
    if command.len() == 0 {
        usage();
        return;
    }
    if verbose {
        eprintln!("Running: {command:?}");
        if let Some(ref filter) = filter_command {
            eprintln!("Filter: \"{filter}\"");
        }
    }
    helpers::signal(libc::SIGINT, interrupt_handler);
    helpers::signal(libc::SIGTERM, terminate_handler);
    helpers::signal(libc::SIGPIPE, terminate_handler);
    helpers::signal(libc::SIGHUP, terminate_handler);

    let mut splitter = LineSplitter::<{ libc::BUFSIZ as usize }> {
        iob: iobuf::IoBuf::new(),
	markup: &mut regexcolors::RegexColors::new(),
        output: filter_command.map(|f| externalfilter::ExternalFilter::new(f.clone())),
    };
    if let Some(ref mut filter) = splitter.output {
        if let Err(errno) = filter.start() {
            eprintln!("Starting filter command: {}", errno);
            std::process::exit(254);
        }
    }
    match pty::ptyrun(command, &mut splitter) {
        -1 => std::process::exit(255),
        status =>
            if libc::WIFEXITED(status) {
                std::process::exit(libc::WEXITSTATUS(status))
            } else if libc::WIFSIGNALED(status) {
                std::process::exit(128 + libc::WTERMSIG(status))
            } else {
                std::process::exit(libc::WEXITSTATUS(status))
            }
    }
}

struct LineSplitter<'a, const S: usize> {
    iob: iobuf::IoBuf<S>,
    markup: &'a mut regexcolors::RegexColors,
    output: Option<externalfilter::ExternalFilter>,
}

impl<'a, const S: usize> LineSplitter<'a, S> {
    fn write_out(&self, b: &[u8]) {
        if let Some(ref filter) = self.output {
            filter.filter(b);
        } else {
            unsafe {
                libc::write(libc::STDOUT_FILENO,
                            b.as_ptr() as *const libc::c_void,
                            b.len());
            }
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

