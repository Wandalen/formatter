use self::input::{Delimiter, Input, Token};

mod classes;
mod cursor;
mod input;

#[derive(Default)]
struct Emitter {
    output: String,
}

impl Emitter {
    fn newline(&mut self) {
        self.output.push('\n');
    }

    fn raw(&mut self, string: &str) {
        self.output.push_str(string);
    }

    fn whitespace(&mut self) {
        self.output.push(' ');
    }

    fn finish(self) -> String {
        self.output
    }
}

impl Emitter {
    fn before(&mut self, current: Token, input: &Input) {
        match current {
            Token::CloseDelimiter(delimiter) => {
                if input
                    .prev()
                    .skip_whitespace(Token::OpenDelimiter(delimiter).into())
                {
                    match delimiter {
                        Delimiter::Brace => {}
                        _ => self.whitespace(),
                    }
                }
            }
            Token::OpenDelimiter(Delimiter::Brace) if input.prev() != Token::Newline => {
                self.newline()
            }
            _ if current.maybe_binary_operator() && input.prev() != Token::Whitespace => {
                self.whitespace()
            }
            _ => {}
        }
    }

    fn after(&mut self, current: Token, input: &Input) {
        match current {
            Token::OpenDelimiter(delimiter) => match delimiter {
                Delimiter::Paren | Delimiter::Bracket
                    if input
                        .peek()
                        .skip_whitespace(Token::CloseDelimiter(delimiter).into()) =>
                {
                    self.whitespace();
                }
                _ => {}
            },
            _ if current.maybe_binary_operator() && input.peek() != Token::Whitespace => {
                self.whitespace()
            }
            _ => (),
        }
    }
}

pub(crate) fn format(source: &str) -> String {
    let input = Input::of(source);
    let mut emitter = Emitter::default();

    for token in input.iter() {
        emitter.before(token, &input);
        emitter.raw(input.slice());
        emitter.after(token, &input);
    }

    emitter.finish()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::format;
    use pretty_assertions::assert_eq;

    fn with_extension(path: &PathBuf, extension: &str) -> PathBuf {
        match path.extension() {
            Some(raw_extension) => {
                let mut raw_extension = raw_extension.to_os_string();
                raw_extension.push(".");
                raw_extension.push(extension);
                path.with_extension(raw_extension)
            }
            None => path.with_extension(extension),
        }
    }

    fn traverse(root: &str, f: impl Fn(PathBuf, PathBuf)) {
        for entry in std::fs::read_dir(root).unwrap() {
            let input_path = entry.unwrap().path();
            let expected_path = with_extension(&input_path, "expected");

            if input_path.extension().unwrap_or_default() == "expected" {
                continue;
            };

            f(input_path, expected_path)
        }
    }

    #[test]
    fn with_extension_test() {
        let path = PathBuf::from("file.txt");
        let expected = PathBuf::from("file.txt.expected");
        assert_eq!(with_extension(&path, "expected"), expected);

        let path = PathBuf::from("dir/file");
        let expected = PathBuf::from("dir/file.expected");
        assert_eq!(with_extension(&path, "expected"), expected);

        let path = PathBuf::from("file");
        let expected = PathBuf::from("file.expected");
        assert_eq!(with_extension(&path, "expected"), expected);
    }

    #[test]
    fn tests() {
        traverse("tests/assets", |input, expected| {
            let input = format(&std::fs::read_to_string(input).unwrap());
            let expected = std::fs::read_to_string(expected).unwrap();

            assert_eq!(input, expected);
        });
    }
}
