#![no_main]

use std::io::{Write, BufRead};
use termcolor::WriteColor;

pub use plogcat::*;

c_ffi::c_main!(run);

const OUTPUT_SEP: &str = " ";
const TIME_LEN: usize = 18; //"04-16 15:39:59.337"

fn run(args: c_ffi::Args) -> u8 {
    let mut args = match cli::new(&args) {
        Ok(args) => args,
        Err(res) => return res,
    };

    if args.tag_width == 0 {
        args.tag_width = 23;
    }

    let mut adb = args.get_logcat_cmd();

    if args.app.is_none() && args.current {
        println!(">Pid not specified, find currently run app");
        if let Err(error) = args.set_current_app() {
            return error
        }
    }

    match args.get_app_pid() {
        Ok(Some(pid)) => {
            println!(">Filtering by pid {}", pid);
            adb.arg(&format!("--pid={}", pid));
        },
        Ok(None) => (),
        Err(err) => return err,
    }

    adb.stdout(std::process::Stdio::piped());
    adb.arg("-v");
    adb.arg("time");

    if args.clear {
        let mut adb = args.get_logcat_cmd();
        adb.arg("-c");


        if let Err(error) = adb.status() {
            eprintln!("Failed to execute adb -c: {}", error);
            return errors::ADB_FAIL
        }
    }

    let tag_include = if args.tag.is_empty() {
        None
    } else {
        match regex::from_tag_list(&args.tag) {
            Ok(regex) => Some(regex),
            Err(error) => {
                eprintln!("Cannot compile regex for includeed tags: {}", error);
                return errors::INTERNAL;
            }
        }
    };

    let tag_exclude = if args.ignored_tag.is_empty() {
        None
    } else {
        match regex::from_tag_list(&args.ignored_tag) {
            Ok(regex) => Some(regex),
            Err(error) => {
                eprintln!("Cannot compile regex for ignored tags: {}", error);
                return errors::INTERNAL;
            }
        }
    };

    let log_re = regex::Regex::new(r#"^([0-9]+-[0-9]+\s[0-9]+:[0-9]+:[0-9]+.[0-9]+)\s([A-Z])/(.+?)\( *(\d+)\): (.*?)$"#).unwrap();
    let color_choice = match args.machine {
        false => termcolor::ColorChoice::Auto,
        true => termcolor::ColorChoice::Never,
    };
    let term = termcolor::StandardStream::stdout(color_choice);
    let term_width = match term_size::dimensions() {
        Some((width, _)) => width,
        None => 0,
    };

    if let Some(filter) = args.get_filter_spec() {
        adb.arg(&filter);
    }

    let adb = match adb.spawn() {
        Ok(adb) => adb,
        Err(error) => {
            eprintln!("Failed to start adb: {}", error);
            return errors::ADB_FAIL;
        }
    };

    let mut adb = scope_guard::scope_guard!(|mut adb| {
        if let Err(error) = adb.kill() {
            eprintln!("Failed to kill adb: {}", error);
        }
    }, adb);

    let stdout = match adb.stdout.take() {
        Some(stdout) => stdout,
        None => {
            eprintln!("Stdout pipe is not available");
            return 0;
        }
    };
    let mut stdout = std::io::BufReader::new(stdout);
    let mut term = term.lock();

    //                    tag + spaces + level
    let mut header_size = args.tag_width + 2 + 1;
    if args.time {
        header_size += TIME_LEN + 3 //time + brackets with space
    }

    let mut tag_colors = color::Stack::new();

    let mut msg_buffer = String::new();
    let mut line = String::new();
    loop {
        match adb.try_wait() {
            Ok(Some(status)) => {
                adb.into_inner();
                break match status.success() {
                    true => 0,
                    false => errors::ADB_FAIL,
                };
            },
            Ok(None) => (),
            Err(error) => {
                eprintln!("Cannot poll adb process: {}", error);
                break errors::ADB_FAIL;
            }
        }

        line.clear();
        if let Err(error) = stdout.read_line(&mut line) {
            eprintln!("Failed to read={}", error);
        }

        if line.contains("nativeGetEnabledTags") {
            continue;
        }
        let caps = match log_re.captures(&line.trim()) {
            Some(caps) if caps.len() == 6 => caps,
            Some(_) | None => continue,
        };

        let tag = caps.get(3).unwrap().as_str().trim();

        if let Some(regex) = tag_exclude.as_ref() {
            if regex.is_match(tag) {
                continue;
            }
        }

        if let Some(regex) = tag_include.as_ref() {
            if !regex.is_match(tag) {
                continue;
            }
        }

        let time = caps.get(1).unwrap().as_str();
        let level = caps.get(2).unwrap().as_str();
        //let pid = caps.get(4).unwrap().as_str();
        let msg = caps.get(5).unwrap().as_str();

        let mut tag_color = termcolor::ColorSpec::new();
        tag_color.set_fg(Some(tag_colors.get_color(tag)));

        let _ = term.set_color(&tag_color);
        let _ = write!(&mut term, "{:width$}", tag, width=args.tag_width);
        let _ = term.reset();

        let _ = write!(&mut term, "{}", OUTPUT_SEP);

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

        if args.time {
            let _ = write!(&mut term, "[{:width$}]", time, width=TIME_LEN);
            let _ = write!(&mut term, "{}", OUTPUT_SEP);
        }

        let _ = term.set_color(&level_color);
        let _ = write!(&mut term, "{}", level);
        let _ = term.reset();
        let _ = write!(&mut term, "{}", OUTPUT_SEP);

        if term_width < header_size {
            let _ = write!(&mut term, "{}\n", msg);
        } else {
            let wrap_area = term_width - header_size;
            let mut msg_len = msg.chars().map(|ch| ch.len_utf8()).sum();
            let mut msg = msg.chars();

            let mut last_char: Option<char> = None;
            loop {
                let mut consumed_len = 0;
                let chunk_len = core::cmp::min(msg_len, wrap_area);

                if let Some(ch) = last_char.take() {
                    consumed_len += ch.len_utf8();
                    msg_buffer.push(ch);
                }

                while let Some(ch) = msg.next() {
                    //Take into account that font can take up to byte len of character
                    //so that we wouldn't overflow with fat wide characters
                    if (consumed_len + ch.len_utf8()) <= chunk_len {
                        msg_buffer.push(ch);
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
                    msg_buffer.push('\n');
                    for _ in 0..header_size {
                        msg_buffer.push(' ');
                    }
                } else {
                    break;
                }
            }

            let _ = write!(&mut term, "{}\n", msg_buffer);
            msg_buffer.clear();
        }
    }

    //0
}
