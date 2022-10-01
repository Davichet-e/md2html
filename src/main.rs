use regex::Regex;
use std::ops::ControlFlow;
use std::io::Write;

fn parse_line(line: &str) -> String {
    let mut parsed_line = String::new();
    let mut bold = false;
    let mut italic = false;
    let mut inline_code = false;

    macro_rules! parse {
        ($variable_name:ident, $tag:literal) => {
            if $variable_name {
                parsed_line.push_str(&format!("</{}>", $tag));
                $variable_name = false;
            } else {
                parsed_line.push_str(&format!("<{}>", $tag));
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
            ControlFlow::Continue(acc + 1)
        } else {
            ControlFlow::Break(acc)
        }
    });
    match value {
        ControlFlow::Continue(heading_level) => heading_level,
        ControlFlow::Break(heading_level) => heading_level,
    }
}

fn md2html<P: AsRef<std::path::Path>>(input_md: P, base_html: P, output_html: P) {
    let parsed_body = parse_text(&std::fs::read_to_string(input_md).unwrap());
    let base_html = std::fs::read_to_string(base_html).unwrap();
    let parsed_html = str::replace(&base_html, "{template}", &parsed_body);
    let mut output_file = std::fs::File::create(output_html).unwrap();
    write!(output_file, "{parsed_html}");
}

fn main() {
    let test_line = "The `question` ends up being `*_why_*` not *how*";
    println!("{}", parse_line(test_line));
    println!("{}", get_level_of_heading("### line"));
    println!("{}", get_level_of_heading("sentenc#e"));
    println!("{}", parse_text(include_str!("../in/input.md")));
    md2html("./in/input.md", "./out/base.html", "./out/parsed.html");
}
