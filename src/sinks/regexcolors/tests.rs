#![allow(unused)]

use super::*;

#[test]
fn test_write1_rec_1level_match() {
    let mut buf = [0u8; 32];
    let mut bufo = 0;

    let m = [
        libc::regmatch_t { rm_so: 0, rm_eo: 8 },
    ];
    let clr = [
        &Markup { beg: "", clr: "X", end: "" },
    ];
    write1_rec("abcdefgh".as_bytes(), &m[..], &clr, 0,
               &mut |b: &[u8]| {
                   buf[bufo .. bufo + b.len()].copy_from_slice(b);
                   bufo += b.len();
               });
    dbg!(str::from_utf8(&buf[..bufo]));
    assert!(&buf[..bufo] == b"Xabcdefgh");
}

#[test]
fn test_write1_rec_2level_match() {
    let mut buf = [0u8; 32];
    let mut bufo = 0;

    // whole input match, with two non-intersecting, non-connecting
    // ranges inside
    let m = [
        libc::regmatch_t { rm_so: 0, rm_eo: 8 },
        libc::regmatch_t { rm_so: 1, rm_eo: 3 },
        libc::regmatch_t { rm_so: 5, rm_eo: 7 },
    ];
    let clr = [
        &Markup { beg: "", clr: "X", end: "" },
        &Markup { beg: "", clr: "Y", end: "" },
        &Markup { beg: "", clr: "Z", end: "" },
    ];
    write1_rec("abcdefgh".as_bytes(), &m[..], &clr, 0,
               &mut |b: &[u8]| {
                   buf[bufo .. bufo + b.len()].copy_from_slice(b);
                   bufo += b.len();
               });
    dbg!(str::from_utf8(&buf[..bufo]));
    assert!(&buf[..bufo] == b"XaYbcXdeZfgXh");

    buf[..].fill(0);
    bufo = 0;
    // 2level match, but not at the beginning, and before the end of the input
    let m = [
        libc::regmatch_t { rm_so: 2, rm_eo: 8 },
        libc::regmatch_t { rm_so: 4, rm_eo: 6 },
    ];
    let clr = [
        &Markup { beg: "", clr: "X", end: "" },
        &Markup { beg: "", clr: "Y", end: "" },
    ];
    write1_rec("abcdefghijkl".as_bytes(), &m[..], &clr, 0,
               &mut |b: &[u8]| {
                   buf[bufo .. bufo + b.len()].copy_from_slice(b);
                   bufo += b.len();
               });
    dbg!(str::from_utf8(&buf[..bufo]));
    assert!(&buf[..bufo] == b"XcdYefXgh");

    buf[..].fill(0);
    bufo = 0;
    // whole input match, with two non-intersecting, connecting
    // ranges inside
    let m = [
        libc::regmatch_t { rm_so: 0, rm_eo: 8 },
        libc::regmatch_t { rm_so: 1, rm_eo: 3 },
        libc::regmatch_t { rm_so: 3, rm_eo: 7 },
    ];
    let clr = [
        &Markup { beg: "", clr: "X", end: "" },
        &Markup { beg: "", clr: "Y", end: "" },
        &Markup { beg: "", clr: "Z", end: "" },
    ];
    write1_rec("abcdefgh".as_bytes(), &m[..], &clr, 0,
               &mut |b: &[u8]| {
                   buf[bufo .. bufo + b.len()].copy_from_slice(b);
                   bufo += b.len();
               });
    dbg!(str::from_utf8(&buf[..bufo]));
    assert!(&buf[..bufo] == b"XaYbcZdefgXh");
}

#[test]
fn test_write1_rec_3level_match() {
    let mut buf = [0u8; 32];
    let mut bufo = 0;

    let m = [
        libc::regmatch_t { rm_so: 0, rm_eo: 12 },
        libc::regmatch_t { rm_so: 2, rm_eo: 8 },
        libc::regmatch_t { rm_so: 4, rm_eo: 6 },
    ];
    let clr = [
        &Markup { beg: "", clr: "1", end: "" },
        &Markup { beg: "", clr: "2", end: "" },
        &Markup { beg: "", clr: "3", end: "" },
    ];
    write1_rec("abcdefghijkl".as_bytes(), &m[..], &clr, 0,
               &mut |b: &[u8]| {
                   buf[bufo .. bufo + b.len()].copy_from_slice(b);
                   bufo += b.len();
               });
    dbg!(str::from_utf8(&buf[..bufo]));
    assert!(&buf[..bufo] == b"1ab2cd3ef2gh1ijkl");
}

// FIXME this fails for now (support ^$ matches)
#[test]
fn test_write1_rec_empty_line() {
    let mut buf = [0u8; 32];
    let mut bufo = 0;

    let m = [
        libc::regmatch_t { rm_so: 0, rm_eo: 0 },
    ];
    let clr = [ &Markup { beg: "color", clr: "", end: "" } ];
    write1_rec("".as_bytes(), &m[..], &clr, 0,
               &mut |b: &[u8]| {
                   buf[bufo .. bufo + b.len()].copy_from_slice(b);
                   bufo += b.len();
               });
    dbg!(str::from_utf8(&buf[..bufo]));
    assert!(&buf[..bufo] == b"color");
}


