mod groups;
mod tree;

use crate::expansion::groups::{create_groups, Group};
use crate::expansion::tree::{parse_tokenstream, TokenTree};
use crate::lexer::{Lexer, Token};
use anyhow::{anyhow, bail, ensure, Error};
use std::mem::take;

pub(crate) struct Chunks<'src> {
    inner: Vec<Chunk<'src>>,
    firsts: Vec<ChunkId>,
}

impl<'src> Chunks<'src> {
    fn new() -> Self {
        Self {
            inner: Vec::new(),
            firsts: Vec::new(),
        }
    }

    pub(crate) fn get(&self, id: ChunkId) -> &Chunk<'src> {
        &self.inner[id.0]
    }

    pub(crate) fn firsts(&self) -> impl Iterator<Item = &Chunk<'src>> {
        self.firsts.iter().map(|id| self.get(*id))
    }

    fn allocate(&mut self, chunk: Chunk<'src>) -> ChunkId {
        let id = ChunkId(self.inner.len());
        self.inner.push(chunk);
        id
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub(crate) struct ChunkId(usize);

#[derive(Clone, PartialEq)]
pub(crate) struct Chunk<'src> {
    pub(crate) tokens: Vec<Token<'src>>,
    pub(crate) childs: Vec<ChunkId>,
}

// Warning: this does not check for delimiter balancing.
pub(super) fn of(input: &str) -> Result<Chunks, Error> {
    let tokens = Lexer::new(input).collect::<Vec<_>>();

    let token_stream = parse_tokenstream(tokens)?;
    let groups = create_groups(token_stream);

    let mut chunks = Chunks::new();
    let result = create_chunks(&mut chunks, groups, Vec::new());
    chunks.firsts = result;

    Ok(chunks)
}

fn create_chunks<'src>(
    chunks: &mut Chunks<'src>,
    groups: Vec<Group<'src>>,
    mut attach_to: Vec<ChunkId>,
) -> Vec<ChunkId> /* First */ {
    for group in groups.into_iter().rev() {
        match group {
            Group::Simple(tokens) => {
                let id = chunks.allocate(Chunk {
                    tokens,
                    childs: attach_to,
                });
                attach_to = vec![id];
            }
            Group::Repetition { content, separator } => {
                // With zero repetitions we don't need an extra node to be created.

                // With one repetition we create chunks attached to the next set of chunks.
                let case_one_ids = create_chunks(chunks, content.clone(), attach_to.clone());
                attach_to.extend(case_one_ids.iter().copied());

                // With two repetitions we create chunks attached to the first repetition.
                let attach_second_to = if let Some(sep) = separator {
                    // If there is a separator, create a chunk with the separator between the first
                    // and the second.
                    vec![chunks.allocate(Chunk {
                        tokens: vec![sep],
                        childs: case_one_ids,
                    })]
                } else {
                    case_one_ids
                };
                let case_two_ids = create_chunks(chunks, content, attach_second_to);
                attach_to.extend(case_two_ids.into_iter());
            }
        }
    }
    attach_to
}

// Debug impls to make the tests look better:

struct ListAsMap<'a, T>(&'a Vec<T>);

impl<T: std::fmt::Debug> std::fmt::Debug for ListAsMap<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for (i, item) in self.0.iter().enumerate() {
            map.entry(&i, item);
        }
        map.finish()
    }
}

struct ForceSingleLine<T>(T);

impl<T: std::fmt::Debug> std::fmt::Debug for ForceSingleLine<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::fmt::Debug for Chunk<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("tokens", &self.tokens)
            .field("childs", &ForceSingleLine(&self.childs))
            .finish()
    }
}

impl std::fmt::Debug for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl std::fmt::Debug for Chunks<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunks")
            .field("inner", &ListAsMap(&self.inner))
            .field("firsts", &ForceSingleLine(&self.firsts))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_expansion_simple() {
        let input = "[1, 2, 3]";
        let result = of(input);

        assert_debug_snapshot!(result, @r###"
        Ok(
            Chunks {
                inner: {
                    0: Chunk {
                        tokens: [
                            Token( [ ),
                            Token( 1 ),
                            Token( , ),
                            Token( 2 ),
                            Token( , ),
                            Token( 3 ),
                            Token( ] ),
                        ],
                        childs: [],
                    },
                },
                firsts: [#0],
            },
        )
        "###);
    }

    #[test]
    fn test_expansion_mild() {
        let input = "[$(1),*]";
        let result = of(input);

        assert_debug_snapshot!(result, @r###"
        Ok(
            Chunks {
                inner: {
                    0: Chunk {
                        tokens: [
                            Token( ] ),
                        ],
                        childs: [],
                    },
                    1: Chunk {
                        tokens: [
                            Token( 1 ),
                        ],
                        childs: [#0],
                    },
                    2: Chunk {
                        tokens: [
                            Token( , ),
                        ],
                        childs: [#1],
                    },
                    3: Chunk {
                        tokens: [
                            Token( 1 ),
                        ],
                        childs: [#2],
                    },
                    4: Chunk {
                        tokens: [
                            Token( [ ),
                        ],
                        childs: [#0, #1, #3],
                    },
                },
                firsts: [#4],
            },
        )
        "###);
    }

    #[test]
    fn test_expansion_complex() {
        let input = "[$(1, $(3,)*),*]";
        let result = of(input);

        assert_debug_snapshot!(result, @r###"
        Ok(
            Chunks {
                inner: {
                    0: Chunk {
                        tokens: [
                            Token( ] ),
                        ],
                        childs: [],
                    },
                    1: Chunk {
                        tokens: [
                            Token( 3 ),
                            Token( , ),
                        ],
                        childs: [#0],
                    },
                    2: Chunk {
                        tokens: [
                            Token( 3 ),
                            Token( , ),
                        ],
                        childs: [#1],
                    },
                    3: Chunk {
                        tokens: [
                            Token( 1 ),
                            Token( , ),
                        ],
                        childs: [#0, #1, #2],
                    },
                    4: Chunk {
                        tokens: [
                            Token( , ),
                        ],
                        childs: [#3],
                    },
                    5: Chunk {
                        tokens: [
                            Token( 3 ),
                            Token( , ),
                        ],
                        childs: [#4],
                    },
                    6: Chunk {
                        tokens: [
                            Token( 3 ),
                            Token( , ),
                        ],
                        childs: [#5],
                    },
                    7: Chunk {
                        tokens: [
                            Token( 1 ),
                            Token( , ),
                        ],
                        childs: [#4, #5, #6],
                    },
                    8: Chunk {
                        tokens: [
                            Token( [ ),
                        ],
                        childs: [#0, #3, #7],
                    },
                },
                firsts: [#8],
            },
        )
        "###);
    }
}
