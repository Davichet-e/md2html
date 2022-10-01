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
        } else {
            parsed_line.push(c);
        }
    }
    parsed_line
}

fn parse_text(text: &str) -> String {
    let regex = Regex::new(r"```(\w*)").unwrap();
    let mut is_within_code_block = false;

    let lines_parsed = text.lines().map(|line| {
        if is_within_code_block {
            if line == "```" {
                is_within_code_block = false;
                "</code></pre>".to_string()
            } else {
                line.to_string()
            }
        } else {
            let heading_level = get_level_of_heading(line);
            if heading_level == 0 {
                let code_block_capture = regex.captures(line);
                if code_block_capture.is_some() {
                    is_within_code_block = true;
                    format!(
                        "<pre><code class='language-{}'>",
                        code_block_capture.unwrap().get(1).unwrap().as_str()
                    )
                } else {
                    parse_line(line)
                }
            } else {
                format!(
                    "<h{heading_level}>{}</h{heading_level}>",
                    parse_line(&line[(heading_level as usize) + 1..])
                )
            }
        }
    });
    lines_parsed.collect::<Vec<String>>().join("\n")
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
    let parsed_body = parse_text(&fs::read_to_string(input_md)?);
    let base_html = fs::read_to_string(base_html)?;
    let parsed_html = str::replace(&base_html, "{template}", &parsed_body);
    let mut output_file = fs::File::create(output_html)?;
    write!(output_file, "{parsed_html}")?;
    Ok(())
}

fn main() {
    let test_line = "The `question` ends up being `*_why_*` not *how*";
    println!("{}", parse_line(test_line));
    println!("{}", get_level_of_heading("### line"));
    println!("{}", get_level_of_heading("sentenc#e"));
    println!("{}", parse_text(include_str!("../in/input.md")));
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
        let test_line = r#"
# Learning AWS

## Day 1

Here is the code I found:

```python
def h():
    return "bye"
```

The question ends up being _why_ not *how*

"#;

        let expected = r#"
<h1>Learning AWS</h1>

<h2>Day 1</h2>

Here is the code I found:

<pre><code class='language-python'>
def h():
    return "bye"
</code></pre>

The question ends up being <i>why</i> not <b>how</b>
"#;
        let actual = parse_text(test_line);

        assert_eq!(expected, actual)
    }
}
