use grouping_by::GroupingBy;
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::Write;
use std::io::{self, Write as IOWrite};
use std::ops::ControlFlow;
use std::str::Lines;
use std::{fs, path};

pub struct Article {
    info: ArticleInfo,
    content_html: String,
}

pub struct ArticleInfo {
    title: String,
    date: String,
    description: String,
}

lazy_static! {
    static ref CODE_REGEX: Regex = Regex::new(r"```(\w*)").unwrap();
}

fn parse_line(line: &str) -> String {
    let mut parsed_line = String::new();
    let mut bold = false;
    let mut italic = false;
    let mut inline_code = false;

    let mut is_reading_link_text = false;
    let mut is_reading_href = false;
    let mut link_text = String::new();
    let mut link_href = String::new();

    fn parse(parsed_line: &mut String, variable: &mut bool, tag: &str) {
        if *variable {
            write!(parsed_line, "</{}>", tag).unwrap();
            *variable = false;
        } else {
            write!(parsed_line, "<{}>", tag).unwrap();
            *variable = true;
        }
    }
    line.chars().for_each(|c| match c {
        '_' => parse(&mut parsed_line, &mut italic, "i"),
        '*' => parse(&mut parsed_line, &mut bold, "b"),
        '`' => parse(&mut parsed_line, &mut inline_code, "code"),
        '[' if !is_reading_link_text => is_reading_link_text = true,
        ']' => is_reading_link_text = false,
        _ if is_reading_link_text => link_text.push(c),
        '(' if !link_text.is_empty() => is_reading_href = true,
        ')' if is_reading_href => {
            is_reading_href = false;
            write!(
                parsed_line,
                r#"<a href="{link_href}" target="_blank" rel="noopener noreferrer">{link_text}</a>"#
            )
            .unwrap();
            link_href.clear();
            link_text.clear();
        }
        _ if is_reading_href => link_href.push(c),
        _ => parsed_line.push(c),
    });
    parsed_line
}

fn parse_markdown(text: &str) -> Article {
    let mut parsed_markdown = String::new();

    let mut is_within_code_block = false;
    let mut is_within_paragraph = false;

    let should_end_paragraph = |is_within_paragraph| {
        if is_within_paragraph {
            "</p>"
        } else {
            ""
        }
    };

    let mut text_lines = text.lines();

    let title = text_lines.next().unwrap().strip_prefix("# ").unwrap();

    let date = text_lines
        .next()
        .unwrap()
        .strip_prefix("{date: ")
        .and_then(|date_str| date_str.strip_suffix('}'))
        .expect("Expected date format: `{date: YY/MM/dd}`");

    let description = text_lines
        .next()
        .unwrap()
        .strip_prefix("{description: ")
        .and_then(|date_str| date_str.strip_suffix('}'))
        .expect("Expected description format: `{description: <text>}`");

    parsed_markdown.push_str("<main>");

    writeln!(parsed_markdown, "<p>{description}</p>").unwrap();

    text_lines.for_each(|line| {
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
                let code_block_capture = CODE_REGEX.captures(line);
                if let Some(capture) = code_block_capture {
                    is_within_code_block = true;
                    write!(
                        parsed_markdown,
                        "{}<pre><code class='language-{}'>",
                        should_end_paragraph(is_within_paragraph),
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
                    should_end_paragraph(is_within_paragraph),
                    parse_line(&line[(heading_level as usize) + 1..])
                )
                .unwrap();
            }
        }
        parsed_markdown.push('\n');
    });
    parsed_markdown.push_str("</main>");
    Article {
        info: ArticleInfo {
            title: title.to_string(),
            date: date.to_string(),
            description: description.to_string(),
        },
        content_html: parsed_markdown,
    }
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

pub fn md2html<P: AsRef<path::Path>>(
    input_md: P,
    base_html: P,
    output_html: P,
) -> io::Result<ArticleInfo> {
    let parsed_article = parse_markdown(&fs::read_to_string(input_md)?);
    let base_html = fs::read_to_string(base_html)?;

    let mut output_file = fs::File::create(output_html)?;
    let parsed_html = base_html
        .replace("{template}", &parsed_article.content_html)
        .replace("{title}", &parsed_article.info.title)
        .replace("{date}", &parsed_article.info.date);

    write!(output_file, "{parsed_html}")?;

    Ok(parsed_article.info)
}

fn main() {
    // md2html("./in/test/input.md", "./out/base.html", "./out/parsed.html").unwrap();
    println!(
        "{}",
        parse_markdown(include_str!("../in/test/input.md")).content_html
    );
}

fn create_blog_list<P: AsRef<path::Path>>(
    blog_list_path: P,
    input_path: P,
    base_path: P,
) -> io::Result<()> {
    let mut file = fs::File::create(blog_list_path)?;
    let files = fs::read_dir(input_path)?.filter_map(|r| match r {
        Ok(entry) if entry.file_type().unwrap().is_file() => Some(entry.path()),
        _ => None,
    });
    for path in files {
        let filename = path.file_stem().unwrap().to_str().unwrap().to_string();
        let out_dir = base_path.as_ref().join("out/");
        let article = md2html(
            path.as_path(),
            out_dir.join("base.html").as_path(),
            out_dir.join(filename + ".html").as_path(),
        )?;
        writeln!(
            file,
            "title: {title}\ndate: {date}\ndescription: {description}\n\n",
            title = article.title,
            date = article.date,
            description = article.description
        )?;
    }
    Ok(())
}

pub fn read_blog_list(blog_list_path: impl AsRef<path::Path>) -> Vec<ArticleInfo> {
    let file = fs::read_to_string(blog_list_path).unwrap();
    file.split("\n\n\n")
        .map(|line_str| parse_article(line_str.lines()).unwrap())
        .collect()
}

fn parse_blog_list(articles: &[&ArticleInfo]) -> String {
    let mut blog_entries = String::new();
    for article in articles {
        writeln!(
            blog_entries,
            r#"
<div class="blog-entry">
    <h3 class="article-name">{title}</h3>
    <h4 class="article-date">{date}</h4>
    <p>
        {description}
    </p>
</div>"#,
            title = article.title,
            date = article.date,
            description = article.description
        )
        .unwrap();
    }
    blog_entries
}

fn parse_article(mut raw_article_lines: Lines) -> Option<ArticleInfo> {
    let title = raw_article_lines.next()?.strip_prefix("title: ")?;
    let date = raw_article_lines.next()?.strip_prefix("date: ")?;
    let description = raw_article_lines.next()?.strip_prefix("description: ")?;

    Some(ArticleInfo {
        title: title.to_string(),
        date: date.to_string(),
        description: description.to_string(),
    })
}

pub fn build_blog_entry_list(blog_list_path: impl AsRef<path::Path>) -> String {
    let articles = read_blog_list(blog_list_path);
    let articles_per_year = articles.iter().grouping_by(|article| &article.date[5..]);
    let articles_list_html: Vec<String> = articles_per_year
        .iter()
        .map(|(year, article_list)| {
            format!(
                r#"
        <h2 class="year">{year}</h2>
        {blog_entries}
        "#,
                blog_entries = parse_blog_list(article_list)
            )
        })
        .collect();

    articles_list_html.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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
    fn test_parse_markdown() {
        let test_line = include_str!("../in/test/input.md");
        let expected = include_str!("../out/test/parse_markdown-expected.html");
        let actual = parse_markdown(test_line);

        assert_eq!(expected, actual.content_html)
    }

    #[test]
    fn test_md2html() {
        let expected = include_str!("../out/test/md2html-expected.html");
        let output_html_path = "./out/test/md2html-received.html";
        let _ = md2html("./in/test/input.md", "./out/base.html", output_html_path);

        assert_eq!(expected, &fs::read_to_string(output_html_path).unwrap())
    }

    #[test]
    fn test_parse_blog_list() {
        // Given
        let date = "2022-08-23".to_string();
        let description = "This is a description of the article".to_string();
        let title = "Title of the article".to_string();
        let article_list = &[&ArticleInfo {
            date: date.clone(),
            description: description.clone(),
            title: title.clone(),
        }];

        // When
        let expected_html = parse_blog_list(article_list);

        // Then
        assert_eq!(
            expected_html,
            format!(
                r#"
<div class="blog-entry">
    <h3 class="article-name">{title}</h3>
    <h4 class="article-date">{date}</h4>
    <p>
        {description}
    </p>
</div>
"#,
            )
            .to_string()
        )
    }
}
