---
title: 'Fortitude: The Fortran Linter'
tags:
  - Fortran
  - static analysis
  - linting
  - linter
  - tool
  - tools
  - tooling
authors:
  - name: Liam Pattinson
    orcid: 0000-0001-8604-6904
    equal-contrib: true
    corresponding: true
    affiliation: 1
  - name: Peter Hill
    orcid: 0000-0003-3092-1858
    equal-contrib: true
    affiliation: 1
  - name: Ian McInerney
    orcid: 0000-0003-2616-9771
    affiliation: 2
  - name: Connor Aird
    affiliation: 3
affiliations:
 - name: University of York, United Kingdom
   index: 1
 - name: Imperial College London, United Kingdom
   index: 2
 - name: University College London, United Kingdom
   index: 3
date: 14 April 2026
bibliography: paper.bib
---

# Summary

Static analysis tools are used by software developers to better understand and
improve their software, and they achieve this by examining their source code
and/or binaries without running the software. A ‘linter’ is a type of static
analysis tool that offers an opinionated critique of software beyond its
syntactic validity, highlighting bug-prone coding patterns, deviations from
common style guides, and ways to upgrade to more modern features.

Fortitude is a linter targeting Fortran, which has hitherto lacked a standard
open-source solution. It is orders of magnitude faster than its open-source
counterparts, is highly customisable, has robust parsing capabilities, can
automatically fix linting issues, and can integrate directly into a user’s
editor or IDE.

# Statement of need

Fortran is the oldest programming language still in common usage today, and is
widely used for high-performance research applications in fields such as
climatology, quantum condensed matter physics, and fusion energy. The longevity
of Fortran means that much of the research software in use today is written to
older standards and contains a lot of technical debt. This places a large burden
on the maintainers of this research software, and porting Fortran codes to more
modern languages is not typically worth the resources required or the risks of
introducing new bugs or performance issues.

Fortitude is a Fortran linter designed to help writers and maintainers of
Fortran research software to improve the quality of their code. Its linting
rules are grouped under categories specifying what aspect of software they
intend to improve:

- Correctness: Rules to find bug-prone coding patterns, helping developers to
  catch errors early and improve the safety of their code.
- Obsolescent: Rules to flag Fortran features marked as obsolescent in the
  standard and recommend refactoring strategies to avoid them.
- Modernisation: Rules to update Fortran code to make use of newer features
  beyond the recommendations of the standard.
- Style: Rules to make Fortran code more readable and help adhere to a common
  set of coding conventions.
- Portability: Rules to avoid compiler- or platform-specific features in favour
  of portable alternatives.

As an example, the correctness rule `nonportable-shortcircuit-inquiry` detects a
subtle bug in which an optional argument is both checked and used within the
same logical expression, e.g:

```f90
if (present(arg) .and. arg > 0) then
    print *, arg
end if
```

In languages like C and Python, the order of operations in a logical expression
is guaranteed to run left-to-right, so equivalent statements would be safe. A
programmer may expect similar behaviour in Fortran, but as this is not
guaranteed by the standard, it is possible that this could result in a reference
to invalid data and a critical error. Running Fortitude over this code with the
rule activated delivers a diagnostic message to the user:

```console
test.f90:12:32: C161 variable inquiry `present(arg)` and use in same logical expression
   |
12 |         if (present(arg) .and. arg > 0) then
   |                                ^^^ C161
13 |             print *, arg
14 |         end if
   |

fortitude: 1 files scanned.
Number of errors: 1
```

The use of a linter is especially beneficial when working on large projects with
multiple developers, as it allows the team to enforce a consistent coding style.
To aid in this, the set of linting rules and other customisations may be set in
a configuration file and saved at a project level. Fortitude also offers a
pre-commit hook so teams can ensure that code is passing all checks before being
committed to a repository.

Fortitude may be used alongside other tooling in the Fortran ecosystem. For
example, its settings may be specified in the `fpm.toml` files used by the
Fortran Package Manager (FPM) [@fpm]. Fortitude’s editor integration using the
Language Server Protocol (LSP), by which it can provide suggestions inline with
the user’s code, is compatible with that of FortLS [@fortls], a plugin that can
aid users in navigating a project. It can also be used alongside the fprettify
[@fprettify] code formatter without generating conflicts.

# State of the field

Fortitude was developed due to notable deficiencies in existing open-source
Fortran linting tools, most of which are unmaintained, lacking in features, use
unreliable Fortran parsers, or have poor performance. The decision to write a
new linter rather than contribute to existing solutions was inspired by the much
higher quality of linters in other languages; it was easier to adapt those
solutions to work with Fortran than to upgrade the existing Fortran linters to
meet the state-of-the-art. Fortitude borrows many language-agnostic assets from
the Python linter Ruff [@ruff], including much of the command-line interface,
output formats, and configuration file design. While these structural elements
could be reused directly, the linting rules themselves had to be written
specifically for Fortran.

Other popular open-source Fortran linting tools include:

- CamFort: Primarily a code refactoring tool, it features some linting rules
  [@Orchard:2013]. Written in Haskell, it features its own Fortran parser.
- Flint: A linter written in Python. Uses lizard to analyse cyclomatic
  complexity and related metrics, and regular expressions otherwise [@flint].
- Stylist: A linter written in Python and using FortranParser to generate an
  AST [@stylist].

To compare Fortitude’s performance to these tools, each was used to lint 72
files in the GS2 source code, a turbulence modelling tool used in
magnetically-confined fusion energy research [@gs2]. Each tool was set up as
follows:

- Fortitude v0.8.0: Activated all 87 rules.
- Stylist v0.5.dev (latest on GitHub): Activated all 11 rules.
- CamFort v1.3.dev (latest on GitHub): Activated the 5 rules in the
  `basic-checks` command.
- Flint v0.7.1: Activated the default set of 38 rules. One file had to be
  excluded due to a parse error.

The runtime of each tool is shown in \autoref{fig:performance}. Despite
implementing many more checks than the alternatives, Fortitude was found to run
between 50 and 100 times as quickly than its main competitors.

![Time taken for Fortran linters to lint 72 files in the GS2 project.\label{fig:performance}](performance_plot.pdf)

Further work is needed to bring Fortitude up to parity with the most popular
linting tools in other languages. For instance, despite borrowing heavily from
Ruff [@ruff], Fortitude does not implement a code formatting mode, and Ruff
implements over 900 rules compared to the 87 rules in Fortitude v0.8.0. It also
lacks some advanced checks that linters based on compilers can perform due to
their ability to access more semantic information within a project, such as
clang-tidy for C++ [@clangtidy].

# Software design

As linting is effectively a solved problem in other languages, the design of
Fortitude was guided by the reuse of existing resources. The core architecture
and user interface of Fortitude was inspired primarily by Ruff [@ruff], a
widely-used Python linter, and, where possible, Ruff’s language-agnostic
features were directly repurposed. This reuse of a tried-and-tested design
permitted much more rapid development than would otherwise be possible, and
resulted in a user experience that can be readily understood by programmers who
are already familiar with Ruff.

However, the core checking loop of Fortitude differs to that of Ruff. Most rules
that operate over the Concrete Syntax Tree (CST) inherit a common trait that
requires them to specify their 'entrypoints': the CST node types on which they
should be activated. When Fortitude scans the CST of any Fortran files, it
checks whether the user has requested the activation of any rules that start on
each node type it encounters, and runs each of these checks in turn. This way,
Fortitude only needs to perform a single pass over the CST for all activated
rules, rather than performing a full pass per rule. This is largely how it
achieves such high performance.

The generation of the CST from a Fortran file is no simple task, especially
given the high degree of backwards compatibility in the Fortran standards.
Rather than writing a Fortran parser from scratch, this was achieved using the
TreeSitter parsing framework, which provides a fast and robust solution with
Rust bindings [@treesitter]. The Fortran extension, tree-sitter-fortran, has
itself received numerous upgrades thanks to the experience gained working on
Fortitude [@treesitterfortran].

# Research impact statement

Fortitude began life as an in-house tool to aid in the refactoring of legacy
Fortran codes within plasma physics, and is now used for code quality control by
teams all over the world, with users ranging from individual researchers to
government institutions. The earliest known external use was for FTorch
[@Atkinson:2025], a library for running neural network models in Fortran. The
Met Office uses Fortitude for various projects, including LFRic [@lfric], CASIM
[@casim], and Socrates [@socrates]. It is also used by the quantum chemistry and
solid state physics package cp2k [@cp2k] and the stellar astrophysics package
MESA [@paxton:2011].

The number of contributors to Fortitude has grown from the original two authors
to over 19, and more community members have raised feature requests and bug
reports. It receives over 4,000 downloads per month via PyPI, and a further
unknown number of downloads of platform-specific binaries using a provided
installer script. With over 190 GitHub stars, Fortitude appears to be the most
popular open-source Fortran linting tool currently in use.

# AI usage disclosure

Some of the code in Fortitude was created with the assistance of the generative
AI tool GitHub Copilot, but none has been generated wholesale, and all code has
been verified by at least two human developers prior to deployment. The software
is thoroughly tested, and all tests have been designed manually. Besides
spelling and grammar checking, this paper was written without AI assistance.

# Acknowledgements

Fortitude's development has been funded by the PlasmaFAIR project.

The authors are also grateful for the support of the Software Sustainabiliy
Institute.

We acknowledge contributions from Lawrence Dior at the University of York, Jack
Atkinson at the University of Cambridge, and Andrew Browne and Austen Rainer at
Queen's University Belfast.

# References
