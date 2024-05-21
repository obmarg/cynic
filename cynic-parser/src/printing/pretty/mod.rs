use std::fmt::Write;

mod executable;
mod type_system;

fn escape_string(src: &str) -> String {
    let mut dest = String::with_capacity(src.len());

    for character in src.chars() {
        match character {
            '"' | '\\' | '\n' | '\r' | '\t' => {
                dest.extend(character.escape_default());
            }
            other if other.is_control() => {
                write!(&mut dest, "\\u{:04}", other as u32).ok();
            }
            _ => dest.push(character),
        }
    }

    dest
}
