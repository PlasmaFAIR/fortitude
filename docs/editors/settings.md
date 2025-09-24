# Settings

The Fortitude Language Server provides a set of configuration options to customise its behaviour along
with the ability to use an existing `pyproject.toml` or `fortitude.toml` file to configure the
linter. This is done by providing these settings while initialising the server.  VS Code provides a
UI to configure these settings, while other editors may require manual configuration. The
[setup](./setup.md) section provides instructions on where to place these settings as per the editor.

## Top-level

### `configuration`

The `configuration` setting allows you to configure editor-specific Fortitude behavior. This can be done
in one of the following ways:

1. **Configuration file path:** Specify the path to a `fortitude.toml` or `pyproject.toml` file that
    contains the configuration. User home directory and environment variables will be expanded.
1. **Inline JSON configuration:** Directly provide the configuration as a JSON object.

The default behavior, if `configuration` is unset, is to load the settings from the project's
configuration (a `fortitude.toml` or `pyproject.toml` in the project's directory), consistent with when
running Fortitude on the command-line.

The [`configurationPreference`](#configurationpreference) setting controls the precedence if both an
editor-provided configuration (`configuration`) and a project level configuration file are present.

#### Resolution order {: #configuration_resolution_order }

In an editor, Fortitude supports three sources of configuration, prioritised as follows (from highest to
lowest):

1. **Specific settings:** Individual settings like [`lineLength`](#linelength) or
    [`lint.select`](#select) defined in the editor
1. [**`fortitude.configuration`**](#configuration): Settings provided via the
    [`configuration`](#configuration) field (either a path to a configuration file or an inline
    configuration object)
1. **Configuration file:** Settings defined in a `fortitude.toml` or `pyproject.toml` file in the
    project's directory (if present)

For example, if the line length is specified in all three sources, Fortitude will use the value from the
[`lineLength`](#linelength) setting.

**Default value**: `null`

**Type**: `string`

**Example usage**:

_Using configuration file path:_

=== "VS Code"

    ```json
    {
        "fortitude.configuration": "~/path/to/fortitude.toml"
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          configuration = "~/path/to/fortitude.toml"
        }
      }
    }
    ```

_Using inline configuration:_

=== "VS Code"

    ```json
    {
        "fortitude.configuration": {
            "check": {
                "extend-select": ["style"],
                "exit-unlabelled-loops": {
                    "allow-unnested-loops": true
                }
            },
        }
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          configuration = {
            check = {
              ["extend-select"] = {"style"},
              ["exit-unlabelled-loops"] = "true"
            },
          }
        }
      }
    }
    ```

### `configurationPreference`

The strategy to use when resolving settings across VS Code and the filesystem. By default, editor
configuration is prioritized over `fortitude.toml` and `pyproject.toml` files.

- `"editorFirst"`: Editor settings take priority over configuration files present in the workspace.
- `"filesystemFirst"`: Configuration files present in the workspace takes priority over editor
    settings.
- `"editorOnly"`: Ignore configuration files entirely i.e., only use editor settings.

**Default value**: `"editorFirst"`

**Type**: `"editorFirst" | "filesystemFirst" | "editorOnly"`

**Example usage**:

=== "VS Code"

    ```json
    {
        "fortitude.configurationPreference": "filesystemFirst"
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          configurationPreference = "filesystemFirst"
        }
      }
    }
    ```

### `exclude`

A list of file patterns to exclude from linting. See [the documentation](../settings.md#check_exclude)
for more details.

**Default value**: `null`

**Type**: `string[]`

**Example usage**:

=== "VS Code"

    ```json
    {
        "fortitude.exclude": ["**/tests/**"]
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          exclude = ["**/tests/**"]
        }
      }
    }
    ```

### `lineLength`

The line length to use for the linter.

**Default value**: `null`

**Type**: `int`

**Example usage**:

=== "VS Code"

    ```json
    {
        "fortitude.lineLength": 100
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          lineLength = 100
        }
      }
    }
    ```

### `fixAll`

Whether to register the server as capable of handling `source.fixAll` code actions.

**Default value**: `true`

**Type**: `bool`

**Example usage**:

=== "VS Code"

    ```json
    {
        "fortitude.fixAll": false
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          fixAll = false
        }
      }
    }
    ```

### `logLevel`

The log level to use for the server.

**Default value**: `"info"`

**Type**: `"trace" | "debug" | "info" | "warn" | "error"`

**Example usage**:

=== "VS Code"

    ```json
    {
        "fortitude.logLevel": "debug"
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          logLevel = "debug"
        }
      }
    }
    ```

### `logFile`

Path to the log file to use for the server.

If not set, logs will be written to stderr.

**Default value**: `null`

**Type**: `string`

**Example usage**:

=== "VS Code"

    ```json
    {
        "fortitude.logFile": "~/path/to/fortitude.log"
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          logFile = "~/path/to/fortitude.log"
        }
      }
    }
    ```

## `codeAction`

Enable or disable code actions provided by the server.

### `fixViolation.enable`

Whether to display Quick Fix actions to autofix violations.

**Default value**: `true`

**Type**: `bool`

**Example usage**:

=== "VS Code"

    ```json
    {
        "fortitude.codeAction.fixViolation.enable": false
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          codeAction = {
            fixViolation = {
              enable = false
            }
          }
        }
      }
    }
    ```

## `check`

Settings specific to the Fortitude linter.

### `preview` {: #check_preview }

Whether to enable Fortitude's preview mode when linting.

**Default value**: `null`

**Type**: `bool`

**Example usage**:

=== "VS Code"

    ```json
    {
        "fortitude.check.preview": true
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          check = {
            preview = true
          }
        }
      }
    }
    ```

### `select`

Rules to enable by default. See [the documentation](https://docs.astral.sh/fortitude/settings/#check_select).

**Default value**: `null`

**Type**: `string[]`

**Example usage**:

=== "VS Code"

    ```json
    {
        "fortitude.check.select": ["E", "F"]
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          check = {
            select = {"E", "F"}
          }
        }
      }
    }
    ```

### `extendSelect`

Rules to enable in addition to those in [`check.select`](#select).

**Default value**: `null`

**Type**: `string[]`

**Example usage**:

=== "VS Code"

    ```json
    {
        "fortitude.check.extendSelect": ["W"]
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          check = {
            extendSelect = {"W"}
          }
        }
      }
    }
    ```

### `ignore`

Rules to disable by default. See [the documentation](https://docs.astral.sh/fortitude/settings/#check_ignore).

**Default value**: `null`

**Type**: `string[]`

**Example usage**:

=== "VS Code"

    ```json
    {
        "fortitude.check.ignore": ["E4", "E7"]
    }
    ```

=== "Neovim"

    ```lua
    require('lspconfig').fortitude.setup {
      init_options = {
        settings = {
          check = {
            ignore = {"E4", "E7"}
          }
        }
      }
    }
    ```

## VS Code specific

Additionally, the Fortitude extension provides the following settings specific to VS Code. These settings
are not used by the language server and are only relevant to the extension.

### `enable`

Whether to enable the Fortitude extension. Modifying this setting requires restarting VS Code to take effect.

**Default value**: `true`

**Type**: `bool`

**Example usage**:

```json
{
    "fortitude.enable": false
}
```

### `importStrategy`

Strategy for loading the `fortitude` executable.

- `fromEnvironment` finds Fortitude in the environment, falling back to the bundled version
- `useBundled` uses the version bundled with the extension

**Default value**: `"fromEnvironment"`

**Type**: `"fromEnvironment" | "useBundled"`

**Example usage**:

```json
{
    "fortitude.importStrategy": "useBundled"
}
```

### `interpreter`

A list of paths to Python interpreters. Even though this is a list, only the first interpreter is
used.

This setting depends on the [`fortitude.nativeServer`](#nativeserver) setting:

- If using the native server, the interpreter is used to find the `fortitude` executable when
    [`fortitude.importStrategy`](#importstrategy) is set to `fromEnvironment`.
- Otherwise, the interpreter is used to run the `fortitude-lsp` server.

**Default value**: `[]`

**Type**: `string[]`

**Example usage**:

```json
{
    "fortitude.interpreter": ["/home/user/.local/bin/python"]
}
```

### `path`

A list of path to `fortitude` executables.

The first executable in the list which is exists is used. This setting takes precedence over the
[`fortitude.importStrategy`](#importstrategy) setting.

**Default value**: `[]`

**Type**: `string[]`

**Example usage**:

```json
{
    "fortitude.path": ["/home/user/.local/bin/fortitude"]
}
```

### `trace.server`

The trace level for the language server. Refer to the [LSP
specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#traceValue)
for more information.

**Default value**: `"off"`

**Type**: `"off" | "messages" | "verbose"`

**Example usage**:

```json
{
    "fortitude.trace.server": "messages"
}
```
