extern crate pulldown_cmark as md;
extern crate unicode_segmentation;
use unicode_segmentation::UnicodeSegmentation;
extern crate xi_unicode;
use xi_unicode::LineBreakIterator;

enum TextBreakRule {
    BreakAtWhitespace,
    BreakAnywhere,
}

enum TextWrapping {
    WrapText {
        text_break_rule: TextBreakRule,
        columns: u32,
        /// Wrap single words longer than a line into multiple lines, breaking anywhere, independent of the text_break_rule
        enforce_max_columns: bool
    },
    NoWrapping,
}

pub struct Config {
    text_wrapping: TextWrapping
}

impl Default for Config {
    fn default() -> Config {
        Config {
            text_wrapping: TextWrapping::WrapText {
                text_break_rule: TextBreakRule::BreakAtWhitespace,
                columns: 80,
                enforce_max_columns: false,
            }
        }
    }
}

fn push_txt<'a, I: Iterator<Item = md::Event<'a>>>(buf: &mut String, mut iter: I, config: &Config) {
    use md::Event::*;
    use md::Tag::*;
    let mut linkctr = 1;
    let mut footer = String::new();
    let mut there_was_a_paragraph_already = false;
    let mut line_buffer = String::new();
    fn split_by_hard_breaks(s: &str) -> Vec<&str> {
        let mut rest = s;
        let mut this = "";
        let mut ret = Vec::new();
        while let Some(i) = LineBreakIterator::new(rest).filter(|b| b.1).map(|b| b.0).next() {
            let t = rest.split_at(i);
            rest = t.1;
            this = t.0;
            ret.push(this);
        }
        ret
    }
    fn line_push(buf: &mut String, linebuf: &mut String, config: &Config) {
        {
            // In this block, linebuf is immutable.
            let actual_lines = split_by_hard_breaks(linebuf);
            for line in actual_lines.iter() {
                match config.text_wrapping {
                    TextWrapping::WrapText { ref columns, ref text_break_rule, ref enforce_max_columns } => {
                        let clusters = UnicodeSegmentation::graphemes(*line, true).collect::<Vec<_>>();
                        if clusters.len() <= *columns as usize {
                            buf.push_str(line);
                        } else {
                            match *text_break_rule {
                                TextBreakRule::BreakAnywhere => {
                                    for (i,s) in clusters.iter().enumerate() {
                                        if i % (*columns as usize) == 0 && i>0 {
                                            buf.push_str("\n");
                                        }
                                        buf.push_str(s);
                                    }
                                }
                                TextBreakRule::BreakAtWhitespace => {
                                    unimplemented!()
                                }
                            }
                        }
                    }
                    TextWrapping::NoWrapping => {
                        buf.push_str(line);
                    }
                }
            }
        }
        linebuf.clear();
    }
    for event in iter {
        match event {
            Start(tag) => {
                match tag {
                    Item => line_buffer.push_str("* "),
                    Paragraph => {
                        if there_was_a_paragraph_already {
                            line_push(buf, &mut line_buffer, config);
                            buf.push_str("\n\n");
                        }
                        there_was_a_paragraph_already = true;
                    }
                    _ => (), // ignore other tags
                }
            }
            End(tag) => {
                match tag {
                    Link(url, title) => {
                        let currentnum = linkctr;
                        linkctr += 1;
                        line_buffer.push_str(&format!("[{}]", currentnum));
                        // We use a non-breaking space, so we won't line-break between the number and the link
                        if title.len() > 0 {
                            footer.push_str(&format!("[{}]\u{00A0}{} {}", currentnum, title, url));
                        } else {
                            footer.push_str(&format!("[{}]\u{00A0}{}", currentnum, url));
                        }
                        footer.push('\n');
                    }
                    Item => {
                        line_buffer.push('\n');
                        line_push(buf, &mut line_buffer, config);
                    }
                    List(_) => {
                        line_buffer.push('\n'); // looks cleaner
                        line_push(buf, &mut line_buffer, config);
                    }
                    _ => (), // ignore other tags
                }
            }
            Text(text) => line_buffer.push_str(&text),
            Html(_) => unimplemented!(),
            InlineHtml(_) => unimplemented!(),
            SoftBreak => line_buffer.push(' '),
            HardBreak => {
                line_buffer.push('\n');
                line_push(buf, &mut line_buffer, config);
            }
            FootnoteReference(_) => {}
        }
    }
    line_push(buf, &mut line_buffer, config);
    if !footer.is_empty() {
        buf.push_str("\n\n");
    }
    buf.push_str(footer.trim());
}

pub fn markdown_to_plaintext<'a>(markdown: &'a str, config: &Config) -> String {
    let mut ret = String::new();
    let parser = md::Parser::new(markdown);
    push_txt(&mut ret, parser, config);
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple() {
        let md = "Dies ist ein Test.";
        let txt = markdown_to_plaintext(md, &Config::default());
        assert_eq!(txt, md);
    }
    #[test]
    fn link() {
        let md = "Dies ist ein [Link](http://example.com).";
        let txt = markdown_to_plaintext(md, &Config::default());
        assert_eq!(txt, "Dies ist ein Link[1].\n\n[1]\u{00A0}http://example.com");
    }
    #[test]
    fn break_anywhere() {
        println!("break anywhere test starting up");
        let cols = 10;
        let strlen = 24;
        let md = std::iter::repeat('x').take(strlen).collect::<String>();
        let mut cfg = Config::default();
        cfg.text_wrapping = TextWrapping::WrapText {
            text_break_rule: TextBreakRule::BreakAnywhere,
            columns: cols as u32,
            enforce_max_columns: true,
        };
        let txt = markdown_to_plaintext(&md, &cfg);
        let expected = "xxxxxxxxxx\nxxxxxxxxxx\nxxxx";
        assert_eq!(txt, expected);
    }
}
