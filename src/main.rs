#![no_main]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

use std::io::BufRead;

pub use plogcat::*;

c_ffi::c_main!(run);

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

    let color_choice = match args.machine {
        false => termcolor::ColorChoice::Auto,
        true => termcolor::ColorChoice::Never,
    };
    let term = termcolor::StandardStream::stdout(color_choice);

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

    let mut plogcat = Plogcat::new(term.lock(), args.tag_width, args.time);
    plogcat.tag_exclude = args.ignored_tag.iter().map(String::as_ref).collect();
    plogcat.tag_include = args.tag.iter().map(String::as_ref).collect();

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

        plogcat.handle_line(&line);
    }
}
