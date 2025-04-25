use rand::prelude::SliceRandom;
use rand::thread_rng;

pub fn generate_password(
    length: usize,
    include_special_symbols: bool,
    include_numbers: bool,
    include_uppercase: bool,
    include_lowercase: bool
) -> String {
    let mut charset: Vec<u8> = Vec::new();

    if include_lowercase {
        charset.extend(b'a'..=b'z');
    }
    if include_uppercase {
        charset.extend(b'A'..=b'Z');
    }
    if include_numbers {
        charset.extend(b'0'..=b'9');
    }
    if include_special_symbols {
        charset.extend(b"!?@#$%^&|*_-+/=<>(){}[]");
    }

    if charset.is_empty() { return String::new(); }

    let mut rng = thread_rng();
    (0..length)
        .filter_map(|_| charset.as_slice().choose(&mut rng))
        .map(|&c| c as char)
        .collect()
}