use super::*;

#[test]
fn test_iobuf_normal() {
    let mut iob = IoBuf::<{ libc::BUFSIZ as usize }>::new();
    let buf = iob.get_buf(7);
    assert!(buf.len() == 7);
    buf.copy_from_slice(b"abc\ndef");
    let len = buf.len();
    iob.put_buf(len);
    assert!(iob.peek() == b"abc\ndef");

    let ln = iob.peek_line();
    assert!(ln == b"abc\n");
    iob.consume(ln.len());

    let ln = iob.peek_line();
    assert!(ln == b"");

    assert!(iob.peek() == b"def");

    iob.consume(iob.peek().len());
    assert!(iob.peek() == b"");
}

#[test]
fn test_iobuf_overflows() {
    let mut iob = IoBuf::<16>::new();

    let buf = iob.get_buf(7);
    buf.copy_from_slice(b"abc\ndef");
    iob.put_buf(7);
    assert!(iob.peek() == b"abc\ndef");
    let buf = iob.get_buf(8);
    buf.copy_from_slice(b"\nABC\nDEF");
    iob.put_buf(8);
    assert!(iob.peek() == b"abc\ndef\nABC\nDEF");

    let buf = iob.get_buf(8);
    assert!(buf.len() == 1 /* 16 - 7 - 8 */);
    buf.copy_from_slice(b"\n");
    iob.put_buf(1);

    iob.consume(8);
    assert!(iob.peek() == b"ABC\nDEF\n");
    let buf = iob.get_buf(8);
    assert!(buf.len() == 8); // relocated
    buf.copy_from_slice(b"0123456\n");
    iob.put_buf(8);
    assert!(iob.peek() == b"ABC\nDEF\n0123456\n");

    assert!(iob.peek_line() == b"ABC\n");
    iob.consume(4);
    assert!(iob.peek_line() == b"DEF\n");
    iob.consume(4);
    assert!(iob.peek_line() == b"0123456\n");
    iob.consume(8);

    assert!(iob.peek_line() == b"");
    assert!(iob.peek() == b"");
}

