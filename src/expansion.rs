use anyhow::{anyhow, bail, ensure, Error};

use crate::lexer::{Lexer, Token};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Chunk<'src> {
    pub(super) tokens: Vec<Token<'src>>,
    pub(super) childs: Option<Box<[Chunk<'src>; 2]>>,
}

// Warning: this does not check for delimiter balancing.
pub(super) fn of(input: &str) -> Result<Chunk, Error> {
    let tokens = Lexer::new(input).collect::<Vec<_>>();

    let token_stream = parse_tokenstream(tokens)?;

    todo!()
}

fn parse_tokenstream(tokens: Vec<Token>) -> Result<Vec<TokenTree>, Error> {
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

fn chunks_from_tokenstream<'src>(
    mut stream: &[TokenTree<'src>],
    childs: Option<[Chunk<'src>; 2]>,
) -> Chunk<'src> {
    todo!()
}

// Mom said we can have a little data structure as a threat.
enum TokenTree<'src> {
    Token(Token<'src>),
    Repetition(TokenRepetition<'src>),
}

struct TokenRepetition<'src> {
    repeated: Vec<TokenTree<'src>>,
    separator: Option<Token<'src>>,
}
