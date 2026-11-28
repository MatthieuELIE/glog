use super::osc52;

#[test]
fn frames_an_empty_text_with_an_empty_payload() {
    assert_eq!(osc52(""), "\x1b]52;c;\x07");
}

#[test]
fn pads_one_trailing_byte_with_two_equal_signs() {
    assert_eq!(osc52("f"), "\x1b]52;c;Zg==\x07");
}

#[test]
fn pads_two_trailing_bytes_with_one_equal_sign() {
    assert_eq!(osc52("fo"), "\x1b]52;c;Zm8=\x07");
}

#[test]
fn encodes_whole_groups_of_three_bytes_without_padding() {
    assert_eq!(osc52("foo"), "\x1b]52;c;Zm9v\x07");
    assert_eq!(osc52("foobar"), "\x1b]52;c;Zm9vYmFy\x07");
}

#[test]
fn encodes_a_full_hash_with_two_equal_signs() {
    assert_eq!(
        osc52("0123456789abcdef0123456789abcdef01234567"),
        "\x1b]52;c;MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWYwMTIzNDU2Nw==\x07"
    );
}
