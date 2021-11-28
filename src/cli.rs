use arg::Args;

use core::convert::TryFrom;

const TIME_PARSE: &'static [time::format_description::FormatItem<'static>] = time::macros::format_description!("%H:%M:%S");
const DATETIME_PARSE: &'static [time::format_description::FormatItem<'static>] = time::macros::format_description!("%Y-%m-%d %H:%M:%S");

const FULL_DATETIME_FMT: &'static [time::format_description::FormatItem<'static>] = time::macros::format_description!("%Y-%m-%d %H:%M:%S.0");

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

#[derive(Debug)]
pub struct Time(pub time::OffsetDateTime);


impl Time {
    #[inline]
    fn duration_offset(diff: time::Duration) -> Self {
        let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
        Self(now - diff)
    }

    #[inline]
    fn time_offset(diff: time::Time) -> Self {
        let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
        let now_date = now.date();
        let now_time = now.time();
        let diff = time::PrimitiveDateTime::new(now_date, now_time) - time::PrimitiveDateTime::new(now_date, diff);
        Self::duration_offset(diff)
    }

    fn parse_adb_format(text: &str) -> Result<Self, ()> {
        use core::convert::TryInto;

        let mut split = text.split(' ');

        let date = split.next().unwrap();
        let time = if let Some(time) = split.next() {
            time
        } else {
            return Err(())
        };

        let date_split = date.split('-').collect::<Vec<_>>();
        if date_split.len() > 3 {
            return Err(());
        }

        let mut month_idx = 0;
        let year = if date_split.len() == 2 {
            time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc()).year()
        } else if let Ok(year) = date_split[0].parse(){
            month_idx = 1;
            year
        } else {
            return Err(());
        };

        let month = if let Ok(month) = date_split[month_idx].parse::<u8>() {
            match time::Month::try_from(month) {
                Ok(month) => month,
                Err(_) => return Err(()),
            }
        } else {
            return Err(());
        };

        let day = if let Ok(day) = date_split[month_idx + 1].parse() {
            day
        } else {
            return Err(());
        };

        let date = match time::Date::from_calendar_date(year, month, day) {
            Ok(date) => date,
            Err(_) => return Err(()),
        };

        let mut time_split = time.split(':');
        let hour = match time_split.next().and_then(|hour| hour.parse().ok()) {
            Some(hour) => hour,
            None => return Err(()),
        };
        let minute = match time_split.next().and_then(|minute| minute.parse().ok()) {
            Some(minute) => minute,
            None => return Err(()),
        };
        let second = match time_split.next().and_then(|second| second.parse::<f64>().ok())
                                            .and_then(|second| (second.trunc() as u64).try_into().ok()) {
            Some(second) => second,
            None => return Err(()),
        };

        let time = match time::Time::from_hms(hour, minute, second) {
            Ok(time) => time,
            Err(_) => return Err(()),
        };

        let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
        Ok(Self(time::PrimitiveDateTime::new(date, time).assume_offset(offset)))
    }
}

impl core::str::FromStr for Time {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if let Some(text) = text.strip_suffix('m') {
            match text[..text.len()-1].parse().map(|min| time::Duration::minutes(min)) {
                Ok(time) => Ok(Self::duration_offset(time)),
                _ => Err(())
            }
        } else if let Some(text) = text.strip_suffix('h') {
            match text[..text.len()-1].parse().map(|min| time::Duration::hours(min)) {
                Ok(time) => Ok(Self::duration_offset(time)),
                _ => Err(())
            }
        } else if let Some(text) = text.strip_suffix('s') {
            match text[..text.len()-1].parse().map(|min| time::Duration::seconds(min)) {
                Ok(time) => Ok(Self::duration_offset(time)),
                _ => Err(())
            }
        } else if let Ok(time) = time::Time::parse(text, TIME_PARSE) {
            Ok(Self::time_offset(time))
        } else if let Ok(time) = time::OffsetDateTime::parse(text, DATETIME_PARSE) {
            Ok(Time(time))
        } else if let Ok(time) = time::OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339) {
            Ok(Time(time))
        } else if let Ok(time) = Time::parse_adb_format(text) {
            Ok(time)
        } else {
            Err(())
        }
    }
}

impl core::ops::Deref for Time {
    type Target = time::OffsetDateTime;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Args, Debug)]
///plogcat 1.0.0
///Colorful wrapper over adb logcat command
pub struct Cli {
    #[arg(short, long)]
    ///Load alternate log buffer.
    pub buffer: Vec<String>,

    #[arg(long)]
    ///Filter output by currently running application, if `app` is not set.
    pub current: bool,

    #[arg(short, long)]
    ///Clears logcat content before running.
    pub clear: bool,

    #[arg(long)]
    ///Specifies to use USB connected device.
    pub device: bool,

    #[arg(short, long)]
    ///Dumps current log and exits.
    pub dump: bool,

    #[arg(long)]
    ///Specifies to use TCP/IP connected device.
    pub emulator: bool,

    #[arg(long)]
    ///Whether to include time. Default: false.
    pub time: bool,

    #[arg(long = "tag-width")]
    ///Specifies tag width. Default: 23.
    pub tag_width: usize,

    #[arg(short, long)]
    ///List of tags to include into output.
    pub tag: Vec<String>,

    #[arg(short, long, default_value)]
    ///Specifies minimum Android log level to include. Default Verbose.
    pub level: Level,

    #[arg(short = "L", long)]
    ///Dumps logs prior to the last reboot.
    pub last: bool,

    #[arg(long)]
    ///Strips output of color, making it more suitable for parsing.
    pub machine: bool,

    #[arg(short, long)]
    ///Print only provided number of lines and exits.
    pub max_count: Option<core::num::NonZeroU64>,

    #[arg(short, long)]
    ///Specifies device's serial number.
    pub serial: Option<String>,

    #[arg(short = "e", long)]
    ///Makes regex against which to match log lines.
    pub regex: Vec<String>,

    #[arg(long = "time-limit")]
    ///Prints within time range from specified time to the current time.
    pub time_limit: Option<Time>,

    #[arg(short, long = "ignored-tag")]
    ///List of tags to exclude from output.
    pub ignored_tag: Vec<String>,

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
        let mut output = match core::str::from_utf8(&output.stdout) {
            Ok(output) => output,
            Err(_) => {
                eprintln!("stdout output is not UTF-8");
                return Err(UTF8_ERROR);
            }
        };

        const TASK_RECORD: &str = "TaskRecord";
        const APP_PREFIX: &str = " A=";
        let app = loop {
            if let Some(record_idx) = output.find(TASK_RECORD) {
                if let Some(output) = output.get(record_idx + TASK_RECORD.len()..) {
                    if let Some(app_name_idx) = output.find(APP_PREFIX) {
                        if let Some(output) = output.get(app_name_idx + APP_PREFIX.len()..) {
                            if let Some(limit) = output.find(' ') {
                                break Some(App::PackageName(output[..limit].to_owned()));
                            } else {
                                break Some(App::PackageName(output.to_owned()));
                            }
                        }
                    }
                }

                output = &output[record_idx..];
            } else {
                break None;
            }
        };

        if app.is_none() {
            println!(">No app currently running");
        }
        self.app = app;

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

        for regex in self.regex.iter() {
            adb.arg("-e");
            adb.arg(&regex);
        }

        if let Some(max_count) = self.max_count {
            adb.arg("-m");
            adb.arg(&format!("{}", max_count));
        }

        if self.dump {
            adb.arg("-d");
        }

        if self.last {
            adb.arg("-L");
        }

        if let Some(ref time_limit) = self.time_limit {
            adb.arg("-T");
            adb.arg(&time_limit.0.format(FULL_DATETIME_FMT).expect("To format time"));
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
