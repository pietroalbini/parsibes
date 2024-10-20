mod helpers;
mod state;

use crate::lexer::Token;
use crate::parser::helpers::{while_any_unpaused, Diverge};
pub use crate::parser::state::State;
use anyhow::Error;

pub fn parse_expression(state: &mut State<'_>) -> Result<(), Error> {
    // An iteration of this loop parses one value and optionally a binary operator. By looping we
    // can parse arbitrarily long expressions, as they will continue to loop until paused.
    while_any_unpaused(state, |state, pause| {
        // Different kinds of expressions require different parsing rules:
        Diverge::new(state, |peek| match peek {
            Token::OpenSquare => ExprState::Array,
            Token::OpenParen => ExprState::Nested,
            _ => ExprState::Fallback,
        })?
        .handle(ExprState::Fallback, |state| {
            state.next_token(|next| match &next.token {
                Token::Number(_) => {}
                Token::String(_) => {}
                _ => next.mismatch("expression"),
            })
        })?
        .handle(ExprState::Nested, |state| {
            state.expect(Token::OpenParen)?;
            parse_expression(state)?;
            state.expect(Token::CloseParen)?;
            Ok(())
        })?
        .handle(ExprState::Array, parse_array)?;

        // As we don't need to return an AST, we don't need to do the nested recursive functions to
        // handle precedence, we can just parse one operator after another.
        state.peek_token(|peek| match &peek.token {
            Some(Token::Dash) => peek.consume(),
            Some(Token::Plus) => peek.consume(),
            // Next token is not a binary operator, stop parsing this expression.
            _ => peek.pause(pause),
        })?;

        Ok(())
    })?;

    Ok(())
}

pub fn parse_array(state: &mut State<'_>) -> Result<(), Error> {
    state.expect(Token::OpenSquare)?;
    // Parse zero or more array items:
    while_any_unpaused(state, |state, pause| {
        // Handles the closing ] either when the array is empty, or when there is a trailing comma.
        state.peek_token(|peek| match &peek.token {
            Some(Token::CloseSquare) => {
                peek.consume();
                peek.pause(pause);
            }
            _ => {}
        })?;

        parse_expression(state)?;

        state.next_token(|next| match &next.token {
            Token::CloseSquare => next.pause(pause),
            Token::Comma => {}
            _ => next.mismatch("end of array or comma"),
        })?;
        Ok(())
    })?;

    Ok(())
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum ExprState {
    Fallback,
    Nested,
    Array,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::streams::Streams;

    #[test]
    fn test_parse_expression() {
        parse_expression(&mut state(&[
            // Parsed in parallel:
            "1",
            "\"hello\"",
            "1 + 2 + [3] + 4 - \"world\"",
            "1 + (3 - 2)",
        ]))
        .unwrap();
    }

    #[test]
    fn test_parse_array() {
        parse_array(&mut state(&[
            // Parsed in parallel:
            "[]",
            "[1]",
            "[1 + 2, \"hello\"]",
            "[1,]",
            "[[[[[[1]]]]]]",
        ]))
        .unwrap();
    }

    fn state(inputs: &[&'static str]) -> State<'static> {
        let mut streams = Streams::new();
        for input in inputs {
            streams.add(input);
        }
        State::new(streams)
    }
}