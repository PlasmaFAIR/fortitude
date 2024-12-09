# incorrect-space-before-comment (S102)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What does it do?
Checks for inline comments that aren't preceeded by at least two spaces.

## Why is this bad?
Inline comments that aren't separated from code by any whitespace can make
code hard to read. Other language style guides recommend the use of two
spaces before inline comments, so we recommend the same here.

## References
- [PEP8 Python Style Guide](https://peps.python.org/pep-0008/)
- [Google C++ Style Guide](https://google.github.io/styleguide/cppguide.html#Horizontal_Whitespace)