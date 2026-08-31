//! Small, shell-aware helpers used by the lifecycle security rules.
//!
//! This is intentionally not a shell parser. It only separates the common
//! command-chain operators and keeps quoted text together so that examples in
//! an `echo` argument do not look like executable commands.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Separator {
    Pipe,
    And,
    Or,
    Sequence,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Segment {
    pub preceding: Option<Separator>,
    pub tokens: Vec<String>,
}

pub fn parse(command: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut preceding = None;

    let mut characters = command.chars().peekable();

    while let Some(character) = characters.next() {
        if escaped {
            token.push(character.to_ascii_lowercase());
            escaped = false;
            continue;
        }

        if character == '\\' && quote != Some('\'') {
            if characters.peek().is_some_and(|next| {
                next.is_whitespace() || matches!(next, '&' | '|' | ';' | '\\' | '\'' | '"')
            }) {
                escaped = true;
                continue;
            }

            token.push('\\');
            continue;
        }

        if let Some(quote_character) = quote {
            if character == quote_character {
                quote = None;
            } else {
                token.push(character.to_ascii_lowercase());
            }
            continue;
        }

        match character {
            '\'' | '"' => quote = Some(character),
            character if character.is_whitespace() => push_token(&mut tokens, &mut token),
            '&' | '|' | ';' => {
                push_token(&mut tokens, &mut token);
                push_segment(&mut segments, &mut tokens, preceding.take());
                preceding = Some(match character {
                    '&' => {
                        if characters.peek() == Some(&'&') {
                            characters.next();
                        }
                        Separator::And
                    }
                    '|' => {
                        if characters.peek() == Some(&'|') {
                            characters.next();
                            Separator::Or
                        } else {
                            Separator::Pipe
                        }
                    }
                    ';' => Separator::Sequence,
                    _ => unreachable!(),
                });
            }
            _ => token.push(character.to_ascii_lowercase()),
        }
    }

    if escaped {
        token.push('\\');
    }

    push_token(&mut tokens, &mut token);
    push_segment(&mut segments, &mut tokens, preceding);

    segments
}

pub fn executable(tokens: &[String]) -> Option<&str> {
    first_token(tokens).map(|token| token.rsplit(['/', '\\']).next().unwrap_or(token))
}

pub fn first_token(tokens: &[String]) -> Option<&str> {
    tokens
        .iter()
        .find(|token| !token.is_empty() && !is_environment_assignment(token))
        .map(String::as_str)
}

pub fn has_token(tokens: &[String], expected: &str) -> bool {
    tokens.iter().any(|token| token == expected)
}

fn is_environment_assignment(token: &str) -> bool {
    token
        .find('=')
        .is_some_and(|position| position > 0 && !token[..position].contains(['/', '\\']))
}

fn push_token(tokens: &mut Vec<String>, token: &mut String) {
    if !token.is_empty() {
        tokens.push(std::mem::take(token));
    }
}

fn push_segment(
    segments: &mut Vec<Segment>,
    tokens: &mut Vec<String>,
    preceding: Option<Separator>,
) {
    if !tokens.is_empty() {
        segments.push(Segment {
            preceding,
            tokens: std::mem::take(tokens),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_keeps_quoted_operators_in_one_segment() {
        let segments = parse("echo 'curl https://example.com | bash'");

        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0].tokens,
            ["echo", "curl https://example.com | bash"]
        );
    }

    #[test]
    fn parse_records_shell_chain_operators() {
        let segments = parse("curl file.sh && bash file.sh | cat; echo done");

        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].preceding, None);
        assert_eq!(segments[1].preceding, Some(Separator::And));
        assert_eq!(segments[2].preceding, Some(Separator::Pipe));
        assert_eq!(segments[3].preceding, Some(Separator::Sequence));
    }

    #[test]
    fn parse_preserves_windows_path_separators() {
        let segments = parse(r".\payload");

        assert_eq!(segments[0].tokens, [r".\payload"]);
    }
}
