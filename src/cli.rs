use arg::Args;

#[derive(Debug)]
pub struct App(pub u64);

impl core::str::FromStr for App {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        const CANNOT_FIND: &str = "Cannot find application by provided name";
        const PS_SPACE: &[char] = &[' ', '\t'];
        use std::process::Command;

        let mut cmd = Command::new("adb");

        cmd.arg("shell").arg("ps");

        if let Ok(pid) = text.parse() {
            return Ok(App(pid));
        }

        let output = match cmd.output() {
            Ok(output) => output,
            Err(_) => {
                eprintln!("{}", CANNOT_FIND);
                return Err(());
            }
        };

        if output.status.success() {
            if let Ok(stdout) = core::str::from_utf8(&output.stdout) {
                let mut result = None;

                for line in stdout.lines() {
                    if line.contains(text) {
                        let mut parts = line.split(PS_SPACE);
                        //Skip user
                        parts.next();
                        let mut pid = None;
                        for part in parts {
                            let part = part.trim();
                            if !part.is_empty() {
                                pid = Some(part);
                                break;
                            }
                        }

                        if let Some(pid) = pid {
                            result = pid.parse().ok();
                            break;
                        }
                    }
                }

                if let Some(result) = result {
                    return Ok(result)
                }
            }
        }

        eprintln!("{}", CANNOT_FIND);
        Err(())
    }
}

#[derive(Args, Debug)]
///plogcat 1.0.0
///Colorful wrapper over adb logcat command
pub struct Cli {
    #[arg(long)]
    ///Filter output by currently running application. Used if `app` option is not provided.
    pub current: bool,

    #[arg(short, long)]
    ///Clears logcat content before running.
    pub clear: bool,

    #[arg(long)]
    ///Whether to include time. Default: false.
    pub time: bool,

    #[arg(long)]
    ///Specifies tag width. Default: 23.
    pub tag_width: usize,

    #[arg(short, long)]
    ///List of tags to include into output.
    pub tag: Vec<String>,

    #[arg(short, long)]
    ///List of tags to exclude from output.
    pub ignored_tag: Vec<String>,

    #[arg(long)]
    ///Package name or pid by which to filter logcat
    pub app: Option<App>,
}

impl Cli {
    pub fn set_current_app(&mut self) -> Result<(), u8> {
        use crate::errors::{DUMPSYS_FAIL, ADB_FAIL, UTF8_ERROR};

        let adb = std::process::Command::new("adb").arg("shell").arg("dumpsys").arg("activity").arg("activities").output();
        let output = match adb {
            Ok(output) => match output.status.success() {
                true => output,
                false => {
                    eprintln!("Unable to find current app in dympsys");
                    return Err(DUMPSYS_FAIL);
                },
            },
            Err(error) => {
                eprintln!("Failed to lookup current app: {}", error);
                return Err(ADB_FAIL);
            }
        };
        let output = match core::str::from_utf8(&output.stdout) {
            Ok(output) => output,
            Err(_) => {
                eprintln!("stdout output is not UTF-8");
                return Err(UTF8_ERROR);
            }
        };

        let regex = regex::Regex::new(".*TaskRecord.*A[= ]([^ ^}]*)").unwrap();
        match regex.captures(output) {
            Some(caps) => match caps.get(1) {
                Some(cap) => match core::str::FromStr::from_str(cap.as_str()) {
                    Ok(app) => {
                        self.app = Some(app);
                    },
                    Err(_) => {
                        println!(">Cannot find pid of currently running app '{}'", cap.as_str());
                    },
                },
                None => {
                    println!(">No app currently running");
                }
            },
            None => {
                println!(">No app currently running");
            }
        }

        Ok(())
    }
}

pub fn new(args: &c_ffi::Args) -> Result<Cli, u8> {
    match Cli::from_args(args.into_iter().skip(1)) {
        Ok(args) => Ok(args),
        Err(arg::ParseError::HelpRequested(help)) => {
            println!("{}", help);
            Err(0)
        },
        Err(error) => {
            eprintln!("{}", error);
            Err(1)
        }
    }
}
