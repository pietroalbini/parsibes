use crate::lexer::Token;
use crate::parser::state::State;
use crate::streams::{PauseId, StreamId};
use anyhow::Error;
use std::collections::BTreeMap;

/// Execute the closure repeatedly until all streams are paused, and then unpause the [`ParseId`]
/// provided as an argument to the closure.
pub(super) fn while_any_unpaused<'a, F>(state: &mut State<'a>, mut f: F) -> Result<(), Error>
where
    F: FnMut(&mut State<'a>, PauseId) -> Result<(), Error>,
{
    let pause = PauseId::new();
    while state.is_any_unpaused() {
        f(state, pause)?;
    }
    state.unpause(pause);
    Ok(())
}

/// The [`Diverge`] struct allows to execute different parsing functions depending on the contents
/// of each stream. This is useful for example when parsing an expression, as there are multiple
/// kinds of expressions with different parsing rules.
///.
/// [`Diverge`] works by first grouping the streams in [`Diverge::new`], checking the (peeked) next
/// token and returning the corresponding group ID. Then, [`Diverge::handle`] is called for each
/// group ID, to provide the logic for how to handle that group.
///
/// Under the hood, when handling a specific group ID, all other streams are paused.
pub(super) struct Diverge<'src, 'state, K: Ord> {
    groups: BTreeMap<K, Vec<StreamId>>,
    state: &'state mut State<'src>,
}

impl<'src, 'state, K: Ord> Diverge<'src, 'state, K> {
    pub(super) fn new<G>(state: &'state mut State<'src>, mut grouper: G) -> Result<Self, Error>
    where
        G: FnMut(&Token<'_>) -> K,
    {
        let mut groups = BTreeMap::new();
        state.peek_token(|peek| {
            if let Some(token) = &peek.token {
                groups
                    .entry(grouper(token))
                    .or_insert_with(Vec::new)
                    .push(peek.stream_id())
            }
        })?;
        Ok(Self { groups, state })
    }

    pub(super) fn handle<F>(mut self, case: K, handler: F) -> Result<Self, Error>
    where
        F: FnOnce(&mut State<'src>) -> Result<(), Error>,
    {
        let Some(group) = self.groups.remove(&case) else {
            return Ok(self);
        };

        let pause = PauseId::new();
        for stream in self.state.streams.iter_mut() {
            if !group.contains(&stream.id()) {
                stream.pause(pause);
            }
        }

        handler(self.state)?;

        for stream in self.state.streams.iter_mut() {
            stream.maybe_unpause(pause);
        }

        Ok(self)
    }
}

#[macro_export]
macro_rules! diverge {
    (match $state:ident { $($pat:pat => |$state_binding:ident| $block:expr),* $(,)? }) => {
        crate::parser::helpers::Diverge::new($state, |token| match &token {
            $($pat => stringify!($pat),)*
        })?
        $(.handle(stringify!($pat), |$state_binding| $block)?)*
        ;
    };
}
