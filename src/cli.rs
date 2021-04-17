use arg::Args;

#[derive(Debug)]
pub struct Level(char);

impl core::str::FromStr for Level {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.eq_ignore_ascii_case("v") || text.eq_ignore_ascii_case("verbose") {
            Ok(Level('V'))
        } else if text.eq_ignore_ascii_case("d") || text.eq_ignore_ascii_case("debug") {
            Ok(Level('D'))
        } else if text.eq_ignore_ascii_case("i") || text.eq_ignore_ascii_case("info") {
            Ok(Level('I'))
        } else if text.eq_ignore_ascii_case("w") || text.eq_ignore_ascii_case("warning") {
            Ok(Level('W'))
        } else if text.eq_ignore_ascii_case("e") || text.eq_ignore_ascii_case("error") {
            Ok(Level('E'))
        } else if text.eq_ignore_ascii_case("f") || text.eq_ignore_ascii_case("fatal") {
            Ok(Level('F'))
        } else {
            Err(())
        }
    }
}

impl Default for Level {
    #[inline(always)]
    fn default() -> Self {
        Level('V')
    }
}

#[derive(Debug)]
pub enum App {
    Pid(u64),
    PackageName(String),
}

impl core::str::FromStr for App {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if let Ok(pid) = text.parse() {
            Ok(App::Pid(pid))
        } else {
            return Ok(App::PackageName(text.to_owned()))
        }
    }
}

#[derive(Args, Debug)]
///plogcat 1.0.0
///Colorful wrapper over adb logcat command
pub struct Cli {
    #[arg(short, long)]
    ///Load an alternate log buffer.
    pub buffer: Vec<String>,

    #[arg(long)]
    ///Filter output by currently running application. Used if `app` option is not provided.
    pub current: bool,

    #[arg(short, long)]
    ///Clears logcat content before running.
    pub clear: bool,

    #[arg(short, long)]
    ///Dumps current log and exits.
    pub dump: bool,

    #[arg(long)]
    ///Whether to include time. Default: false.
    pub time: bool,

    #[arg(long)]
    ///Specifies tag width. Default: 23.
    pub tag_width: usize,

    #[arg(short, long)]
    ///List of tags to include into output.
    pub tag: Vec<String>,

    #[arg(short, long, default_value)]
    ///Specifies minimum Android log level to include. Default Verbose.
    pub level: Level,

    #[arg(short, long)]
    ///Tells to print only number of lines, before exiting.
    pub max_count: Option<core::num::NonZeroU64>,

    #[arg(short, long)]
    ///Specifies device's serial number.
    pub serial: Option<String>,

    #[arg(short)]
    ///Specifies to use USB connected device.
    pub device: bool,

    #[arg(short)]
    ///Specifies to use TCP/IP connected device.
    pub emulator: bool,

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

    pub fn get_app_pid(&self) -> Result<Option<u64>, u8> {
        const CANNOT_FIND: &str = "Cannot find application by provided name";
        const PS_SPACE: &[char] = &[' ', '\t'];

        let name = match self.app.as_ref() {
            Some(App::Pid(pid)) => return Ok(Some(*pid)),
            None => return Ok(None),
            Some(App::PackageName(name)) => name.as_str(),
        };

        let mut cmd = self.get_adb_cmd();

        cmd.arg("shell").arg("ps");

        let output = match cmd.output() {
            Ok(output) => output,
            Err(_) => {
                eprintln!("{}", CANNOT_FIND);
                return Err(crate::errors::ADB_FAIL);
            }
        };

        if output.status.success() {
            if let Ok(stdout) = core::str::from_utf8(&output.stdout) {
                let mut result = None;

                for line in stdout.lines() {
                    if line.contains(name) {
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
                    return Ok(Some(result))
                }
            }
        }

        eprintln!("{}", CANNOT_FIND);
        Err(1)
    }

    pub fn get_adb_cmd(&self) -> std::process::Command {
        let mut adb = std::process::Command::new("adb");

        if let Some(serial) = self.serial.as_ref() {
            adb.arg("-s");
            adb.arg(serial);
        }

        if self.device {
            adb.arg("-d");
        }

        if self.emulator {
            adb.arg("-e");
        }

        adb
    }

    pub fn get_logcat_cmd(&self) -> std::process::Command {
        let mut adb = self.get_adb_cmd();
        adb.arg("logcat");

        for buffer in self.buffer.iter() {
            adb.arg("-b");
            adb.arg(&buffer);
        }

        if let Some(max_count) = self.max_count {
            adb.arg("-m");
            adb.arg(&format!("{}", max_count));
        }

        if self.dump {
            adb.arg("-d");
        }

        adb
    }

    pub fn get_filter_spec(&self) -> Option<String> {
        if self.level.0 == 'V' {
            None
        } else {
            Some(format!("*:{}", self.level.0))
        }
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
