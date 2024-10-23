use crate::lexer::Token;
use anyhow::{anyhow, bail, ensure, Error};

pub(super) fn parse_tokenstream(tokens: Vec<Token>) -> Result<Vec<TokenTree>, Error> {
    let mut tokens = tokens.as_slice();
    let mut trees = Vec::new();
    while !tokens.is_empty() {
        let (tree, tokens_) = parse_tokentree(tokens)?;
        tokens = tokens_;

        trees.push(tree);
    }

    Ok(trees)
}

fn parse_tokentree<'a, 'src>(
    input: &'a [Token<'src>],
) -> Result<(TokenTree<'src>, &'a [Token<'src>]), Error> {
    let tok = *input
        .first()
        .ok_or_else(|| anyhow!("Failed to parse a tokentree out of no token at all :/"))?;

    if tok != Token::Dollar {
        return Ok((TokenTree::Token(tok), &input[1..]));
    }

    // Eat the `$`.
    let input = &input[1..];

    // Eat the `(`.
    ensure!(
        matches!(input.first(), Some(Token::OpenParen)),
        "Expected `(` after the `$`"
    );
    let input = &input[1..];

    // Depth = 0 => we reached the closing paren!
    let mut depth = 1;
    let mut idx = 0;
    while depth > 0 {
        match input.get(idx) {
            Some(Token::CloseParen) => depth -= 1,
            Some(Token::OpenParen) => depth += 1,
            Some(_) => {}

            None => bail!("Unbalanced parentheses"),
        }

        idx += 1;
    }

    let (inner_tokens, tail) = input.split_at(idx);

    // Remove `)`.
    let mut inner_tokens = &inner_tokens[..inner_tokens.len() - 1];
    // Remove repetition seperator and operator.
    //
    // TODOWO: handle `+` and `?` :3
    let (separator, tail) = match tail.split_first() {
        Some((Token::Star, tail)) => (None, tail),
        Some((anything, tail)) => {
            ensure!(tail.first().copied() == Some(Token::Star), "Expected `*`");
            let tail = &tail[1..];
            (Some(*anything), tail)
        }

        None => bail!("Expected tokens :O"),
    };

    let mut repeated = Vec::new();
    while !inner_tokens.is_empty() {
        let (tok, inner_tokens_) = parse_tokentree(inner_tokens)?;
        repeated.push(tok);
        inner_tokens = inner_tokens_;
    }

    let tree = TokenTree::Repetition(TokenRepetition {
        repeated,
        separator,
    });

    Ok((tree, tail))
}

#[derive(Debug)]
pub(super) enum TokenTree<'src> {
    Token(Token<'src>),
    Repetition(TokenRepetition<'src>),
}

#[derive(Debug)]
pub(super) struct TokenRepetition<'src> {
    pub(super) repeated: Vec<TokenTree<'src>>,
    pub(super) separator: Option<Token<'src>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_parse_tokenstream() {
        let input = "[$(1, 2),*]";
        let lexed = Lexer::new(input).collect::<Vec<_>>();
        let stream = parse_tokenstream(lexed).unwrap();

        assert_debug_snapshot!(stream, @r###"
        [
            Token(
                Token( [ ),
            ),
            Repetition(
                TokenRepetition {
                    repeated: [
                        Token(
                            Token( 1 ),
                        ),
                        Token(
                            Token( , ),
                        ),
                        Token(
                            Token( 2 ),
                        ),
                    ],
                    separator: Some(
                        Token( , ),
                    ),
                },
            ),
            Token(
                Token( ] ),
            ),
        ]
        "###);
    }
}
