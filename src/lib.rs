#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

pub mod cli;
pub mod errors;
pub mod color;
mod parser;
pub use parser::{parse, LogCatLine};

use std::collections::HashSet;
use std::io::Write;
use termcolor::WriteColor;

const OUTPUT_SEP: &str = " ";
//const TIME_LEN: usize = 18; //"04-16 15:39:59.337"
const TIME_LEN: usize = 12; //"15:39:59.337"

pub struct Plogcat<'a> {
    buffer: String,
    term: termcolor::StandardStreamLock<'a>,
    include_time: bool,
    header_size: usize,
    tag_colors: color::Stack,
    ///Max possible space to allocate for printing tag.
    pub tag_width: usize,
    ///By default automatically calculated from current console width.
    pub term_width: usize,
    ///By default none, which means include all.
    pub tag_include: HashSet<&'a str>,
    ///By default none, which means exclude none.
    pub tag_exclude: HashSet<&'a str>,
}

impl<'a> Plogcat<'a> {
    pub fn new(term: termcolor::StandardStreamLock<'a>, tag_width: usize, include_time: bool) -> Self {
        let term_width = match term_size::dimensions() {
            Some((width, _)) => width,
            None => 0,
        };

        //                    tag + spaces + level + spaces
        let mut header_size = tag_width + 2 + 1 + 2;
        if include_time {
            header_size += TIME_LEN + 3 //time + brackets with space
        }

        Self {
            buffer: String::new(),
            term,
            include_time,
            header_size,
            term_width,
            tag_width,
            tag_colors: color::Stack::new(),
            tag_exclude: HashSet::new(),
            tag_include: HashSet::new(),
        }
    }

    pub fn handle_line(&mut self, line: &str) {
        if line.contains("nativeGetEnabledTags") {
            return;
        }

        let LogCatLine { time, level, tag, msg, .. } = match parse(&line) {
            Some(line) => line,
            None => return,
        };

        if !self.tag_exclude.is_empty() {
            if self.tag_exclude.contains(tag) {
                return;
            }
        }

        if !self.tag_include.is_empty() {
            if !self.tag_include.contains(tag) {
                return;
            }
        }

        let mut tag_color = termcolor::ColorSpec::new();
        tag_color.set_fg(Some(self.tag_colors.get_color(tag)));

        let _ = self.term.set_color(&tag_color);
        let _ = write!(&mut self.term, "{:>width$}", tag, width=self.tag_width);
        let _ = self.term.reset();

        let _ = write!(&mut self.term, "{}", OUTPUT_SEP);

        let mut level_color = termcolor::ColorSpec::new();

        match level {
            "V" => {
                level_color.set_fg(Some(termcolor::Color::White));
                level_color.set_bg(Some(termcolor::Color::Black));
            },
            "D" => {
                level_color.set_fg(Some(termcolor::Color::Black));
                level_color.set_bg(Some(termcolor::Color::Blue));
            },
            "I" => {
                level_color.set_fg(Some(termcolor::Color::Black));
                level_color.set_bg(Some(termcolor::Color::Green));
            },
            "W" => {
                level_color.set_fg(Some(termcolor::Color::Black));
                level_color.set_bg(Some(termcolor::Color::Yellow));
            },
            "E" | "F" => {
                level_color.set_fg(Some(termcolor::Color::Black));
                level_color.set_bg(Some(termcolor::Color::Red));
            },
            _ => (),
        }

        if self.include_time {
            let _ = write!(&mut self.term, "[{:width$}]", time, width=TIME_LEN);
            let _ = write!(&mut self.term, "{}", OUTPUT_SEP);
        }

        let _ = self.term.set_color(&level_color);
        let _ = write!(&mut self.term, "{}", OUTPUT_SEP);
        let _ = write!(&mut self.term, "{}", level);
        let _ = write!(&mut self.term, "{}", OUTPUT_SEP);
        let _ = self.term.reset();

        let _ = write!(&mut self.term, "{}", OUTPUT_SEP);

        if self.term_width < self.header_size {
            let _ = write!(&mut self.term, "{}\n", msg);
        } else {
            let wrap_area = self.term_width - self.header_size;
            let mut msg_len = msg.chars().map(|ch| ch.len_utf8()).sum();
            let mut msg = msg.chars();

            let mut last_char: Option<char> = None;
            loop {
                let mut consumed_len = 0;
                let chunk_len = core::cmp::min(msg_len, wrap_area);

                if let Some(ch) = last_char.take() {
                    consumed_len += ch.len_utf8();
                    self.buffer.push(ch);
                }

                while let Some(ch) = msg.next() {
                    //Take into account that font can take up to byte len of character
                    //so that we wouldn't overflow with fat wide characters
                    if (consumed_len + ch.len_utf8()) <= chunk_len {
                        self.buffer.push(ch);
                        consumed_len += ch.len_utf8();

                        if consumed_len == chunk_len {
                            break;
                        }
                    } else {
                        last_char = Some(ch);
                        break;
                    }
                }

                msg_len = msg_len.saturating_sub(consumed_len);

                if msg_len > 0 {
                    self.buffer.push('\n');
                    for _ in 0..self.header_size {
                        self.buffer.push(' ');
                    }
                } else {
                    break;
                }
            }

            let _ = write!(&mut self.term, "{}\n", self.buffer);
            self.buffer.clear();
        }

    }
}
