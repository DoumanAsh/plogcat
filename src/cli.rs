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
    #[arg(short, long)]
    ///Filter output by currently running application.
    pub current: bool,

    #[arg(short, long)]
    ///Specifies tag width. Default: 23.
    pub tag_width: usize,

    #[arg(short, long)]
    ///Package name or pid by which to filter logcat
    pub app: Option<App>,
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
