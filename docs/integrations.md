# Integrations

## pre-commit

Fortitude can be used as a [pre-commit](https://pre-commit.com/) hook via
[fortitude-pre-commit](https://github.com/PlasmaFAIR/fortitude-pre-commit):

```yaml
repos:
- repo: https://github.com/PlasmaFAIR/fortitude-pre-commit
  # Fortitude version.
  rev: v0.10.0
  hooks:
    - id: fortitude
```

To enable fixes, add the [`--fix`](settings.md#fix) argument:

```yaml
repos:
- repo: https://github.com/PlasmaFAIR/fortitude-pre-commit
  # Fortitude version.
  rev: v0.10.0
  hooks:
    - id: fortitude
      args: ["--fix"]
```

Other additional arguments are also accepted:

```yaml
repos:
- repo: https://github.com/PlasmaFAIR/fortitude-pre-commit
  # Fortitude version.
  rev: v0.10.0
  hooks:
    - id: fortitude
      args: ["--fix", "--preview", "--unsafe-fixes"]
```

When running with `--fix`, Fortitude's lint hook should be placed before any code
formatters in use.

Note that the Fortitude pre-commit hook will run with the option
[`--force-exclude`](settings.md#force-exclude) switched on by default.  This
will prevent Fortitude from checking files that have been explicitly added to
your exclude list in your configuration file (`fortitude/fpm/pyproject.toml`).

## AI agent skills

AI coding assistants can be taught how to use Fortitude with an
[agent skill](https://agentskills.io/): a folder containing a `SKILL.md` file that
describes when the skill applies and what the agent should do. The agent reads the
short description up front and loads the full instructions only when a task matches.

Where a skill lives depends on your tool. For example, Claude Code reads project
skills from `.claude/skills/fortitude/SKILL.md`. Check your tool's documentation for
the right location.

Below is a minimal skill for running Fortitude:

````markdown
---
name: fortitude
description: Use this skill when linting Fortran source code with the Fortitude linter, including running checks, fixing warnings, and configuring rule selection.
---

# Linting Fortran with Fortitude

1. Check for a config file (`fortitude.toml`, `.fortitude.toml`, `fpm.toml` or
   `pyproject.toml`). If there is none, run `fortitude check` with defaults first.
2. Run `fortitude check` and read the output (file:line:column, rule code, message).
3. Look up unfamiliar rules with `fortitude explain <RULE>`.
4. Fix genuine issues in the source. Use `fortitude check --diff` to preview
   automatic fixes, then `fortitude check --fix` to apply them and review the result.
5. For deliberate exceptions, prefer narrow `! allow(...)` comments or entries in
   `ignore` over broad suppressions.
6. Re-run `fortitude check` until clean.

Unsafe fixes may change the meaning of code. Only use `--unsafe-fixes` if asked.
````

A fuller example, covering rule selection, configuration formats, preview rules and
common pitfalls, is available in the
[ICCS-SKILLS repository](https://github.com/Cambridge-ICCS/ICCS-SKILLS/blob/main/skills/fortitude/SKILL.md).

!!! note
    Fixes suggested or applied by an agent should be reviewed like any other change.
