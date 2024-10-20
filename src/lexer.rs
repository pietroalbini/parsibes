#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token<'a> {
    OpenParen,
    CloseParen,
    OpenSquare,
    CloseSquare,
    Comma,
    Plus,
    Dash,
    Number(i64),
    String(&'a str),
}

pub(crate) struct Lexer<'a> {
    input: &'a str,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self { input }
    }

    fn first<F: Fn(char) -> bool>(&self, condition: F) -> Option<usize> {
        self.input
            .char_indices()
            .find(|(_, c)| condition(*c))
            .map(|(i, _)| i)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let first = self.input.chars().next()?;

            if first.is_ascii_digit() {
                let end = self
                    .first(|c| !c.is_ascii_digit())
                    .unwrap_or(self.input.len());

                let number: i64 = self.input[..end].parse().unwrap();
                self.input = &self.input[end..];
                return Some(Token::Number(number));
            }

            self.input = &self.input[first.len_utf8()..];

            if first.is_whitespace() {
                continue;
            }
            if first == '"' {
                let end = self.first(|c| c == '"').expect("unterminated string");

                let result = Token::String(&self.input[..end]);
                self.input = &self.input[end + 1..];
                return Some(result);
            }
            match first {
                '(' => return Some(Token::OpenParen),
                ')' => return Some(Token::CloseParen),
                '[' => return Some(Token::OpenSquare),
                ']' => return Some(Token::CloseSquare),
                '-' => return Some(Token::Dash),
                '+' => return Some(Token::Plus),
                ',' => return Some(Token::Comma),
                _ => panic!("unexpected char: {first}"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex() {
        let input = "1234  +-,[] ()   \t \"hello world\"69";
        let tokens = Lexer::new(input).collect::<Vec<_>>();
        assert_eq!(
            &[
                Token::Number(1234),
                Token::Plus,
                Token::Dash,
                Token::Comma,
                Token::OpenSquare,
                Token::CloseSquare,
                Token::OpenParen,
                Token::CloseParen,
                Token::String("hello world"),
                Token::Number(69)
            ],
            tokens.as_slice()
        );
    }
}
