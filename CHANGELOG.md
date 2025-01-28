# Changelog

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


