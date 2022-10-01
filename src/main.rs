use regex::Regex;
use std::fmt::Write;
use std::io::{self, Write as IOWrite};
use std::ops::ControlFlow;
use std::{fs, path};

fn parse_line(line: &str) -> String {
    let mut parsed_line = String::new();
    let mut bold = false;
    let mut italic = false;
    let mut inline_code = false;

    let mut is_reading_link_text = false;
    let mut is_reading_href = false;
    let mut link_text = String::new();
    let mut link_href = String::new();

    macro_rules! parse {
        ($variable_name:ident, $tag:literal) => {
            if $variable_name {
                write!(parsed_line, "</{}>", $tag).unwrap();
                $variable_name = false;
            } else {
                write!(parsed_line, "<{}>", $tag).unwrap();
                $variable_name = true;
            }
        };
    }
    for c in line.chars() {
        if c == '_' {
            parse!(italic, "i");
        } else if c == '*' {
            parse!(bold, "b");
        } else if c == '`' {
            parse!(inline_code, "code");
        } else if c == '[' && !is_reading_link_text {
            is_reading_link_text = true;
        } else if c == ']' {
            is_reading_link_text = false;
        } else if is_reading_link_text {
            link_text.push(c);
        } else if c == '(' && !link_text.is_empty() {
            is_reading_href = true;
        } else if c == ')' && is_reading_href {
            is_reading_href = false;
            write!(
                parsed_line,
                r#"<a href="{link_href}" target="_blank" rel="noopener noreferrer">{link_text}</a>"#
            )
            .unwrap();
            link_href.clear();
            link_text.clear();
        } else if is_reading_href {
            link_href.push(c);
        } else {
            parsed_line.push(c);
        }
    }
    parsed_line
}

fn parse_markdown(text: &str) -> String {
    let mut parsed_markdown = String::new();

    let regex = Regex::new(r"```(\w*)").unwrap();
    let mut is_within_code_block = false;
    let mut is_within_paragraph = false;

    let should_end_paragraph = move || {
        if is_within_paragraph {
            "</p>"
        } else {
            ""
        }
    };

    text.lines().for_each(|line| {
        if is_within_code_block {
            if line == "```" {
                is_within_code_block = false;
                parsed_markdown.push_str("</code></pre>");
            } else {
                parsed_markdown.push_str(line);
            }
        } else {
            let heading_level = get_level_of_heading(line);
            if heading_level == 0 {
                let code_block_capture = regex.captures(line);
                if let Some(capture) = code_block_capture {
                    is_within_code_block = true;
                    write!(
                        parsed_markdown,
                        "{}<pre><code class='language-{}'>",
                        should_end_paragraph(),
                        capture.get(1).unwrap().as_str()
                    )
                    .unwrap();
                } else if !is_within_paragraph && !line.trim().is_empty() {
                    is_within_paragraph = true;
                    write!(parsed_markdown, "<p>{}", parse_line(line)).unwrap();
                } else if is_within_paragraph && line.trim().is_empty() {
                    is_within_paragraph = false;
                    parsed_markdown.push_str("</p>");
                } else {
                    parsed_markdown.push_str(&parse_line(line));
                }
            } else {
                write!(
                    parsed_markdown,
                    "{}<h{heading_level}>{}</h{heading_level}>",
                    should_end_paragraph(),
                    parse_line(&line[(heading_level as usize) + 1..])
                )
                .unwrap();
            }
            parsed_markdown.push('\n')
        }
    });
    parsed_markdown
}

fn get_level_of_heading(heading: &str) -> u8 {
    let value = heading.chars().try_fold(0, |acc, c| {
        if c == '#' {
            if acc > 5 {
                ControlFlow::Break(0)
            } else {
                ControlFlow::Continue(acc + 1)
            }
        } else {
            ControlFlow::Break(acc)
        }
    });
    match value {
        ControlFlow::Continue(heading_level) => heading_level,
        ControlFlow::Break(heading_level) => heading_level,
    }
}

fn md2html<P: AsRef<path::Path>>(input_md: P, base_html: P, output_html: P) -> io::Result<()> {
    let parsed_body = parse_markdown(&fs::read_to_string(input_md)?);
    let base_html = fs::read_to_string(base_html)?;

    let mut output_file = fs::File::create(output_html)?;
    let parsed_html = base_html.replace("{template}", &parsed_body);

    write!(output_file, "{parsed_html}")
}

fn main() {
    md2html("./in/input.md", "./out/base.html", "./out/parsed.html").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line() {
        let test_line = "The `question` ends up being `*_why_*` not *how*";

        let expected =
            "The <code>question</code> ends up being <code><b><i>why</i></b></code> not <b>how</b>";
        let actual = parse_line(test_line);

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_get_level_of_heading_returns_3() {
        let test_line = "### Level #3";

        let expected = 3;
        let actual = get_level_of_heading(test_line);

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_get_level_of_heading_returns_5() {
        let test_line = "###### Level ###6";

        let expected = 6;
        let actual = get_level_of_heading(test_line);

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_get_level_of_heading_returns_0() {
        let test_line = "Level #3";

        let expected = 0;
        let actual = get_level_of_heading(test_line);

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_get_level_of_heading_returns_0_when_more_than_6() {
        let test_line = "####### Overflow";

        let expected = 0;
        let actual = get_level_of_heading(test_line);

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_parse_text() {
        let test_line = include_str!("../in/test/input.md");
        let expected = include_str!("../out/test/expected.html");
        let actual = parse_markdown(test_line);

        assert_eq!(expected, actual)
    }
}
