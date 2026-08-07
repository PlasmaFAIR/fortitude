# Fortitude wrapper for tree-sitter

Inspired by
[`yak-sitter`](https://github.com/Jakobeha/type-sitter/blob/main/yak-sitter/),
this is a mostly drop-in replacement for `tree-sitter`, except that like
`yak-sitter`, it stores the source text in the `Tree` which allows each `Node`
to immediately access its text. The cost of this is that it's now unsafe to make
edits to the `Tree` -- but we modify the source text and re-parse, so that's not
a problem for us.
