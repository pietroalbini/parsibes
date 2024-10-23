use crate::lexer::Token;
use crate::streams::{PauseId, Stream, StreamId, Streams};
use anyhow::{anyhow, Error};
use std::fmt::Debug;

pub struct State<'src> {
    pub(super) streams: Streams<'src>,
}

impl<'src> State<'src> {
    pub fn new(streams: Streams<'src>) -> Self {
        Self { streams }
    }
}

impl<'src> State<'src> {
    /// Check whether any of the streams is unpaused.
    pub(super) fn is_any_unpaused(&self) -> bool {
        self.streams.iter().any(|s| !s.is_paused())
    }

    /// Unpause all streams currently paused due to the provided [`PauseId`]. If a stream is paused
    /// both by the provided [`PauseId`] and another one, it will not actually be unpaused until
    /// all [`PauseId`]s are removed.
    pub(super) fn unpause(&mut self, id: PauseId) {
        for stream in self.streams.iter_mut() {
            stream.maybe_unpause(id);
        }
    }

    /// Check that the next token in all unpaused streams matches the expected one.
    pub(super) fn expect(&mut self, expected: Token<'static>) -> Result<(), Error> {
        self.next_token(|next| {
            if next.token != expected {
                next.mismatch(&format!("{expected:?}"));
            }
        })
    }

    /// Consume the next token in all unpaused streams and invoke the provided closure for each
    /// consumed token.
    pub(super) fn next_token<F>(&mut self, action: F) -> Result<(), Error>
    where
        F: FnMut(&mut StreamActions<'_, 'src, Token<'src>>),
    {
        self.action_on_token(action, |stream| {
            stream.lexer.next().ok_or_else(|| anyhow!("end of input"))
        })
    }

    /// Peek at the next token in all unpaused streams without consuming it, and invoke the
    /// provided closure for each peeked token.
    pub(super) fn peek_token<F>(&mut self, action: F) -> Result<(), Error>
    where
        F: FnMut(&mut StreamActions<'_, 'src, Option<Token<'src>>>),
    {
        self.action_on_token(action, |stream| Ok(stream.lexer.peek().cloned()))
    }

    fn action_on_token<T: Debug, F, G>(
        &mut self,
        mut action: F,
        token_getter: G,
    ) -> Result<(), Error>
    where
        F: FnMut(&mut StreamActions<'_, 'src, T>),
        G: Fn(&mut Stream<'src>) -> Result<T, Error>,
    {
        for stream in self.streams.iter_mut() {
            if stream.is_paused() {
                continue;
            }
            let token = token_getter(stream)?;
            let mut actions = StreamActions {
                stream,
                token,
                error: None,
            };
            action(&mut actions);
            if let Some(err) = actions.error {
                return Err(err);
            }
        }
        Ok(())
    }
}

pub(super) struct StreamActions<'parent, 'src, T: Debug> {
    pub(super) token: T,
    stream: &'parent mut Stream<'src>,
    error: Option<Error>,
}

impl<T: Debug> StreamActions<'_, '_, T> {
    /// Cause the parsing to stop with a token mismatch error.
    pub(super) fn mismatch(&mut self, expected: &str) {
        self.error = Some(anyhow!("expected {expected}, found {:?}", self.token));
    }

    /// Pause this stream with the provided [`PauseId`].
    pub(super) fn pause(&mut self, id: PauseId) {
        self.stream.pause(id);
    }

    pub(super) fn stream_id(&self) -> StreamId {
        self.stream.id()
    }
}

impl<T: Debug> StreamActions<'_, '_, Option<T>> {
    /// Consume the peeked token.
    pub(super) fn consume(&mut self) {
        self.stream.lexer.next();
    }
}
