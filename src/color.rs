use std::collections::HashMap;

pub struct Stack {
    colors: [termcolor::Color; 5],
    tags: HashMap<String, termcolor::Color>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            tags: Default::default(),
            colors: [termcolor::Color::Green,
                     termcolor::Color::Yellow, termcolor::Color::Blue,
                     termcolor::Color::Magenta, termcolor::Color::Cyan],
        }
    }

    pub fn get_color(&mut self, tag: &str) -> termcolor::Color {
        match self.tags.get(tag) {
            Some(color) => *color,
            None => {
                let color = if tag.contains("error") || tag.contains("Error") {
                    termcolor::Color::Red
                } else {
                    let color = self.colors[0];

                    for idx in 0..(self.colors.len()-1) {
                        self.colors.swap(idx, idx+1);
                    }

                    color
                };

                self.tags.insert(tag.to_owned(), color);
                color
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Stack;

    #[test]
    fn verify_color_stack_shift() {
        let mut stack = Stack::new();
        assert_eq!(stack.get_color("1"), termcolor::Color::Green);
        assert_eq!(stack.get_color("2"), termcolor::Color::Yellow);
        assert_eq!(stack.get_color("3"), termcolor::Color::Blue);
        assert_eq!(stack.get_color("4"), termcolor::Color::Magenta);
        assert_eq!(stack.get_color("5"), termcolor::Color::Cyan);
        assert_eq!(stack.get_color("6"), termcolor::Color::Green);
    }

    #[test]
    fn verify_cached_color() {
        let mut stack = Stack::new();
        assert_eq!(stack.get_color("1"), termcolor::Color::Green);
        assert_eq!(stack.get_color("2"), termcolor::Color::Yellow);
        assert_eq!(stack.get_color("1"), termcolor::Color::Green);
        assert_eq!(stack.get_color("2"), termcolor::Color::Yellow);
    }
}
