use std::fmt;

use crate::{lexer, parser::AdditionalErrors, Span};

pub(crate) fn unquote_block_string(src: &str) -> &str {
    assert!(src.starts_with("\"\"\"") && src.ends_with("\"\"\""));

    &src[3..src.len() - 3]
}

pub(crate) fn trim_block_string_whitespace(src: &str) -> String {
    let lines = src.lines().collect::<Vec<_>>();

    let mut common_indent = usize::MAX;
    let mut first_non_empty_line: Option<usize> = None;
    let mut last_non_empty_line = 0;
    for (idx, line) in lines.iter().enumerate() {
        let indent = line.find(|c: char| !c.is_whitespace());

        if indent.is_none() || indent.unwrap() == line.len() {
            continue;
        }
        let indent = indent.unwrap();

        first_non_empty_line.get_or_insert(idx);
        last_non_empty_line = idx;

        if idx != 0 {
            common_indent = std::cmp::min(common_indent, indent);
        }
    }

    let Some(first_non_empty_line) = first_non_empty_line else {
        // The block string contains only whitespace.
        return "".to_string();
    };

    let mut result = String::with_capacity(src.len() - 6);
    let mut lines = lines
        .into_iter()
        .enumerate()
        // Skip leading and trailing empty lines.
        .skip(first_non_empty_line)
        .take(last_non_empty_line - first_non_empty_line + 1)
        // Remove indent, except the first line.
        .map(|(idx, line)| {
            if idx == 0 {
                line
            } else if line.len() >= common_indent {
                &line[common_indent..]
            } else {
                ""
            }
        })
        // Handle escaped triple-quote (\""").
        .map(|x| x.replace(r#"\""""#, r#"""""#));

    if let Some(line) = lines.next() {
        // TODO: Handle replacing the escaped tripe quote inline here maybe?
        // Or possibly just don't, I don't know.
        result.push_str(&line);

        for line in lines {
            result.push('\n');
            result.push_str(&line);
        }
    }

    result
}

pub(crate) fn unquote_string(s: &str, start_span: usize) -> Result<String, MalformedStringError> {
    let mut res = String::with_capacity(s.len());
    assert!(s.starts_with('"') && s.ends_with('"'));

    let mut chars = s[1..s.len() - 1].char_indices();

    // Count the '"' in our span
    let start_span = start_span + 1;

    let mut temp_code_point = String::with_capacity(4);
    while let Some((index, c)) = chars.next() {
        match c {
            '\\' => {
                match chars.next().expect("slash cant be at the end") {
                    (_, c @ '"' | c @ '\\' | c @ '/') => res.push(c),
                    (_, 'b') => res.push('\u{0010}'),
                    (_, 'f') => res.push('\u{000C}'),
                    (_, 'n') => res.push('\n'),
                    (_, 'r') => res.push('\r'),
                    (_, 't') => res.push('\t'),
                    (_, 'u') => {
                        temp_code_point.clear();
                        let mut end_index = index;
                        for _ in 0..4 {
                            match chars.next() {
                                Some((index, inner_c)) => {
                                    temp_code_point.push(inner_c);
                                    end_index = index;
                                }
                                None => {
                                    return Err(MalformedStringError::MalformedCodePoint(
                                        temp_code_point,
                                        index + start_span,
                                        end_index + start_span,
                                    ));
                                }
                            }
                        }

                        // convert our hex string into a u32, then convert that into a char
                        match u32::from_str_radix(&temp_code_point, 16).map(std::char::from_u32) {
                            Ok(Some(unicode_char)) => res.push(unicode_char),
                            _ => {
                                return Err(MalformedStringError::UnknownCodePoint(
                                    temp_code_point,
                                    index + start_span,
                                    end_index + start_span,
                                ));
                            }
                        }
                    }
                    (end_index, c) => {
                        return Err(MalformedStringError::UnknownEscapeChar(
                            c,
                            index + start_span,
                            end_index + start_span,
                        ));
                    }
                }
            }
            c => res.push(c),
        }
    }

    Ok(res)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MalformedStringError {
    MalformedCodePoint(String, usize, usize),
    UnknownCodePoint(String, usize, usize),
    UnknownEscapeChar(char, usize, usize),
}

impl MalformedStringError {
    pub fn span(&self) -> Span {
        let (start, end) = match self {
            MalformedStringError::MalformedCodePoint(_, start, end) => (start, end),
            MalformedStringError::UnknownCodePoint(_, start, end) => (start, end),
            MalformedStringError::UnknownEscapeChar(_, start, end) => (start, end),
        };

        Span::new(*start, *end)
    }
}

impl std::error::Error for MalformedStringError {}

impl fmt::Display for MalformedStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MalformedStringError::MalformedCodePoint(code_point, _, _) => {
                write!(
                    f,
                    "\\u must have 4 characters after it, only found '{code_point}'"
                )
            }
            MalformedStringError::UnknownCodePoint(code_point, _, _) => {
                write!(f, "{code_point} is not a valid unicode code point",)
            }
            MalformedStringError::UnknownEscapeChar(char, _, _) => {
                write!(f, "unknown escape character {char}")
            }
        }
    }
}

impl From<MalformedStringError>
    for lalrpop_util::ParseError<usize, lexer::Token<'static>, AdditionalErrors>
{
    fn from(value: MalformedStringError) -> Self {
        lalrpop_util::ParseError::User {
            error: AdditionalErrors::MalformedString(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::trim_block_string_whitespace;

    #[test]
    fn test_block_string_trim() {
        assert_eq!(
            trim_block_string_whitespace(
                r#"Hello there you fool

            I am a thing

                I am indented
            "#
            ),
            "Hello there you fool\n\nI am a thing\n\n    I am indented"
        );

        assert_eq!(
            trim_block_string_whitespace(
                r#"
            Hello there you fool

            I am a thing

                I am indented
            "#
            ),
            "Hello there you fool\n\nI am a thing\n\n    I am indented"
        );
    }
}
