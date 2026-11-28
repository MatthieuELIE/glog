const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64(bytes: &[u8]) -> String {
    bytes
        .chunks(3)
        .flat_map(|chunk| {
            let group = chunk.iter().fold(0u32, |group, &byte| (group << 8) | u32::from(byte)) << (8 * (3 - chunk.len()));

            (0..4).map(move |index| {
                if index > chunk.len() {
                    return '=';
                }

                char::from(BASE64[((group >> (18 - 6 * index)) & 0x3f) as usize])
            })
        })
        .collect()
}

/// The OSC 52 sequence that asks the terminal to put `text` in the clipboard.
pub fn osc52(text: &str) -> String {
    format!("\x1b]52;c;{}\x07", base64(text.as_bytes()))
}

#[cfg(test)]
mod tests;
