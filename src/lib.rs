extern crate pulldown_cmark as md;
extern crate cursive_break;
use cursive_break::utils::LinesIterator;
extern crate unicode_width;
use unicode_width::UnicodeWidthStr;

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

struct PrefixManager {
    quote_level: u32
}

impl PrefixManager {
    fn new() -> PrefixManager {
        PrefixManager {
            quote_level: 0,
        }
    }
    fn start_quote(&mut self) {
        self.quote_level += 1;
    }
    fn end_quote(&mut self) {
        self.quote_level -= 1;
    }
    fn get_prefix(&self) -> String {
        if self.quote_level > 0 {
            std::iter::repeat('>').take(self.quote_level as usize).chain(" ".chars()).collect()
        } else {
            String::new()
        }
    }
}

fn push_txt<'a, I: Iterator<Item = md::Event<'a>>>(buf: &mut String, iter: I, config: &Config) {
    use md::Event::*;
    use md::Tag::*;
    let mut linkctr = 1;
    let mut footer = String::new();
    let mut there_was_a_paragraph_already = false;
    let mut line_buffer = String::new();
    let mut prefix_manager = PrefixManager::new();
    fn line_push(buf: &mut String, linebuf: &mut String, config: &Config, prefix_manager: &PrefixManager) {
        use TextWrapping::*;
        let prefix = prefix_manager.get_prefix();
        match config.text_wrapping {
            WrapText{columns} => {
                let columns_left = columns.saturating_sub(UnicodeWidthStr::width(prefix.as_str()) as u32);
                // FIXME If the columns_left is actually zero, the LinesIterator will not work properly.
                let it = LinesIterator::new(linebuf, columns_left as usize);
                let mut first_row = true;
                for row in it {
                    assert!(row.width <= columns_left as usize);
                    if !first_row {
                        buf.push_str("\n");
                    }
                    first_row = false;
                    buf.push_str(&prefix);
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
                            line_push(buf, &mut line_buffer, config, &prefix_manager);
                            buf.push_str("\n\n");
                        }
                        there_was_a_paragraph_already = true;
                    }
                    BlockQuote => {
                        prefix_manager.start_quote();
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
                        line_push(buf, &mut line_buffer, config, &prefix_manager);
                    }
                    List(_) => {
                        line_buffer.push('\n'); // looks cleaner
                        line_push(buf, &mut line_buffer, config, &prefix_manager);
                    }
                    BlockQuote => {
                        line_push(buf, &mut line_buffer, config, &prefix_manager);
                        prefix_manager.end_quote();
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
                line_push(buf, &mut line_buffer, config, &prefix_manager);
            }
            FootnoteReference(_) => {}
        }
    }
    debug_assert_eq!(prefix_manager.get_prefix(), "");
    line_push(buf, &mut line_buffer, config, &prefix_manager);
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
