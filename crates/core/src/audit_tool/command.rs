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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenQuote {
    Unquoted,
    Single,
    Double,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TokenPart {
    pub text: String,
    pub quote: TokenQuote,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Segment {
    pub preceding: Option<Separator>,
    pub tokens: Vec<String>,
    pub(crate) token_parts: Vec<Vec<TokenPart>>,
}

pub fn parse(command: &str) -> Vec<Segment> {
    parse_internal(command, true)
}

/// Parses command segments while retaining the original case of token data.
///
/// Lifecycle rules use [`parse`] so command matching remains case-insensitive.
/// Workflow secret analysis also needs the original case because GitHub secret
/// and shell variable names are case-sensitive.
pub(crate) fn parse_preserving_case(command: &str) -> Vec<Segment> {
    parse_internal(command, false)
}

fn parse_internal(command: &str, lowercase: bool) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut tokens = Vec::new();
    let mut token_parts = Vec::new();
    let mut token = String::new();
    let mut token_part = String::new();
    let mut token_parts_for_token = Vec::new();
    let mut token_quote = TokenQuote::Unquoted;
    let mut quote = None;
    let mut escaped = false;
    let mut preceding = None;

    let mut characters = command.chars().peekable();

    while let Some(character) = characters.next() {
        if escaped {
            escaped = false;
            if !matches!(character, '\n' | '\r') {
                push_character(&mut token, &mut token_part, normalize(character, lowercase));
            } else if character == '\r' && characters.peek() == Some(&'\n') {
                characters.next();
            }
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
                push_part(&mut token_parts_for_token, &mut token_part, token_quote);
                quote = None;
                token_quote = TokenQuote::Unquoted;
            } else {
                push_character(&mut token, &mut token_part, normalize(character, lowercase));
            }
            continue;
        }

        // GitHub expressions may contain spaces even when the surrounding
        // shell argument is not quoted. Retain a complete expression as one
        // token so downstream workflow analysis can inspect it without
        // treating the expression body as separate shell arguments.
        if character == '$' && characters.peek() == Some(&'{') {
            let mut lookahead = characters.clone();
            lookahead.next();
            if lookahead.next() == Some('{') {
                push_character(&mut token, &mut token_part, normalize(character, lowercase));
                push_character(&mut token, &mut token_part, characters.next().unwrap());
                push_character(&mut token, &mut token_part, characters.next().unwrap());
                while let Some(expression_character) = characters.next() {
                    push_character(
                        &mut token,
                        &mut token_part,
                        normalize(expression_character, lowercase),
                    );
                    if expression_character == '}' && characters.peek() == Some(&'}') {
                        push_character(
                            &mut token,
                            &mut token_part,
                            normalize(characters.next().unwrap(), lowercase),
                        );
                        break;
                    }
                }
                continue;
            }
        }

        match character {
            '\'' | '"' => {
                push_part(&mut token_parts_for_token, &mut token_part, token_quote);
                quote = Some(character);
                token_quote = if character == '\'' {
                    TokenQuote::Single
                } else {
                    TokenQuote::Double
                };
            }
            '\r' | '\n' => {
                if character == '\r' && characters.peek() == Some(&'\n') {
                    characters.next();
                }
                push_token(
                    &mut tokens,
                    &mut token_parts,
                    &mut token,
                    &mut token_part,
                    &mut token_parts_for_token,
                    token_quote,
                );
                if tokens.is_empty() {
                    if preceding.is_none() {
                        preceding = Some(Separator::Sequence);
                    }
                } else {
                    push_segment(
                        &mut segments,
                        &mut tokens,
                        &mut token_parts,
                        preceding.take(),
                    );
                    preceding = Some(Separator::Sequence);
                }
            }
            character if character.is_whitespace() => push_token(
                &mut tokens,
                &mut token_parts,
                &mut token,
                &mut token_part,
                &mut token_parts_for_token,
                token_quote,
            ),
            '&' | '|' | ';' => {
                push_token(
                    &mut tokens,
                    &mut token_parts,
                    &mut token,
                    &mut token_part,
                    &mut token_parts_for_token,
                    token_quote,
                );
                push_segment(
                    &mut segments,
                    &mut tokens,
                    &mut token_parts,
                    preceding.take(),
                );
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
            '#' if token.is_empty() => {
                while let Some(comment_character) = characters.next() {
                    if comment_character == '\n' || comment_character == '\r' {
                        if comment_character == '\r' && characters.peek() == Some(&'\n') {
                            characters.next();
                        }
                        push_token(
                            &mut tokens,
                            &mut token_parts,
                            &mut token,
                            &mut token_part,
                            &mut token_parts_for_token,
                            token_quote,
                        );
                        push_segment(
                            &mut segments,
                            &mut tokens,
                            &mut token_parts,
                            preceding.take(),
                        );
                        preceding = Some(Separator::Sequence);
                        break;
                    }
                }
            }
            _ => push_character(&mut token, &mut token_part, normalize(character, lowercase)),
        }
    }

    if escaped {
        push_character(&mut token, &mut token_part, '\\');
    }

    push_token(
        &mut tokens,
        &mut token_parts,
        &mut token,
        &mut token_part,
        &mut token_parts_for_token,
        token_quote,
    );
    push_segment(&mut segments, &mut tokens, &mut token_parts, preceding);

    segments
}

fn normalize(character: char, lowercase: bool) -> char {
    if lowercase {
        character.to_ascii_lowercase()
    } else {
        character
    }
}

fn push_character(token: &mut String, token_part: &mut String, character: char) {
    token.push(character);
    token_part.push(character);
}

fn push_part(
    token_parts_for_token: &mut Vec<TokenPart>,
    token_part: &mut String,
    quote: TokenQuote,
) {
    if !token_part.is_empty() {
        token_parts_for_token.push(TokenPart {
            text: std::mem::take(token_part),
            quote,
        });
    }
}

pub fn executable(tokens: &[String]) -> Option<&str> {
    command_token(tokens).map(|token| token.rsplit(['/', '\\']).next().unwrap_or(token))
}

/// Returns the executable name from a case-preserving parse.
pub(crate) fn executable_preserving_case(tokens: &[String]) -> Option<&str> {
    command_token_preserving_case(tokens)
        .map(|token| token.rsplit(['/', '\\']).next().unwrap_or(token))
}

pub fn command_token(tokens: &[String]) -> Option<&str> {
    executable_index(tokens).map(|index| tokens[index].as_str())
}

pub(crate) fn command_token_preserving_case(tokens: &[String]) -> Option<&str> {
    executable_index_with_case(tokens, true).map(|index| tokens[index].as_str())
}

pub fn arguments(tokens: &[String]) -> &[String] {
    executable_index(tokens)
        .map(|index| &tokens[index + 1..])
        .unwrap_or(&[])
}

pub(crate) fn arguments_preserving_case(tokens: &[String]) -> &[String] {
    executable_index_with_case(tokens, true)
        .map(|index| &tokens[index + 1..])
        .unwrap_or(&[])
}

pub(crate) fn argument_parts_preserving_case<'a>(
    tokens: &[String],
    token_parts: &'a [Vec<TokenPart>],
) -> &'a [Vec<TokenPart>] {
    executable_index_with_case(tokens, true)
        .map(|index| &token_parts[index + 1..])
        .unwrap_or(&[])
}

fn is_environment_assignment(token: &str) -> bool {
    token
        .find('=')
        .is_some_and(|position| position > 0 && !token[..position].contains(['/', '\\']))
}

fn first_token_index(tokens: &[String]) -> Option<usize> {
    tokens
        .iter()
        .position(|token| !token.is_empty() && !is_environment_assignment(token))
}

fn executable_index(tokens: &[String]) -> Option<usize> {
    executable_index_with_case(tokens, false)
}

fn executable_index_with_case(tokens: &[String], ignore_case: bool) -> Option<usize> {
    let mut index = first_token_index(tokens)?;

    loop {
        let executable = tokens[index]
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(&tokens[index]);

        if equals(executable, "command", ignore_case) {
            index += 1;
        } else if equals(executable, "env", ignore_case) {
            index += 1;
            while index < tokens.len()
                && (is_environment_assignment(&tokens[index])
                    || equals(&tokens[index], "-i", ignore_case))
            {
                index += 1;
            }
        } else if equals(executable, "sudo", ignore_case) {
            index += 1;
            skip_sudo_options(tokens, &mut index, ignore_case);
        } else {
            return (index < tokens.len()).then_some(index);
        }

        if index >= tokens.len() {
            return None;
        }
    }
}

fn skip_sudo_options(tokens: &[String], index: &mut usize, ignore_case: bool) {
    while *index < tokens.len() {
        let token = &tokens[*index];

        if is_environment_assignment(token) {
            *index += 1;
            continue;
        }

        if !token.starts_with('-') {
            break;
        }

        let takes_value = [
            "-u", "--user", "-g", "--group", "-p", "--prompt", "-C", "--chdir",
        ]
        .iter()
        .any(|option| equals(token, option, ignore_case));
        *index += 1;

        if takes_value && *index < tokens.len() {
            *index += 1;
        }
    }
}

fn equals(left: &str, right: &str, ignore_case: bool) -> bool {
    if ignore_case {
        left.eq_ignore_ascii_case(right)
    } else {
        left == right
    }
}

fn push_token(
    tokens: &mut Vec<String>,
    token_parts: &mut Vec<Vec<TokenPart>>,
    token: &mut String,
    token_part: &mut String,
    token_parts_for_token: &mut Vec<TokenPart>,
    token_quote: TokenQuote,
) {
    if !token.is_empty() {
        push_part(token_parts_for_token, token_part, token_quote);
        tokens.push(std::mem::take(token));
        token_parts.push(std::mem::take(token_parts_for_token));
    }
}

fn push_segment(
    segments: &mut Vec<Segment>,
    tokens: &mut Vec<String>,
    token_parts: &mut Vec<Vec<TokenPart>>,
    preceding: Option<Separator>,
) {
    if !tokens.is_empty() {
        segments.push(Segment {
            preceding,
            tokens: std::mem::take(tokens),
            token_parts: std::mem::take(token_parts),
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
    fn parse_records_unescaped_newlines_as_command_boundaries() {
        let segments = parse("echo setup\nwget payload\nchmod +x payload\n./payload");

        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].tokens, ["echo", "setup"]);
        assert_eq!(segments[1].preceding, Some(Separator::Sequence));
        assert_eq!(segments[1].tokens, ["wget", "payload"]);
        assert_eq!(segments[2].preceding, Some(Separator::Sequence));
        assert_eq!(segments[2].tokens, ["chmod", "+x", "payload"]);
        assert_eq!(segments[3].preceding, Some(Separator::Sequence));
        assert_eq!(segments[3].tokens, ["./payload"]);
    }

    #[test]
    fn parse_preserves_line_continuations_and_operator_continuations() {
        let continued = parse(concat!("echo setup \\", "\n", "wget payload"));
        assert_eq!(continued.len(), 1);
        assert_eq!(continued[0].tokens, ["echo", "setup", "wget", "payload"]);

        let operator_continued = parse("curl payload |\n bash");
        assert_eq!(operator_continued.len(), 2);
        assert_eq!(operator_continued[1].preceding, Some(Separator::Pipe));
        assert_eq!(operator_continued[1].tokens, ["bash"]);
    }

    #[test]
    fn parse_preserves_windows_path_separators() {
        let segments = parse(r".\payload");

        assert_eq!(segments[0].tokens, [r".\payload"]);
    }

    #[test]
    fn executable_skips_command_wrappers() {
        let segments = parse("sudo -u root rm -rf /tmp/example");

        assert_eq!(executable(&segments[0].tokens), Some("rm"));
        assert_eq!(arguments(&segments[0].tokens), ["-rf", "/tmp/example"]);
    }
}
