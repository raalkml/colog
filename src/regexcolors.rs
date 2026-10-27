use crate::helpers::count_of;
use crate::eprintsln;
use core::mem;
mod tests;

#[derive(Debug)]
struct Markup {
    beg: &'static str,
    clr: &'static str,
    end: &'static str,
}

#[derive(Debug)]
struct Pattern {
    patn: &'static str,
    clr: &'static [&'static Markup],
}

macro_rules! patn {
    ($pat:literal, $clr:expr) => { Pattern { patn: concat!($pat, "\0"), clr: $clr } };
    ($pat:literal, $clr:expr, $start:expr, $end:expr) => {
        Pattern { patn: concat!($pat, "\0"), clr: $clr }
    };
}
macro_rules! tspn {
    ($pat:literal, $clr:expr) => {
        Pattern {
            patn: concat!(
                // unix syslog and systemd timestamp formattings
                "(...[ \t]*[0-9]+[ \t]+..:..:.. |[0-9]+-[0-9]+-[0-9]+T..:..:..\\.[0-9]+\\+..:.. )",
                $pat, "\0"),
            clr: $clr,
        }
    };
}
macro_rules! C {
    ($($clr:expr),+) => { &[ $( &Markup { beg: "", clr: $clr, end: "" }),+ ] };
}

static PATTERNS : [ Pattern; 6 ] = [
    patn!("^==> (.*) <==$", C!(MAGENTA, "\x1b[37m")),
    tspn!("([^ \t]+ )(postfix/local)(\\[[0-9]+\\])(:)(.*, status=sent )",
          C!(CYAN, YELLOW, CYAN, BOLD_GREEN, GREEN, BOLD_CYAN, CYAN)),
    tspn!("([^ \t]+ )(postfix/smtp)(\\[[0-9]+\\])(:)(.*, status=deferred )(.*)",
          C!(CYAN, YELLOW, CYAN, BOLD_GREEN, GREEN, BOLD_CYAN, CYAN, RED)),
    tspn!("([^ \t]+ )([^ \t]+)(\\[[0-9]+\\])(:)(.*)",
          C!(CYAN, YELLOW, CYAN, BOLD_GREEN, GREEN, BOLD_CYAN, CYAN)),
    tspn!("([^ \t]+ )([^ \t[]+):(.*)",
          C!(CYAN, YELLOW, CYAN, BOLD_GREEN, CYAN)),
    patn!("^$", &[ &Markup {beg: "\x1b[35m$", clr: "", end: "\x1b[0m" } ]),
];

static DEFLT : &[u8] = b"\x1b[0m";
static EOL : &[u8] = b"\x1b[0m\r\n";
static CYAN : &str = "\x1b[22;36m";
static BOLD_CYAN : &str = "\x1b[1;36m";
static YELLOW : &str = "\x1b[22;33m";
//static BOLD_YELLOW : &str = "\x1b[1;33m";
static RED : &str = "\x1b[22;31m";
//static BOLD_RED : &str = "\x1b[1;31m";
static GREEN : &str = "\x1b[22;32m";
static BOLD_GREEN : &str = "\x1b[1;32m";
static MAGENTA : &str = "\x1b[22;35m";

pub struct RegexColors {
    re: [[u8; mem::size_of::<libc::regex_t>()]; count_of(&PATTERNS) ]
}

impl RegexColors {
    pub fn new() -> RegexColors {
        let rc = RegexColors { re: [ [0; mem::size_of::<libc::regex_t>()]; count_of(&PATTERNS) ] };
        for (i, pat) in PATTERNS.iter().enumerate() {
            let err = unsafe {
                libc::regcomp(rc.re[i].as_ptr() as *mut libc::regex_t,
                              pat.patn.as_ptr() as *const libc::c_char,
                              libc::REG_EXTENDED | libc::REG_ICASE)
            };
            if err != 0 {
                eprintsln!("TODO print regex compilation errors");
                //eprintln!("regcomp{i} \"{}\" -> {_err}", pat.patn);
            }
        }
        rc
    }
}

fn write1(b: &[u8]) {
    unsafe {
        libc::write(1, b.as_ptr() as *const libc::c_void,
                    b.len() as libc::size_t);
    }
}

fn write1_rec<F>(buf: &[u8], m: &[libc::regmatch_t], clr: &[&Markup], sub: usize, out: &mut F) -> usize
where
    F: FnMut(&[u8])
{
    assert!(clr.len() > 0);
    assert!(sub < m.len());
    assert!(sub < clr.len());
    assert!(m[sub].rm_so >= 0 && m[sub].rm_eo >= 0);

    if clr[sub].beg.len() > 0 { out(clr[sub].beg.as_bytes()); }
    let mut so = m[sub].rm_so;
    let eo = m[sub].rm_eo;
    let mut next = sub + 1;
    while next < clr.len() {
        if m[next].rm_so >= eo { break; }
        if so < m[next].rm_so {
            out(clr[sub].clr.as_bytes());
            out(&buf[so as usize .. m[next].rm_so as usize]);
        }
        so = m[next].rm_eo;
        next = write1_rec(buf, m, clr, next, out);
    }
    if so < eo {
        out(clr[sub].clr.as_bytes());
        out(&buf[so as usize .. eo as usize]);
    }
    if clr[sub].end.len() > 0 { out(clr[sub].end.as_bytes()); }
    next
}

impl RegexColors {
    pub fn markup(&mut self, buf: &[u8]) -> bool {
        let mut len = buf.len();
        for c in buf.iter().rev() {
            if *c == b'\r' || *c == b'\n' { len -= 1; } else { break; }
        }
        let mut m = [ libc::regmatch_t {rm_so: 0, rm_eo: 0}; 20 ];
        for (i, pat) in PATTERNS.iter().enumerate() {
            // See regexec manual for REG_STARTEND:
            m[0] = libc::regmatch_t { rm_so: 0, rm_eo: len as libc::regoff_t };
            if unsafe {
                libc::regexec(self.re[i].as_ptr() as *const libc::regex_t,
                              buf.as_ptr() as *const libc::c_char,
                              pat.clr.len().clamp(1, count_of(&m)) as libc::size_t,
                              m.as_ptr() as *mut libc::regmatch_t,
                              libc::REG_STARTEND)
            } == 0 {
                // `m[0].rm_[se]o` cannot be negative if `err == 0`:
                // it contains the whole range of this match
                assert!(m[0].rm_so >= 0  && m[0].rm_eo >= m[0].rm_so);

                // the part before match
                if 0 < m[0].rm_so {
                    write1(DEFLT);
                    write1(&buf[..m[0].rm_so as usize]);
                }
                // the match
                if pat.clr.len() == 0 {
                    write1(&buf[m[0].rm_so as usize .. m[0].rm_eo as usize]);
                } else {
                    write1_rec(&buf[..len], &m[..], pat.clr, 0, &mut write1);
                }
                // part after match
                if m[0].rm_eo < len as libc::regoff_t {
                    write1(DEFLT);
                    write1(&buf[m[0].rm_eo as usize .. len]);
                }
                // the line terminator (the original line terminator was removed)
                write1(EOL);
                return true;
            }
        }
        false
    }
}

