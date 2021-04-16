pub use regex::Regex;

pub fn from_tag_list<I: AsRef<str>, T: IntoIterator<Item = I>>(tags: T) -> Result<Regex, regex::Error> {
    let mut regex_str = String::new();
    regex_str.push_str("^(");

    for tag in tags.into_iter() {
        regex_str.push_str(tag.as_ref());
        regex_str.push('|');
    }

    regex_str.pop();
    regex_str.push_str(")$");

    regex::Regex::new(&regex_str)
}
