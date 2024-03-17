fn unquote_block_string(src: &str) -> Result<String, Error<Token<'_>, Token<'_>>> {
    debug_assert!(src.starts_with("\"\"\"") && src.ends_with("\"\"\""));
    let lines = src[3..src.len() - 3].lines();

    let mut common_indent = usize::MAX;
    let mut first_non_empty_line: Option<usize> = None;
    let mut last_non_empty_line = 0;
    for (idx, line) in lines.clone().enumerate() {
        let indent = line.len() - line.trim_start().len();
        if indent == line.len() {
            continue;
        }

        first_non_empty_line.get_or_insert(idx);
        last_non_empty_line = idx;

        if idx != 0 {
            common_indent = std::cmp::min(common_indent, indent);
        }
    }

    if first_non_empty_line.is_none() {
        // The block string contains only whitespace.
        return Ok("".to_string());
    }
    let first_non_empty_line = first_non_empty_line.unwrap();

    let mut result = String::with_capacity(src.len() - 6);
    let mut lines = lines
        .enumerate()
        // Skip leading and trailing empty lines.
        .skip(first_non_empty_line)
        .take(last_non_empty_line - first_non_empty_line + 1)
        // Remove indent, except the first line.
        .map(|(idx, line)| {
            if idx != 0 && line.len() >= common_indent {
                &line[common_indent..]
            } else {
                line
            }
        })
        // Handle escaped triple-quote (\""").
        .map(|x| x.replace(r#"\""""#, r#"""""#));

    if let Some(line) = lines.next() {
        result.push_str(&line);

        for line in lines {
            result.push_str("\n");
            result.push_str(&line);
        }
    }
    return Ok(result);
}

fn unquote_string(s: &str) -> Result<String, Error<Token, Token>> {
    let mut res = String::with_capacity(s.len());
    debug_assert!(s.starts_with('"') && s.ends_with('"'));
    let mut chars = s[1..s.len() - 1].chars();
    let mut temp_code_point = String::with_capacity(4);
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                match chars.next().expect("slash cant be at the end") {
                    c @ '"' | c @ '\\' | c @ '/' => res.push(c),
                    'b' => res.push('\u{0010}'),
                    'f' => res.push('\u{000C}'),
                    'n' => res.push('\n'),
                    'r' => res.push('\r'),
                    't' => res.push('\t'),
                    'u' => {
                        temp_code_point.clear();
                        for _ in 0..4 {
                            match chars.next() {
                                Some(inner_c) => temp_code_point.push(inner_c),
                                None => {
                                    return Err(Error::Unexpected(Info::Owned(
                                        format_args!(
                                            "\\u must have 4 characters after it, only found '{}'",
                                            temp_code_point
                                        )
                                        .to_string(),
                                    )))
                                }
                            }
                        }

                        // convert our hex string into a u32, then convert that into a char
                        match u32::from_str_radix(&temp_code_point, 16).map(std::char::from_u32) {
                            Ok(Some(unicode_char)) => res.push(unicode_char),
                            _ => {
                                return Err(Error::Unexpected(Info::Owned(
                                    format_args!(
                                        "{} is not a valid unicode code point",
                                        temp_code_point
                                    )
                                    .to_string(),
                                )))
                            }
                        }
                    }
                    c => {
                        return Err(Error::Unexpected(Info::Owned(
                            format_args!("bad escaped char {:?}", c).to_string(),
                        )));
                    }
                }
            }
            c => res.push(c),
        }
    }

    Ok(res)
}
