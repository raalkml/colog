/*
 * A log colorizer.
 */
extern crate libc;
mod sinks;
mod pty;
mod helpers;
mod iobuf;

use sinks::Sink;
use pty::ptyrun;

extern "C" fn interrupt_handler(sig: libc::c_int) {
    eprintln!("\n{:?}", helpers::strsignal(sig));
    std::process::exit(1);
}

extern "C" fn terminate_handler(sig: libc::c_int) {
    eprintln!("\n{:?}", helpers::strsignal(sig));
    std::process::exit(1);
}

fn usage() {
    println!("colog [-v|--verbose -c <filter>] PROGRAM ARGS...");
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
    let mut extf; // so that the *optional* sink can "live long enough"
    let hooks : &mut [&mut dyn Sink] = &mut [
	&mut sinks::regexcolors::RegexColors::new(),
	if let Some(ref filter) = filter_command {
            extf = sinks::externalfilter::ExternalFilter::new(filter.clone());
            if let Err(errno) = extf.start() {
                eprintln!("Starting filter command: {}", errno);
                std::process::exit(254);
            }
            &mut extf
        } else {
            &mut sinks::stdout::StdoutSink {}
        },
    ];

    match ptyrun(command, hooks) {
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

