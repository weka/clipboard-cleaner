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

    lines.join("\n")
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
    fn test_ticket_in_sentence_not_trimmed() {
        let input = "Fix WEKAPP-123 soon";
        assert_eq!(clean_content(input), input);
    }
}
