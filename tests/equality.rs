extern crate markdown_to_plaintext;

use markdown_to_plaintext::*;

// A wrapper around Config so we can implement traits on it for convenience
struct MyCfg {
    c: Config
}

impl From<Config> for MyCfg {
    fn from(x: Config) -> MyCfg {
        MyCfg { c: x }
    }
}

impl Into<Config> for MyCfg {
    fn into(self) -> Config {
        self.c
    }
}

impl From<Option<u32>> for MyCfg {
    fn from(x: Option<u32>) -> MyCfg {
        let base = Config::default();
        MyCfg { c: match x {
                Some(cols) => base.with_line_wrapping_after(cols),
                None => base.without_line_wrapping(),
            }
        }
    }
}
fn eq_test<C: Into<MyCfg>>(md: &str, expected_txt: &str, config: C) {
    // The first `into` goes from C to MyCfg, the second from MyCfg to Config.
    let config : Config = config.into().into();
    assert_eq!(expected_txt, markdown_to_plaintext(md, &config));
}
#[test]
fn simple() {
    let s = "Dies ist ein Test.";
    eq_test(s, s, Config::default());
}
#[test]
fn link() {
    eq_test("Dies ist ein [Link](http://example.com).",
        "Dies ist ein Link[1].\n\n[1]\u{00A0}http://example.com",
        Config::default());
}
#[test]
fn regular_break() {
    eq_test("Lorem Ipsum Dolor Sit",
        "Lorem Ipsum\nDolor Sit",
        Some(11));
}
#[test]
fn break_anywhere() {
    eq_test("xxxxxxxxxxxxxxxxxxxxxxxx",
        "xxxxxxxxxx\nxxxxxxxxxx\nxxxx",
        Some(10));
}
#[test]
fn no_wrap() {
    let s : String = "word ".chars().cycle().take(500).collect();
    eq_test(&s, &s, None); // None => NoWrapping
}
