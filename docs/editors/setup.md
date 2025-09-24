# Setup

We have specific setup instructions depending on your editor of choice. If you don't see your editor on this
list and would like a setup guide, please open an issue.

!!! note

    The setup instructions provided below are on a best-effort basis. If you encounter any issues
    while setting up the Fortitude in an editor, please [open an issue](https://github.com/PlasmaFAIR/fortitude/issues/new)
    for assistance and help in improving this documentation.

## VS Code

!!! warning

    The VS Code extension is not yet published!

Install the Fortitude extension from the [VS Code
Marketplace](https://marketplace.visualstudio.com/items?itemName=charliermarsh.fortitude). It is
recommended to have the Fortitude extension version `2024.32.0` or later to get the best experience with
the Fortitude Language Server.

For more documentation on the Fortitude extension, refer to the
[README](https://github.com/PlasmaFAIR/fortitude-vscode/blob/main/README.md) of the extension repository.

## Neovim

The [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig) plugin can be used to configure the
Fortitude Language Server in Neovim. To set it up, install
[`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig) plugin, set it up as per the
[configuration](https://github.com/neovim/nvim-lspconfig#configuration) documentation, and add the
following to your `init.lua`:

=== "Neovim 0.10 (with [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig))"

    ```lua
    require('lspconfig').fortitude.setup({
      init_options = {
        settings = {
          -- Fortitude language server settings go here
        }
      }
    })
    ```

=== "Neovim 0.11+ (with [`vim.lsp.config`](https://neovim.io/doc/user/lsp.html#vim.lsp.config()))"

    ```lua
    vim.lsp.config('fortitude', {
      init_options = {
        settings = {
          -- Fortitude language server settings go here
        }
      }
    })

    vim.lsp.enable('fortitude')
    ```

If you're using Fortitude alongside another language server (like fortls), you may want to defer to that
language server for certain capabilities, like [`textDocument/hover`](./features.md#hover):

```lua
vim.api.nvim_create_autocmd("LspAttach", {
  group = vim.api.nvim_create_augroup('lsp_attach_disable_fortitude_hover', { clear = true }),
  callback = function(args)
    local client = vim.lsp.get_client_by_id(args.data.client_id)
    if client == nil then
      return
    end
    if client.name == 'fortitude' then
      -- Disable hover in favor of fortls
      client.server_capabilities.hoverProvider = false
    end
  end,
  desc = 'LSP: Disable hover capability from Fortitude',
})
```

By default, the log level for Fortitude is set to `info`. To change the log level, you can set the
[`logLevel`](./settings.md#loglevel) setting:

```lua
require('lspconfig').fortitude.setup {
  init_options = {
    settings = {
      logLevel = 'debug',
    }
  }
}
```

By default, Fortitude will write logs to stderr which will be available in Neovim's LSP client log file
(`:lua vim.print(vim.lsp.get_log_path())`). It's also possible to divert these logs to a separate
file with the [`logFile`](./settings.md#logfile) setting.

To view the trace logs between Neovim and Fortitude, set the log level for Neovim's LSP client to `debug`:

```lua
vim.lsp.set_log_level('debug')
```

<details>
<summary>With the <a href="https://github.com/stevearc/conform.nvim"><code>conform.nvim</code></a> plugin for Neovim.</summary>

```lua
require("conform").setup({
    formatters_by_ft = {
        fortran = {
          -- To fix auto-fixable lint errors.
          "fortitude_fix",
        },
    },
})
```

</details>

<details>
<summary>With the <a href="https://github.com/mfussenegger/nvim-lint"><code>nvim-lint</code></a> plugin for Neovim.</summary>

```lua
require("lint").linters_by_ft = {
  fortran = { "fortitude" },
}
```

</details>

<details>
<summary>With the <a href="https://github.com/dense-analysis/ale">ALE</a> plugin for Neovim or Vim.</summary>

<i>Neovim (using Lua):</i>

```lua
-- Linters
vim.g.ale_linters = { fortran = { "fortitude" } }
-- Fixers
vim.g.ale_fixers = { fortran = { "fortitude", "fortitude_format" } }
```

<i>Vim (using Vimscript):</i>

```vim
" Linters
let g:ale_linters = { "fortran": ["fortitude"] }
" Fixers
let g:ale_fixers = { "fortran": ["fortitude", "fortitude_format"] }
```

For the fixers, <code>fortitude</code> will run <code>fortitude check --fix</code> (to fix all auto-fixable
problems) whereas <code>fortitude_format</code> will run <code>fortitude format</code>.

</details>

## Vim

The [`vim-lsp`](https://github.com/prabirshrestha/vim-lsp) plugin can be used to configure the Fortitude Language Server in Vim.
To set it up, install [`vim-lsp`](https://github.com/prabirshrestha/vim-lsp) plugin and register the server using the following
in your `.vimrc`:

```vim
if executable('fortitude')
    au User lsp_setup call lsp#register_server({
        \ 'name': 'fortitude',
        \ 'cmd': {server_info->['fortitude', 'server']},
        \ 'allowlist': ['fortran'],
        \ 'workspace_config': {},
        \ })
endif
```

See the `vim-lsp`
[documentation](https://github.com/prabirshrestha/vim-lsp/blob/master/doc/vim-lsp.txt) for more
details on how to configure the language server.

If you're using Fortitude alongside another LSP (like `fortls`), you may want to defer to that LSP for certain capabilities,
like [`textDocument/hover`](./features.md#hover) by adding the following to the function `s:on_lsp_buffer_enabled()`:

```vim
function! s:on_lsp_buffer_enabled() abort
    " add your keybindings here (see https://github.com/prabirshrestha/vim-lsp?tab=readme-ov-file#registering-servers)

    let l:capabilities = lsp#get_server_capabilities('fortitude')
    if !empty(l:capabilities)
      let l:capabilities.hoverProvider = v:false
    endif
endfunction
```

<details>
<summary>Fortitude can also be integrated via <a href="https://github.com/mattn/efm-langserver">efm language server</a> in just a few lines.</summary>

Following is an example config for efm to use Fortitude for linting and formatting Fortran files:

```yaml
tools:
  fortran-fortitude:
    lint-command: "fortitude check --stdin-filename ${INPUT} --output-format concise --quiet -"
    lint-stdin: true
    lint-formats:
      - "%f:%l:%c: %m"
    format-command: "fortitude format --stdin-filename ${INPUT} --quiet -"
    format-stdin: true
```

</details>

## Helix

Open the [language configuration file](https://docs.helix-editor.com/languages.html#languagestoml-files) for
Helix and add the language server as follows:

```toml
[language-server.fortitude]
command = "fortitude"
args = ["server"]
```

Then, you'll register the language server as the one to use with Fortran. If you don't already have a
language server registered to use with Fortran, add this to `languages.toml`:

```toml
[[language]]
name = "fortran"
language-servers = ["fortitude"]
```

Otherwise, if you already have `language-servers` defined, you can simply add `"fortitude"` to the list. For example,
if you already have `fortls` as a language server, you can modify the language entry as follows:

```toml
[[language]]
name = "fortran"
language-servers = ["fortitude", "fortls"]
```

!!! note

    Support for multiple language servers for a language is only available in Helix version
    [`23.10`](https://github.com/helix-editor/helix/blob/master/CHANGELOG.md#2310-2023-10-24) and later.

See the [Helix documentation](https://docs.helix-editor.com/languages.html) for more settings you can use here.

You can pass settings into `fortitude server` using `[language-server.fortitude.config.settings]`. For example:

```toml
[language-server.fortitude.config.settings]
lineLength = 80

[language-server.fortitude.config.settings.check]
select = ["correctness", "S001"]
preview = false

[language-server.fortitude.config.settings.format]
preview = true
```

By default, the log level for Fortitude is set to `info`. To change the log level, you can set the
[`logLevel`](./settings.md#loglevel) setting:

```toml
[language-server.fortitude]
command = "fortitude"
args = ["server"]

[language-server.fortitude.config.settings]
logLevel = "debug"
```

You can also divert Fortitude's logs to a separate file with the [`logFile`](./settings.md#logfile) setting.

To view the trace logs between Helix and Fortitude, pass in the `-v` (verbose) flag when starting Helix:

```sh
hx -v path/to/file.f90
```

## Kate

1. Activate the [LSP Client plugin](https://docs.kde.org/stable5/en/kate/kate/plugins.html#kate-application-plugins).
1. Setup LSP Client [as desired](https://docs.kde.org/stable5/en/kate/kate/kate-application-plugin-lspclient.html).
1. Finally, add this to `Settings` -> `Configure Kate` -> `LSP Client` -> `User Server Settings`:

```json
{
  "servers": {
    "fortran": {
      "command": ["fortitude", "server"],
      "url": "https://github.com/PlasmaFAIR/fortitude",
      "highlightingModeRegex": "^Fortran$",
      "settings": {}
    }
  }
}
```

See [LSP Client documentation](https://docs.kde.org/stable5/en/kate/kate/kate-application-plugin-lspclient.html) for more details
on how to configure the server from there.

!!! important

    Kate's LSP Client plugin does not support multiple servers for the same language.

## Sublime Text

To use Fortitude with Sublime Text, install Sublime Text's [LSP](https://github.com/sublimelsp/LSP)
and open `Preferences > Package Settings > LSP > Settings` and add the `fortitude` configuration to
the `"clients"`:

```json
{
  "clients": {
    "fortitude-lsp": {
      "enabled": true,
      "command": ["/home/peter/Codes/fortitude/target/release/fortitude", "server"],
      "selector": "source.fortran | source.modern-fortran"
    }
  }
}
```

## Emacs

Fortitude can be utilized as a language server via [`Eglot`](https://github.com/joaotavora/eglot), which is in Emacs's core:

```elisp
(add-hook 'f90-mode-hook 'eglot-ensure)
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(f90-mode . ("fortitude" "server"))))
```

You can also use Fortitude in [`lsp-mode`](https://emacs-lsp.github.io/lsp-mode/):

```elisp
(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection '("fortitude" "server"))
  :activation-fn (lsp-activate-on "fortran")
  :server-id 'fortitude
  :priority -2
  :add-on? t))
```
