# plogcat (pretty logcat)

[![Rust](https://github.com/DoumanAsh/plogcat/actions/workflows/rust.yml/badge.svg)](https://github.com/DoumanAsh/plogcat/actions/workflows/rust.yml)

Simple wrapper over logcat to provide easy way to interact with `adb logcat`

## Usage

```
plogcat 1.0.0
Colorful wrapper over adb logcat command

USAGE: [OPTIONS] [app]

OPTIONS:
    -h,  --help                          Prints this help information
    -b,  --buffer <buffer>...            Load alternate log buffer.
         --current                       Filter output by currently running application, if `app` is not set.
    -c,  --clear                         Clears logcat content before running.
         --device                        Specifies to use USB connected device.
    -d,  --dump                          Dumps current log and exits.
         --emulator                      Specifies to use TCP/IP connected device.
         --time                          Whether to include time. Default: false.
         --tag-width <tag_width>         Specifies tag width. Default: 23.
    -t,  --tag <tag>...                  List of tags to include into output.
    -l,  --level <level>                 Specifies minimum Android log level to include. Default Verbose.
    -L,  --last                          Dumps logs prior to the last reboot.
         --machine                       Strips output of color, making it more suitable for parsing.
    -m,  --max_count <max_count>         Print only provided number of lines and exits.
    -s,  --serial <serial>               Specifies device's serial number.
    -e,  --regex <regex>...              Makes regex against which to match log lines.
         --time-limit <time_limit>       Prints within time range from specified time to the current time.
    -i,  --ignored-tag <ignored_tag>...  List of tags to exclude from output.

ARGS:
    [app]  Package name or pid by which to filter logcat. If multiple apps found with the same name, it will output for every match
```
