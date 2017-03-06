extern crate pulldown_cmark as md;
extern crate cursive_break;
use cursive_break::utils::LinesIterator;

enum TextWrapping {
    WrapText {
        columns: u32,
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
                columns: 80,
            }
        }
    }
}

impl Config {
    pub fn with_line_wrapping_after(mut self, columns: u32) -> Self {
        self.text_wrapping = TextWrapping::WrapText { columns: columns };
        self
    }
    pub fn without_line_wrapping(mut self) -> Self {
        self.text_wrapping = TextWrapping::NoWrapping;
        self
    }
}

fn push_txt<'a, I: Iterator<Item = md::Event<'a>>>(buf: &mut String, iter: I, config: &Config) {
    use md::Event::*;
    use md::Tag::*;
    let mut linkctr = 1;
    let mut footer = String::new();
    let mut there_was_a_paragraph_already = false;
    let mut line_buffer = String::new();
    fn line_push(buf: &mut String, linebuf: &mut String, config: &Config) {
        use TextWrapping::*;
        match config.text_wrapping {
            WrapText{columns} => {
                let it = LinesIterator::new(linebuf, columns as usize);
                let mut first_row = true;
                for row in it {
                    assert!(row.width <= columns as usize);
                    if !first_row {
                        buf.push_str("\n");
                    }
                    first_row = false;
                    let slice = &linebuf[row.start..row.end];
                    buf.push_str(slice);
                }
            }
            NoWrapping => {
                buf.push_str(linebuf);
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
