// What wie relies on since `patches/classfile` was dropped (docs/report/0572): crates.io `classfile`
// reads a CONSTANT_Utf8 through this decoder and `String::from_utf16_lossy`. These are the dropped
// patch's own cases; if a pin move breaks one, classes holding such constants stop loading.
use jvm_types::modified_utf8::decode;

fn constant(bytes: &[u8]) -> Option<String> {
    decode(bytes).map(|units| String::from_utf16_lossy(&units))
}

#[test]
fn class_constants_decode_as_modified_utf8() {
    // javac's encoding of "MTR\0" — the constant that first hit this (docs/report/0327)
    assert_eq!(constant(b"MTR\xC0\x80").as_deref(), Some("MTR\0"));
    // a supplementary character is a surrogate pair: six bytes, two 3-byte halves
    assert_eq!(constant(b"a\xED\xA0\xBD\xED\xB0\x8Db").as_deref(), Some("a\u{1F40D}b"));
    assert_eq!(constant(b"java/lang/Object").as_deref(), Some("java/lang/Object"));
    assert_eq!(constant("조약돌 5개".as_bytes()).as_deref(), Some("조약돌 5개"));
    assert_eq!(constant(b"").as_deref(), Some(""));
    // a lone surrogate half cannot live in a Rust String
    assert_eq!(constant(b"x\xED\xA0\xBD").as_deref(), Some("x\u{FFFD}"));
    // malformed: truncated sequence, stray continuation byte, invalid lead byte
    assert_eq!(constant(b"\xC0"), None);
    assert_eq!(constant(b"\x80"), None);
    assert_eq!(constant(b"\xFF"), None);
    // Unlike the dropped patch, standard 4-byte UTF-8 is rejected. javac never writes it, and no
    // class in the corpus holds one (0572 §3).
    assert_eq!(constant(b"\xF0\x9F\x90\x8D"), None);
}
