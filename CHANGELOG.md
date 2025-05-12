# Changelog

## 0.7.3

This release adds 5 new rules along with some minor bug fixes and other improvements.

### Rule changes

- Add rule `missing-end-label` ([#407](https://github.com/PlasmaFAIR/fortitude/pull/407))
- Add rule `single-quote-string` ([#423](https://github.com/PlasmaFAIR/fortitude/pull/423))
- Add rule `misleading-inline-if-semicolon` ([#426](https://github.com/PlasmaFAIR/fortitude/pull/426))
- Add rule `misleading-inline-if-continuation` ([#428](https://github.com/PlasmaFAIR/fortitude/pull/428))
- Add rule `nonportable-shortcircuit-inquiry` ([#431](https://github.com/PlasmaFAIR/fortitude/pull/431))
- Add rule `split-escaped-quote` ([#438](https://github.com/PlasmaFAIR/fortitude/pull/438))
- Permit `value` dummy args in `missing-intent` ([#421](https://github.com/PlasmaFAIR/fortitude/pull/421))

### Bug fixes

- Don't raise `SyntaxError` for missing nodes if rule not enabled ([#430](https://github.com/PlasmaFAIR/fortitude/pull/430))

### Documentation

- Clarify `line-too-long` behaviour for long strings and comments ([#434](https://github.com/PlasmaFAIR/fortitude/pull/434))

## 0.7.2

### Rule changes

- Add rule `double-precision-literal` ([#390](https://github.com/PlasmaFAIR/fortitude/pull/390))
- Add rule to detect missing function result clause ([#386](https://github.com/PlasmaFAIR/fortitude/pull/386))
- Add rule to flag implicit save in pointer initialization ([#396](https://github.com/PlasmaFAIR/fortitude/pull/396))
- Add rules preferring multi-word keywords to include spaces ([#393](https://github.com/PlasmaFAIR/fortitude/pull/393))

### Documentation

- Fix `fpm.toml` examples in `docs/settings.md` ([#387](https://github.com/PlasmaFAIR/fortitude/pull/387))

### Other changes

- Raise `SyntaxError` for missing nodes ([#394](https://github.com/PlasmaFAIR/fortitude/pull/394))

## 0.7.1

This is a bug-fix release, with one new rule. Thanks to our new
contributors for improving the documentation, and packaging for AUR!

### Rule changes

- Add rule for `exit`/`cycle` in unlabelled loop ([#364](https://github.com/PlasmaFAIR/fortitude/pull/364))
- Expand rule for `character*(*)` to handle `character*N`, `character*(:)`, and `character*(expression)` ([#354](https://github.com/PlasmaFAIR/fortitude/pull/354))

### Bug fixes

- Don't print diagnostics in `--fix-only` mode ([#359](https://github.com/PlasmaFAIR/fortitude/pull/359))
- Fix `--statistics` reporting for unsafe fixes ([#368](https://github.com/PlasmaFAIR/fortitude/pull/368))
- Fix false positive `real-implicit-kind` in `type` declaration ([#378](https://github.com/PlasmaFAIR/fortitude/pull/378))
- Move `default-public-accessibility` highlight to `public` statement ([#351](https://github.com/PlasmaFAIR/fortitude/pull/351))
- Write to stdout even when no fixes are applied in stdin mode ([#358](https://github.com/PlasmaFAIR/fortitude/pull/358))

### Documentation

- Update introduction to `README.dev.md` ([#360](https://github.com/PlasmaFAIR/fortitude/pull/360))
- docs(index): fix link to best practices ([#350](https://github.com/PlasmaFAIR/fortitude/pull/350))
- docs: add package manager install instructions ([#377](https://github.com/PlasmaFAIR/fortitude/pull/377))
- docs: fix links to best practice ([#357](https://github.com/PlasmaFAIR/fortitude/pull/357))
- fixing some spelling and grammar ([#352](https://github.com/PlasmaFAIR/fortitude/pull/352))

### Other changes

- Add pre-commit hooks ([#376](https://github.com/PlasmaFAIR/fortitude/pull/376))

## 0.7.0

This release features 25 new rules, many more options for controlling
exactly which rules are enabled anywhere from per-file to per-line,
and some useful command line options.

### Breaking changes

We've reorganised a lot of the rules and categories. Rule and category
redirects should minimise any breakages, but you should update your
config appropriately.

We have also changed the rules that are on by default. This may mean
you now have to explicitly enable some rules there were previously
checked, and you may find more warnings being raised from new rules.

The rule `statement-function` has been temporarily removed while we
work to reduce false positives.

### Stabilisation

The following rules have been stabilised and are no longer in preview:

- [`missing-accessibility-statement`](https://fortitude.readthedocs.io/en/stable/rules/missing-accessibility-statement/) (`C131`)
- [`default-public-accessibility`](https://fortitude.readthedocs.io/en/stable/rules/default-public-accessibility/) (`C132`)
- [`implicit-external-procedures`](https://fortitude.readthedocs.io/en/stable/rules/implicit-external-procedures/) (`C003`)

### Rule changes

- Add `missing-default-case` ([#240](https://github.com/PlasmaFAIR/fortitude/pull/240))
- Add check for use of specific names for intrinsic functions ([#254](https://github.com/PlasmaFAIR/fortitude/pull/254))
- Add rule `magic-number-in-array-size` ([#236](https://github.com/PlasmaFAIR/fortitude/pull/236))
- Add rule `missing-action-specifier` and `Io` rule category ([#230](https://github.com/PlasmaFAIR/fortitude/pull/230))
- Add rule for deleted feature `pause` statements ([#304](https://github.com/PlasmaFAIR/fortitude/pull/304))
- Add rule to find computed go to statements ([#264](https://github.com/PlasmaFAIR/fortitude/pull/264))
- Add rule to find missing intrinsic specifiers in `use` statements ([#253](https://github.com/PlasmaFAIR/fortitude/pull/253))
- Add rule to flag trailing backslash ([#311](https://github.com/PlasmaFAIR/fortitude/pull/311))
- Add rule to test for uninitialized pointers in derived types ([#299](https://github.com/PlasmaFAIR/fortitude/pull/299))
- Add rules for magic/non-portable IO units ([#239](https://github.com/PlasmaFAIR/fortitude/pull/239))
- Add rules for multiple modules/programs in same file; Add rule for `include` statements ([#268](https://github.com/PlasmaFAIR/fortitude/pull/268))
- Add rules for unused/duplicated/redirected allow comments ([#334](https://github.com/PlasmaFAIR/fortitude/pull/334))
- Rule: `multiple-statements-per-line` ([#246](https://github.com/PlasmaFAIR/fortitude/pull/246))
- Temporarily remove `statement-function` ([#339](https://github.com/PlasmaFAIR/fortitude/pull/339))
- Warn about invalid rules in allow-comments ([#266](https://github.com/PlasmaFAIR/fortitude/pull/266))
- Suggest `iso_fortran_env` parameters for literal kinds ([#245](https://github.com/PlasmaFAIR/fortitude/pull/245))

### CLI

- Add `--exit-zero` and `--exit-non-zero-on-fix` CLI options ([#328](https://github.com/PlasmaFAIR/fortitude/pull/328))
- Add `--statistics` CLI flag to show counts of violations ([#330](https://github.com/PlasmaFAIR/fortitude/pull/330))
- Add ability to read from stdin ([#307](https://github.com/PlasmaFAIR/fortitude/pull/307))
- Add command to generate shell completion scripts ([#340](https://github.com/PlasmaFAIR/fortitude/pull/340))
- Add logging framework and `--verbose/quiet/silent` flags ([#274](https://github.com/PlasmaFAIR/fortitude/pull/274))

### Configuration

- Add "allow" comments ([#242](https://github.com/PlasmaFAIR/fortitude/pull/242))
- Add options to exclude files ([#238](https://github.com/PlasmaFAIR/fortitude/pull/238))
- Add per-file-ignores ([#232](https://github.com/PlasmaFAIR/fortitude/pull/232))
- Decouple CLI and config file ([#312](https://github.com/PlasmaFAIR/fortitude/pull/312))
- Mark rules as `Default` or `Optional` in `map_codes` ([#324](https://github.com/PlasmaFAIR/fortitude/pull/324))
- Recategorise rules ([#329](https://github.com/PlasmaFAIR/fortitude/pull/329))
- Respect `.gitignore` ([#285](https://github.com/PlasmaFAIR/fortitude/pull/285))
- Warn when selected rules are ignored for some reason ([#305](https://github.com/PlasmaFAIR/fortitude/pull/305))

### Bug fixes

- Bugfix: Catch negative exponents in `missing-kind-suffix` ([#244](https://github.com/PlasmaFAIR/fortitude/pull/244))
- Bugfix: Count files scanned and files skipped correctly ([#309](https://github.com/PlasmaFAIR/fortitude/pull/309))
- Bugfix: `line-length` and `file-extensions` ignored in toml files ([#251](https://github.com/PlasmaFAIR/fortitude/pull/251))
- Bugfix: fix `get_files` force excluding with directories ([#296](https://github.com/PlasmaFAIR/fortitude/pull/296))
- Bugfix: make `specific-names` an unsafe fix ([#263](https://github.com/PlasmaFAIR/fortitude/pull/263))
- Catch syntax errors introduced by fixes ([#227](https://github.com/PlasmaFAIR/fortitude/pull/227))
- Don't raise linter violations or apply fixes when syntax errors are detected ([#336](https://github.com/PlasmaFAIR/fortitude/pull/336))
- Fix Windows testing issues ([#318](https://github.com/PlasmaFAIR/fortitude/pull/318))
- Fix some issues with newlines in fixes on Windows ([#315](https://github.com/PlasmaFAIR/fortitude/pull/315))
- Handle broken pipes ([#290](https://github.com/PlasmaFAIR/fortitude/pull/290))

### Documentation

- Add guide for adding new rules to `README.dev.md` ([#300](https://github.com/PlasmaFAIR/fortitude/pull/300))
- T042: Fix example code in assumed-size-character-intent rule ([#234](https://github.com/PlasmaFAIR/fortitude/pull/234))

## 0.6.2

### Bug fixes

- Fix `Node::edit_replacement` eating newlines ([#225](https://github.com/PlasmaFAIR/fortitude/pull/225))

## 0.6.1

### Bug fixes

- Bugfix: `star-kind` fix set to unsafe ([#218](https://github.com/PlasmaFAIR/fortitude/pull/218))

### Documentation

- Fix link to rules table in Readme ([#217](https://github.com/PlasmaFAIR/fortitude/pull/217))

## 0.6.0

### Preview features

- Move statement-function rule to preview set ([#164](https://github.com/PlasmaFAIR/fortitude/pull/164))

### Rule changes

- Add automatic fixes ([#181](https://github.com/PlasmaFAIR/fortitude/pull/181))
- Add check for `external` procedures ([#195](https://github.com/PlasmaFAIR/fortitude/pull/195))
- Add fix for `deprecated-relational-operators` ([#182](https://github.com/PlasmaFAIR/fortitude/pull/182))
- Add fix for `old-style-array-literal` ([#183](https://github.com/PlasmaFAIR/fortitude/pull/183))
- Add fix for `star-kinds` ([#186](https://github.com/PlasmaFAIR/fortitude/pull/186))
- Add fix for `trailing-whitespace` ([#187](https://github.com/PlasmaFAIR/fortitude/pull/187))
- Add fix for `unnamed-end-statement` ([#185](https://github.com/PlasmaFAIR/fortitude/pull/185))
- Add fix for missing label on `exit`/`cycle` ([#184](https://github.com/PlasmaFAIR/fortitude/pull/184))
- Add rule `common-block` ([#165](https://github.com/PlasmaFAIR/fortitude/pull/165))
- Add rule for implicit external procedures ([#192](https://github.com/PlasmaFAIR/fortitude/pull/192))
- Add rule for incorrect whitespace before comment ([#194](https://github.com/PlasmaFAIR/fortitude/pull/194))
- Add rule for missing `private` statements in modules ([#150](https://github.com/PlasmaFAIR/fortitude/pull/150))
- Add rule for missing double-colon in variable decls ([#171](https://github.com/PlasmaFAIR/fortitude/pull/171))
- Add rule for obsolescent `entry` statement ([#169](https://github.com/PlasmaFAIR/fortitude/pull/169))
- Add rule for statement functions ([#162](https://github.com/PlasmaFAIR/fortitude/pull/162))
- Rename `external-function` to `procedure-not-in-module` ([#200](https://github.com/PlasmaFAIR/fortitude/pull/200))

### CLI

- Add `--preview` option ([#151](https://github.com/PlasmaFAIR/fortitude/pull/151))
- Add fancy progress bar ([#145](https://github.com/PlasmaFAIR/fortitude/pull/145))
- Don't require argument for CLI `--preview` ([#153](https://github.com/PlasmaFAIR/fortitude/pull/153))
- Print git commit and extra info with `--version` ([#174](https://github.com/PlasmaFAIR/fortitude/pull/174))
- Show preview rules in `explain` ([#154](https://github.com/PlasmaFAIR/fortitude/pull/154))

### Configuration

- Add (many) more output formats ([#177](https://github.com/PlasmaFAIR/fortitude/pull/177))

### Bug fixes

- Bugfix: `DiagnosticMessages` were created on pre-transformed file ([#191](https://github.com/PlasmaFAIR/fortitude/pull/191))
- Bugfix: report number of files scanned ([#173](https://github.com/PlasmaFAIR/fortitude/pull/173))
- Bugfix: utf-8 multi-byte characters breaking diagnostic output ([#198](https://github.com/PlasmaFAIR/fortitude/pull/198))
- Less aggressive warnings for `no-real-suffix` ([#156](https://github.com/PlasmaFAIR/fortitude/pull/156))

### Documentation

- Add basic docs ([#210](https://github.com/PlasmaFAIR/fortitude/pull/210))

## 0.5.1

### Performance

- Better parallel performance ([#137](https://github.com/PlasmaFAIR/fortitude/pull/137))

### Bug fixes

- Don't warn on missing intent for `procedure` arguments ([#139](https://github.com/PlasmaFAIR/fortitude/pull/139))

### Documentation

- Clarify check help text for recursive file searches ([#143](https://github.com/PlasmaFAIR/fortitude/pull/143))

## 0.5.0

### Performance

- Parallelise check ([#129](https://github.com/PlasmaFAIR/fortitude/pull/129))

### CLI

- Add a success message when all checks are passed and a summary of number of files scanned ([#104](https://github.com/PlasmaFAIR/fortitude/pull/104))
- Make `explain` arguments positional ([#113](https://github.com/PlasmaFAIR/fortitude/pull/113))
- Select categories by name ([#112](https://github.com/PlasmaFAIR/fortitude/pull/112))
- Select rules by name ([#111](https://github.com/PlasmaFAIR/fortitude/pull/111))

### Bug fixes

- Add fix for superfluous implicit none ([#99](https://github.com/PlasmaFAIR/fortitude/pull/99))
- Fix explain bug with RuleSelector, improve explain output ([#110](https://github.com/PlasmaFAIR/fortitude/pull/110))
- Fix reading config files ([#124](https://github.com/PlasmaFAIR/fortitude/pull/124))
- Remove deleted flag from help text ([#115](https://github.com/PlasmaFAIR/fortitude/pull/115))

### Documentation

- Update precision explanations ([#126](https://github.com/PlasmaFAIR/fortitude/pull/126))
- docs: Clarify how `check` searches for files ([#128](https://github.com/PlasmaFAIR/fortitude/pull/128))

### Other changes

- Enable more sophisticated rule selection ([#108](https://github.com/PlasmaFAIR/fortitude/pull/108))

## 0.4.0

### Configuration

- Add setting for file extensions to check ([#89](https://github.com/PlasmaFAIR/fortitude/pull/89))
- Add support for config files ([#87](https://github.com/PlasmaFAIR/fortitude/pull/87))

## 0.3.0

### Rule changes

- Add check for old-style array literals `(/.../)` ([#54](https://github.com/PlasmaFAIR/fortitude/pull/54))
- Add rule for assumed size dummy arguments ([#57](https://github.com/PlasmaFAIR/fortitude/pull/57))
- Add rule for deprecated relational operators ([#82](https://github.com/PlasmaFAIR/fortitude/pull/82))
- Add rule for initialisation in declarations ([#72](https://github.com/PlasmaFAIR/fortitude/pull/72))
- Add rule for missing `intent` attribute ([#50](https://github.com/PlasmaFAIR/fortitude/pull/50))
- Add rule for missing label on `exit`/`cycle` ([#53](https://github.com/PlasmaFAIR/fortitude/pull/53))
- Add rule for non-explicit end statements ([#84](https://github.com/PlasmaFAIR/fortitude/pull/84))
- Add rules for `character` assumed size issues ([#75](https://github.com/PlasmaFAIR/fortitude/pull/75))
- Implicit real kinds rule ([#81](https://github.com/PlasmaFAIR/fortitude/pull/81))
- Print context of violations ([#45](https://github.com/PlasmaFAIR/fortitude/pull/45))

### Bug fixes

- Bugfix: newline printed for files without rule violations ([#42](https://github.com/PlasmaFAIR/fortitude/pull/42))

## 0.2.0

<!-- No changes -->
