#![no_main]

use std::io::{Write, BufRead};
use termcolor::WriteColor;

mod cli;
mod errors;

c_ffi::c_main!(run);

const OUTPUT_SEP: &str = " ";

fn run(args: c_ffi::Args) -> u8 {
    let mut args = match cli::new(&args) {
        Ok(args) => args,
        Err(res) => return res,
    };

    if args.tag_width == 0 {
        args.tag_width = 23;
    }

    let mut adb = std::process::Command::new("adb");
    adb.arg("logcat");

    if args.app.is_none() && args.current {
        println!(">Pid not specified, find currently run app");
        if let Err(error) = args.set_current_app() {
            return error
        }
    }

    if let Some(pid) = args.app {
        println!(">Using pid {}", pid.0);
        adb.arg(&format!("--pid={}", pid.0));
    }

    adb.stdout(std::process::Stdio::piped());
    adb.arg("-v");
    adb.arg("brief");

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

    let term_width = match term_size::dimensions() {
        Some((width, _)) => width,
        None => 0,
    };

    let log_re = regex::Regex::new(r#"^([A-Z])/(.+?)\( *(\d+)\): (.*?)$"#).unwrap();
    let term = termcolor::StandardStream::stdout(termcolor::ColorChoice::Always);
    let mut term = term.lock();

    let header_size = args.tag_width + 3;
    let mut msg_buffer = String::new();
    let mut line = String::new();
    loop {
        line.clear();
        if let Err(error) = stdout.read_line(&mut line) {
            eprintln!("Failed to read={}", error);
        }

        if line.contains("nativeGetEnabledTags") {
            continue;
        }
        let caps = match log_re.captures(&line.trim()) {
            Some(caps) if caps.len() == 5 => caps,
            Some(_) | None => continue,
        };

        let level = caps.get(1).unwrap().as_str();
        let tag = caps.get(2).unwrap().as_str().trim();
        //let pid = caps.get(3).unwrap().as_str();
        let msg = caps.get(4).unwrap().as_str();

        let _ = write!(&mut term, "{:width$}", tag, width=args.tag_width);
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

        let _ = term.set_color(&level_color);
        let _ = write!(&mut term, "{}", level);
        let _ = term.reset();
        let _ = write!(&mut term, "{}", OUTPUT_SEP);

        if term_width < header_size {
            let _ = write!(&mut term, "{}\n", msg);
        } else {
            let wrap_area = term_width - header_size;
            let mut current = 0;
            let msg_len = msg.chars().count();
            let mut msg = msg.chars();

            while current < msg_len {
                let next = core::cmp::min(current + wrap_area, msg_len);
                for _ in 0..(next - current) {
                    msg_buffer.push(msg.next().unwrap());
                }

                if next < msg_len {
                    msg_buffer.push('\n');
                    for _ in 0..header_size {
                        msg_buffer.push(' ');
                    }
                }
                current = next;
            }

            let _ = write!(&mut term, "{}\n", msg_buffer);
            msg_buffer.clear();
        }
    }

    //0
}
