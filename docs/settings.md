# Settings

## Top-level
#### [`include`](#include) {: #include }

A list of file patterns to include when linting.

Inclusion are based on globs, and should be single-path patterns, like
`*.f90`, to include any file with the `.f90` extension.

For more information on the glob syntax, refer to the [`globset` documentation](https://docs.rs/globset/latest/globset/#syntax).

!!! info "_Introduced in 0.7.6_"

**Default value**: `["*.f90", "*.F90", "*.f95", "*.F95", "*.f03", "*.F03", "*.f08", "*.F08", "*.f18", "*.F18", "*.f23", "*.F23"]`

**Type**: `list[str]`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude]
    include = ["*.f90", "*.F90"]
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    
    include = ["*.f90", "*.F90"]
    ```

---

### `check`

Configures how Fortitude checks your code.

Options specified in the `check` section take precedence over the deprecated top-level settings.

#### [`exclude`](#check_exclude) {: #check_exclude }
<span id="exclude"></span>

A list of file patterns to exclude from formatting and linting.

Exclusions are based on globs, and can be either:

- Single-path patterns, like `build` (to exclude any directory named
  `build` in the tree), `foo.f90` (to exclude any file named `foo.f90`),
  or `foo_*.f90` (to exclude any file matching `foo_*.f90`).
- Relative patterns, like `directory/foo.f90` (to exclude that specific
  file) or `directory/*.f90` (to exclude any Fortran files in
  `directory`). Note that these paths are relative to the project root
  (e.g., the directory containing your `fpm.toml`).

For more information on the glob syntax, refer to the [`globset` documentation](https://docs.rs/globset/latest/globset/#syntax).

Note that you'll typically want to use
[`extend-exclude`](#extend-exclude) to modify the excluded paths.

**Default value**: `[".git", ".git-rewrite", ".hg", ".svn", "venv", ".venv", "pyenv", ".pyenv", ".eggs", "site-packages", ".vscode", "build", "_build", "dist", "_dist"]`

**Type**: `list[str]`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    exclude = [".venv"]
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    exclude = [".venv"]
    ```

---

#### [`extend-exclude`](#check_extend-exclude) {: #check_extend-exclude }
<span id="extend-exclude"></span>

A list of file patterns to omit from formatting and linting, in addition to those
specified by [`exclude`](#exclude).

Exclusions are based on globs, and can be either:

- Single-path patterns, like `build` (to exclude any directory named
  `build` in the tree), `foo.f90` (to exclude any file named `foo.f90`),
  or `foo_*.f90` (to exclude any file matching `foo_*.f90`).
- Relative patterns, like `directory/foo.f90` (to exclude that specific
  file) or `directory/*.f90` (to exclude any Fortran files in
  `directory`). Note that these paths are relative to the project root
  (e.g., the directory containing your `fpm.toml`).

For more information on the glob syntax, refer to the [`globset` documentation](https://docs.rs/globset/latest/globset/#syntax).

**Default value**: `[]`

**Type**: `list[str]`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    # In addition to the standard set of exclusions, omit all tests, plus a specific file.
    extend-exclude = ["tests", "src/bad.f90"]
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    # In addition to the standard set of exclusions, omit all tests, plus a specific file.
    extend-exclude = ["tests", "src/bad.f90"]
    ```

---

#### [`extend-select`](#check_extend-select) {: #check_extend-select }
<span id="extend-select"></span>

A list of rule codes or prefixes to enable, in addition to those
specified by [`select`](#check_select).

**Default value**: `[]`

**Type**: `list[RuleSelector]`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    # On top of the current `select` rules, enable missing-intent (`T031`) and readability rules (`R`).
    extend-select = ["T031", "R"]
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    # On top of the current `select` rules, enable missing-intent (`T031`) and readability rules (`R`).
    extend-select = ["T031", "R"]
    ```

---

#### [`file-extensions`](#check_file-extensions) {: #check_file-extensions }
<span id="file-extensions"></span>

!!! warning "Deprecated"
    This option has been deprecated in 0.7.6. The `file_extensions` option is now deprecated in favour of the top-level [`include`](#include). Please update your configuration to use the [`include`](#include) instead.

A list of file extensions to check

**Default value**: `["f90", "F90", "f95", "F95", "f03", "F03", "f08", "F08", "f18", "F18", "f23", "F23"]`

**Type**: `list[str]`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    file-extensions = ["f90", "fpp"]
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    file-extensions = ["f90", "fpp"]
    ```

---

#### [`files`](#check_files) {: #check_files }
<span id="files"></span>

!!! warning "Deprecated"
    This option has been deprecated in 0.7.6. The `files` option is now deprecated in favour of the top-level [`include`](#include). Please update your configuration to use the [`include`](#include) instead.

A list of file patterns to include when linting.

Inclusion are based on globs, and should be single-path patterns, like
`*.f90`, to include any file with the `.f90` extension.

For more information on the glob syntax, refer to the [`globset` documentation](https://docs.rs/globset/latest/globset/#syntax).

**Default value**: `["."]`

**Type**: `list[str]`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    files = ["foo.f90"]
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    files = ["foo.f90"]
    ```

---

#### [`fix`](#check_fix) {: #check_fix }
<span id="fix"></span>

Enable fix behavior by-default when running `fortitude` (overridden
by the `--fix` and `--no-fix` command-line flags).
Only includes automatic fixes unless `--unsafe-fixes` is provided.

**Default value**: `false`

**Type**: `bool`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    fix = true
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    fix = true
    ```

---

#### [`fix-only`](#check_fix-only) {: #check_fix-only }
<span id="fix-only"></span>

Like [`fix`](#fix), but disables reporting on leftover violation. Implies [`fix`](#fix).

**Default value**: `false`

**Type**: `bool`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    fix-only = true
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    fix-only = true
    ```

---

#### [`force-exclude`](#check_force-exclude) {: #check_force-exclude }
<span id="force-exclude"></span>

Whether to enforce [`exclude`](#exclude) and [`extend-exclude`](#extend-exclude) patterns,
even for paths that are passed to Fortitude explicitly. Typically, Fortitude will lint
any paths passed in directly, even if they would typically be
excluded. Setting `force-exclude = true` will cause Fortitude to
respect these exclusions unequivocally.

This is useful for CI jobs which might explicitly pass all changed
files, regardless of whether they're marked as excluded by Fortitude's
own settings.

**Default value**: `false`

**Type**: `bool`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    force-exclude = true
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    force-exclude = true
    ```

---

#### [`ignore`](#check_ignore) {: #check_ignore }
<span id="ignore"></span>

A list of rule codes or prefixes to ignore. Prefixes can specify exact
rules (like `T003` or `superfluous-implicit-none`), entire categories
(like `T` or `typing`), or anything in between.

When breaking ties between enabled and disabled rules (via `select` and
`ignore`, respectively), more specific prefixes override less
specific prefixes.

**Default value**: `[]`

**Type**: `list[RuleSelector]`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    ignore = ["superfluous-implicit-none"]
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    ignore = ["superfluous-implicit-none"]
    ```

---

#### [`line-length`](#check_line-length) {: #check_line-length }
<span id="line-length"></span>

The line length to use when enforcing long-lines violations (like `S001`).

The length is determined by the number of characters per line, except for lines containing East Asian characters or emojis.
For these lines, the [unicode width](https://unicode.org/reports/tr11/) of each character is added up to determine the length.

**Default value**: `100`

**Type**: `int`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    # Allow lines to be as long as 120.
    line-length = 120
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    # Allow lines to be as long as 120.
    line-length = 120
    ```

---

#### [`output-format`](#check_output-format) {: #check_output-format }
<span id="output-format"></span>

The style in which violation messages should be formatted: `"full"` (default)
(shows source), `"concise"`, `"grouped"` (group messages by file), `"json"`
(machine-readable), `"junit"` (machine-readable XML), `"github"` (GitHub
Actions annotations), `"gitlab"` (GitLab CI code quality report),
`"pylint"` (Pylint text format) or `"azure"` (Azure Pipeline logging commands).

**Default value**: `"full"`

**Type**: `"full" | "concise" | "grouped" | "json" | "junit" | "github" | "gitlab" | "pylint" | "azure"`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    # Group violations by containing file.
    output-format = "grouped"
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    # Group violations by containing file.
    output-format = "grouped"
    ```

---

#### [`per-file-ignores`](#check_per-file-ignores) {: #check_per-file-ignores }
<span id="per-file-ignores"></span>

A list of mappings from file pattern to rule codes or prefixes to
exclude, when considering any matching files. An initial '!' negates
the file pattern.

**Default value**: `{}`

**Type**: `dict[str, list[RuleSelector]]`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check.per-file-ignores]
    # Ignore `T003` (superfluous implicit none) in all `test.f90` files, and in `path/to/file.f90`.
    "test.f90" = ["T003"]
    "path/to/file.f90" = ["T003"]
    # Ignore `P` rules everywhere except for the `src/` directory.
    "!src/**.f90" = ["P"]
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check.per-file-ignores]
    # Ignore `T003` (superfluous implicit none) in all `test.f90` files, and in `path/to/file.f90`.
    "test.f90" = ["T003"]
    "path/to/file.f90" = ["T003"]
    # Ignore `P` rules everywhere except for the `src/` directory.
    "!src/**.f90" = ["P"]
    ```

---

#### [`preview`](#check_preview) {: #check_preview }
<span id="preview"></span>

Whether to enable preview mode. When preview mode is enabled, Fortitude will
use unstable rules, fixes, and formatting.

**Default value**: `false`

**Type**: `bool`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    # Enable preview features.
    preview = true
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    # Enable preview features.
    preview = true
    ```

---

#### [`progress-bar`](#check_progress-bar) {: #check_progress-bar }
<span id="progress-bar"></span>

Progress bar settings.
Options are "off" (default), "ascii", and "fancy"

**Default value**: `off`

**Type**: `str`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check.progress-bar]
    # Enable unicode progress bar
    progress-bar = "fancy"
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check.progress-bar]
    # Enable unicode progress bar
    progress-bar = "fancy"
    ```

---

#### [`respect-gitignore`](#check_respect-gitignore) {: #check_respect-gitignore }
<span id="respect-gitignore"></span>

Whether to automatically exclude files that are ignored by `.ignore`,
`.gitignore`, `.git/info/exclude`, and global `gitignore` files.
Enabled by default.

**Default value**: `true`

**Type**: `bool`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    respect-gitignore = false
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    respect-gitignore = false
    ```

---

#### [`select`](#check_select) {: #check_select }
<span id="select"></span>

A list of rule codes or prefixes to enable. Prefixes can specify exact
rules (like `T003` or `superfluous-implicit-none`), entire categories
(like `T` or `typing`), or anything in between.

When breaking ties between enabled and disabled rules (via `select` and
`ignore`, respectively), more specific prefixes override less
specific prefixes.

**Default value**: `["E", "F", "S", "T", "OB", "P", "M", "IO", "R", "B"]`

**Type**: `list[RuleSelector]`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    # Only check errors and obsolescent features
    select = ["E", "OB"]
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    # Only check errors and obsolescent features
    select = ["E", "OB"]
    ```

---

#### [`show-fixes`](#check_show-fixes) {: #check_show-fixes }
<span id="show-fixes"></span>

Whether to show an enumeration of all fixed lint violations
(overridden by the `--show-fixes` command-line flag).

**Default value**: `false`

**Type**: `bool`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    # Enumerate all fixed violations.
    show-fixes = true
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    # Enumerate all fixed violations.
    show-fixes = true
    ```

---

#### [`unsafe-fixes`](#check_unsafe-fixes) {: #check_unsafe-fixes }
<span id="unsafe-fixes"></span>

Enable application of unsafe fixes.
If excluded, a hint will be displayed when unsafe fixes are available.
If set to false, the hint will be hidden.

**Default value**: `null`

**Type**: `bool`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    unsafe-fixes = true
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    unsafe-fixes = true
    ```

---

### `check.exit-unlabelled-loops`

Options for the `exit-or-cycle-in-unlabelled-loops` rule

#### [`allow-unnested-loops`](#check_exit-unlabelled-loops_allow-unnested-loops) {: #check_exit-unlabelled-loops_allow-unnested-loops }
<span id="allow-unnested-loops"></span>

Whether to check for `exit`/`cycle` in unlabelled loops only if the loop has at
least one level of nesting. With this setting off (default), the following will
raise a warning, and with it on, it won't:

```f90
do i = 1, 100
    if (i == 50) exit
end do
```

**Default value**: `false`

**Type**: `bool`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check.exit-unlabelled-loops]
    allow-unnested-loops = true
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check.exit-unlabelled-loops]
    allow-unnested-loops = true
    ```

---

### `check.invalid-tab`

Options for `invalid-tab` rule

#### [`indent-width`](#check_invalid-tab_indent-width) {: #check_invalid-tab_indent-width }
<span id="indent-width"></span>

The number of spaces to replace tabs with.

**Default value**: `4`

**Type**: `int`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check.invalid-tab]
    indent-width = 2
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check.invalid-tab]
    indent-width = 2
    ```

---

### `check.keyword-whitespace`

Options for the `keyword-missing-space` and `keyword-has-whitespace` rules

#### [`goto-with-space`](#check_keyword-whitespace_goto-with-space) {: #check_keyword-whitespace_goto-with-space }
<span id="goto-with-space"></span>

Whether to enforce the use of `go to` instead of `goto`.

**Default value**: `false`

**Type**: `bool`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check.keyword-whitespace]
    goto-with-space = true
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check.keyword-whitespace]
    goto-with-space = true
    ```

---

#### [`inout-with-space`](#check_keyword-whitespace_inout-with-space) {: #check_keyword-whitespace_inout-with-space }
<span id="inout-with-space"></span>

Whether to enforce the use of `in out` instead of `inout`.

**Default value**: `false`

**Type**: `bool`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check.keyword-whitespace]
    inout-with-space = true
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check.keyword-whitespace]
    inout-with-space = true
    ```

---

### `check.portability`

Options for the portability rules

#### [`allow-cray-file-units`](#check_portability_allow-cray-file-units) {: #check_portability_allow-cray-file-units }
<span id="allow-cray-file-units"></span>

Whether to allow file units of `100`, `101`, `102` in `read/write` statements
for [`non-portable-io-unit`](rules/non-portable-io-unit.md). The Cray
compiler pre-connects these to `stdin`, `stdout`, and `stderr`,
respectively. However, if you are `open`-ing these units explicitly, you may
wish to switch this to `true` -- but see also
[`magic-io-unit`](rules/magic-io-unit.md).

**Default value**: `false`

**Type**: `bool`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check.portability]
    allow-cray-file-units = true
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check.portability]
    allow-cray-file-units = true
    ```

---

### `check.strings`

Options for the string literal rules

#### [`quotes`](#check_strings_quotes) {: #check_strings_quotes }
<span id="quotes"></span>

Quote style to prefer for string literals (either "single" or "double").

**Default value**: `"double"`

**Type**: `"single" | "double"`

**Example usage**:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check.strings]
    quotes = "single"
    ```
=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check.strings]
    quotes = "single"
    ```

---

