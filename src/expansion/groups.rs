use crate::expansion::tree::TokenTree;
use crate::lexer::Token;
use std::mem::take;

/// [`Group`] propagates repetitions as-is from [`TokenTree`], and collapses multiple
/// [`TokenTree`]s without repetitions into a single element (the "group").
#[derive(Debug, Clone)]
pub(super) enum Group<'src> {
    Simple(Vec<Token<'src>>),
    Repetition {
        content: Vec<Group<'src>>,
        separator: Option<Token<'src>>,
    },
}

pub(super) fn create_groups(stream: Vec<TokenTree<'_>>) -> Vec<Group<'_>> {
    let mut result = Vec::new();
    let mut current_simple = Vec::new();

    for tree in stream {
        match tree {
            TokenTree::Token(token) => current_simple.push(token),
            TokenTree::Repetition(repetition) => {
                if !current_simple.is_empty() {
                    result.push(Group::Simple(take(&mut current_simple)));
                }
                result.push(Group::Repetition {
                    content: create_groups(repetition.repeated),
                    separator: repetition.separator,
                });
            }
        }
    }

    if !current_simple.is_empty() {
        result.push(Group::Simple(current_simple));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expansion::tree::parse_tokenstream;
    use crate::lexer::Lexer;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_create_groups() {
        let input = "[$(1, $(3,)*,),*]";
        let stream = parse_tokenstream(Lexer::new(input).collect()).unwrap();

        let groups = create_groups(stream);
        assert_debug_snapshot!(groups, @r###"
        [
            Simple(
                [
                    Token( [ ),
                ],
            ),
            Repetition {
                content: [
                    Simple(
                        [
                            Token( 1 ),
                            Token( , ),
                        ],
                    ),
                    Repetition {
                        content: [
                            Simple(
                                [
                                    Token( 3 ),
                                    Token( , ),
                                ],
                            ),
                        ],
                        separator: None,
                    },
                    Simple(
                        [
                            Token( , ),
                        ],
                    ),
                ],
                separator: Some(
                    Token( , ),
                ),
            },
            Simple(
                [
                    Token( ] ),
                ],
            ),
        ]
        "###);
    }
}
