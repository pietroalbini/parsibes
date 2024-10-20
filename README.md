# Parsibes

Parsibes is an *proof of concept* for how a parser could check the validity of
multiple token streams during a single parsing process. The source code is
released under either the MIT or the Apache 2.0 licenses.

> [!CAUTION]
>
> This is a prototype I wrote during an afternoon: it is incomplete and I don't
> have plans to finish it.

## Goal

The main goal of Parsibes is to check the validity of one or more token streams
at the same time. The original use case for this is [expandable]: Rust macros
contain repetitions, and the token stream for the macro expansion could
bifurcate in the presence of repetitions.

Parsibes shows an approach for how such validation could be done, by starting
from a single token stream, bifurcating into multiple token streams in the
middle of the parsing, and then continue *in the same parsing invocation* to
operate on all the parallel token streams.

Note that for this prototype, the multiple token streams are defined at startup
and do not bifurcate in the middle of parsing, as the goal of the prototype is
to demonstrate that parsing multiple token streams at the same time is possible
at all.

Generating an AST is explicitly an anti-goal for Parsibes, as [expandable] just
needs to validate the macro expansion, not perform any analysis on the AST.

## High level design

Parsibes's parser is structured like any traditional recursive descent parser,
with functions parsing every construct and recursively calling each other. As
we want to parse each token stream in parallel, every parsing operation acts on
all token streams at the same time (instead of just on one token stream).

Just parsing every token stream at the same time wouldn't work though, because
the token streams can have different programs defined in them. `[1]` would
finish parsing sooner than `[1, 2, 3]`.

To solve that, Parsibes supports "pausing" the parsing of an individual token
stream, and resuming the parsing at a later point in time. This way, the
parsing of all token streams can be synchronized even if the programs contained
are not exactly the same.

[expandable]: https://github.com/scrabsha/expandable
