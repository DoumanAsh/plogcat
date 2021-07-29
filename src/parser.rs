///Logcat line
pub struct LogCatLine<'a> {
    pub date: &'a str,
    pub time: &'a str,
    pub level: &'a str,
    pub tag: &'a str,
    pub msg: &'a str,
}

macro_rules! next_part {
    ($cursor:ident) => {
        if let Some(mut idx) = $cursor.find(' ') {
            let res = &$cursor[..idx];
            while let Some(b' ') = $cursor.as_bytes().get(idx) {
                idx += 1;
            }

            if idx >= $cursor.len() {
                return None;
            }

            $cursor = &$cursor[idx..];
            res
        } else {
            return None;
        }
    }
}

//^([0-9]+-[0-9]+\s[0-9]+:[0-9]+:[0-9]+.[0-9]+)\s([A-Z])/(.+?)\( *(\d+)\): (.*?)$;
///Parses line from output of logcat -v time
pub fn parse<'a>(text: &'a str) -> Option<LogCatLine<'a>> {
    let mut cursor = text;
    let date = next_part!(cursor);
    let time = next_part!(cursor);
    let level_tag = if let Some(idx) = cursor.find('(') {
            let res = &cursor[..idx];
            cursor = &cursor[idx..];

            if let Some(msg_idx) = cursor.find(':') {
                if let Some(msg) = cursor.get(msg_idx+1..) {
                    cursor = msg;
                } else {
                    return None;
                }
            } else {
                return None;
            }

            res.trim()
    } else {
        return None;
    };

    let (level, tag) = {
        let mut level_tag_split = level_tag.splitn(2, '/');
        let level = match level_tag_split.next() {
            Some(level) => level,
            None => unsafe {
                core::hint::unreachable_unchecked()
            },
        };

        let tag = match level_tag_split.next() {
            Some(tag) => tag,
            None => return None,
        };

        (level, tag)
    };

    Some(LogCatLine {
        date,
        time,
        level,
        tag,
        msg: cursor.trim(),
    })
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn should_parse_valid_line() {
        let result = parse("12-02    24:01:13.237   i/flutter ( 666):     my super log ").expect("To parse");
        assert_eq!(result.date, "12-02");
        assert_eq!(result.time, "24:01:13.237");
        assert_eq!(result.level, "i");
        assert_eq!(result.tag, "flutter");
        assert_eq!(result.msg, "my super log");
    }

}
