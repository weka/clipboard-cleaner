use regex::Regex;
use std::sync::OnceLock;

static SUFFIX_PATTERN: OnceLock<Regex> = OnceLock::new();
static TICKET_PATTERN: OnceLock<Regex> = OnceLock::new();

fn get_suffix_pattern() -> &'static Regex {
    SUFFIX_PATTERN.get_or_init(|| Regex::new(r" ?│[·]+$").expect("Failed to compile regex pattern"))
}

fn get_ticket_pattern() -> &'static Regex {
    TICKET_PATTERN.get_or_init(|| {
        Regex::new(r"^(?:WEKAPP|REG)-[0-9]+$").expect("Failed to compile ticket regex pattern")
    })
}

pub fn clean_content(content: &str) -> String {
    if content.is_empty() {
        return content.to_string();
    }

    // If the trimmed content is a ticket ID, just return it trimmed
    let trimmed = content.trim();
    if get_ticket_pattern().is_match(trimmed) {
        return trimmed.to_string();
    }

    let pattern = get_suffix_pattern();
    let mut lines: Vec<String> = content
        .lines()
        .map(|line| {
            let without_suffix = pattern.replace_all(line, "");
            without_suffix.trim_end().to_string()
        })
        .collect();

    while let Some(last_line) = lines.last() {
        if last_line.is_empty() {
            lines.pop();
        } else {
            break;
        }
    }

    if let Some(joined) = join_wrapped_token(&lines) {
        return joined;
    }

    strip_two_space_indent(&mut lines);

    lines.join("\n")
}

/// A long path/token wrapped by the terminal arrives as a first line with no
/// whitespace followed by lines indented by two spaces, each a single
/// whitespace-free chunk. Rejoin them into the original token.
fn join_wrapped_token(lines: &[String]) -> Option<String> {
    let (first, rest) = lines.split_first()?;
    if rest.is_empty() || first.is_empty() || first.contains(char::is_whitespace) {
        return None;
    }
    let mut joined = first.clone();
    for line in rest {
        let chunk = line.strip_prefix("  ")?;
        if chunk.is_empty() || chunk.contains(char::is_whitespace) {
            return None;
        }
        joined.push_str(chunk);
    }
    Some(joined)
}

/// Claude Code indents snippets by two spaces. If every non-empty line starts
/// with two spaces, remove them so the snippet can be pasted as-is.
fn strip_two_space_indent(lines: &mut [String]) {
    let mut non_empty = lines.iter().filter(|l| !l.is_empty()).peekable();
    if non_empty.peek().is_none() || !non_empty.all(|l| l.starts_with("  ")) {
        return;
    }
    for line in lines.iter_mut().filter(|l| !l.is_empty()) {
        line.drain(..2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_suffix_pattern_nospace() {
        let input = "Hello world│··············";
        let expected = "Hello world";
        assert_eq!(clean_content(input), expected);
    }

    #[test]
    fn test_remove_suffix_pattern() {
        let input = "Hello world │··············";
        let expected = "Hello world";
        assert_eq!(clean_content(input), expected);
    }

    #[test]
    fn test_remove_different_length_suffix() {
        let input = "Test text │···";
        let expected = "Test text";
        assert_eq!(clean_content(input), expected);
    }

    #[test]
    fn test_trim_trailing_spaces() {
        let input = "Hello world   ";
        let expected = "Hello world";
        assert_eq!(clean_content(input), expected);
    }

    #[test]
    fn test_multiline_content() {
        let input = "Line 1 │·····\nLine 2   \nLine 3 │···";
        let expected = "Line 1\nLine 2\nLine 3";
        assert_eq!(clean_content(input), expected);
    }

    #[test]
    fn test_no_changes_needed() {
        let input = "Clean text already";
        assert_eq!(clean_content(input), input);
    }

    #[test]
    fn test_empty_content() {
        let input = "";
        assert_eq!(clean_content(input), "");
    }

    #[test]
    fn test_remove_trailing_empty_lines() {
        let input = "Line 1\nLine 2\n\n\n";
        let expected = "Line 1\nLine 2";
        assert_eq!(clean_content(input), expected);
    }

    #[test]
    fn test_suffix_in_middle_not_removed() {
        let input = "Text │··· with more text after";
        assert_eq!(clean_content(input), input);
    }

    #[test]
    fn test_trim_wekapp_ticket() {
        assert_eq!(clean_content("  WEKAPP-12345  "), "WEKAPP-12345");
    }

    #[test]
    fn test_trim_wekapp_ticket_newlines() {
        assert_eq!(clean_content("\n WEKAPP-12345\n"), "WEKAPP-12345");
    }

    #[test]
    fn test_trim_reg_ticket() {
        assert_eq!(clean_content("  REG-999  "), "REG-999");
    }

    #[test]
    fn test_trim_reg_ticket_newlines() {
        assert_eq!(clean_content("\nREG-42\n\n"), "REG-42");
    }

    #[test]
    fn test_strip_two_space_indent() {
        let input = "  cd /tmp\n  ls -la\n    nested";
        let expected = "cd /tmp\nls -la\n  nested";
        assert_eq!(clean_content(input), expected);
    }

    #[test]
    fn test_strip_two_space_indent_ignores_empty_lines() {
        let input = "  for x in a b; do\n\n    echo $x\n  done\n";
        let expected = "for x in a b; do\n\n  echo $x\ndone";
        assert_eq!(clean_content(input), expected);
    }

    #[test]
    fn test_no_strip_when_some_line_not_indented() {
        let input = "  indented\nnot indented";
        assert_eq!(clean_content(input), input);
    }

    #[test]
    fn test_no_strip_when_single_space_indent() {
        let input = "  two\n one";
        assert_eq!(clean_content(input), input);
    }

    #[test]
    fn test_join_wrapped_path() {
        let input = "/tmp/claude-1001/-home-baruch-dev-bare-wekapp-git--cla\n  ude-worktrees-n2n-relay-qa/45247c19-d8f5-44b6-bc2b-e6ba0efe3a69/scratchpad/n1126/vrdf58/status";
        let expected = "/tmp/claude-1001/-home-baruch-dev-bare-wekapp-git--claude-worktrees-n2n-relay-qa/45247c19-d8f5-44b6-bc2b-e6ba0efe3a69/scratchpad/n1126/vrdf58/status";
        assert_eq!(clean_content(input), expected);
    }

    #[test]
    fn test_join_wrapped_path_multiple_breaks() {
        assert_eq!(clean_content("/a/b\n  c/d\n  e/f\n"), "/a/bc/de/f");
    }

    #[test]
    fn test_no_join_when_first_line_has_spaces() {
        let input = "ls /tmp\n  foo";
        assert_eq!(clean_content(input), input);
    }

    #[test]
    fn test_ticket_in_sentence_not_trimmed() {
        let input = "Fix WEKAPP-123 soon";
        assert_eq!(clean_content(input), input);
    }
}
