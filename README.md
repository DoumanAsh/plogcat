# plogcat (pretty logcat)

![Rust](https://github.com/DoumanAsh/plogcat/workflows/Rust/badge.svg?branch=master)

Simple wrapper over logcat to provide easy way to interact with `adb logcat`

## Usage

```
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
         --tag_width <tag_width>         Specifies tag width. Default: 23.
    -t,  --tag <tag>...                  List of tags to include into output.
    -l,  --level <level>                 Specifies minimum Android log level to include. Default Verbose.
    -L,  --last                          Dumps logs prior to the last reboot.
    -m,  --max_count <max_count>         Print only provided number of lines and exits.
    -s,  --serial <serial>               Specifies device's serial number.
    -e,  --regex <regex>...              Makes regex against which to match log lines.
    -i,  --ignored_tag <ignored_tag>...  List of tags to exclude from output.

ARGS:
    [app]  Package name or pid by which to filter logcat
```
