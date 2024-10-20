use crate::lexer::Lexer;
use std::collections::HashSet;
use std::iter::Peekable;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Streams<'src> {
    streams: Vec<Stream<'src>>,
}

impl<'src> Streams<'src> {
    pub fn new() -> Self {
        Self {
            streams: Vec::new(),
        }
    }

    pub fn add(&mut self, program: &'src str) {
        let id = StreamId(self.streams.len());
        self.streams.push(Stream {
            lexer: Lexer::new(program).peekable(),
            pause: HashSet::new(),
            id,
        });
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Stream<'src>> {
        self.streams.iter()
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut Stream<'src>> {
        self.streams.iter_mut()
    }
}

pub(crate) struct Stream<'src> {
    pub(crate) lexer: Peekable<Lexer<'src>>,
    id: StreamId,
    pause: HashSet<PauseId>,
}

impl<'src> Stream<'src> {
    pub(crate) fn id(&self) -> StreamId {
        self.id
    }

    /// Mark the stream to be paused, with the provided pause ID. The only effect of this is that
    /// [`Stream::maybe_unpause`] will return `false`: it's up to the user to verify whether the
    /// stream is paused before pulling tokens from it.
    ///
    /// It's possible to call this multiple times with different [`PauseId`], which will mark the
    /// stream to be paused by all of them.
    pub(crate) fn pause(&mut self, id: PauseId) {
        self.pause.insert(id);
    }

    /// If the stream is paused by the provided [`PauseId`] unpause it, otherwise do nothing.
    ///
    /// Note that it's possible to pause a stream with multiple [`PauseId`]. In that case, the
    /// stream will only be unpaused if *all* of the pauses are removed.
    pub(crate) fn maybe_unpause(&mut self, id: PauseId) {
        self.pause.remove(&id);
    }

    /// Return whether the stream is supposed to be paused.
    pub(crate) fn is_paused(&self) -> bool {
        !self.pause.is_empty()
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StreamId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PauseId(usize);

impl PauseId {
    pub(crate) fn new() -> PauseId {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        PauseId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}
