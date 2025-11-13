# Installation

Fortitude is available as
[`fortitude-lint`](https://pypi.org/project/fortitude-lint) on PyPI:

```bash
# With uv:
uv tool install fortitude-lint@latest

# With pip:
pip install fortitude-lint
```

If you're not working in a virtual environment, these commands
typically install to `~/.local/bin` on Linux. You may need to add this
directory to your `$PATH` environment variable. If you install
Fortitude inside a virtual environment, it will already be in your
`PATH`.

Starting with version `0.7.0`, Fortitude can be installed with our
standalone installers:

```bash
# On macOS and Linux:
curl -LsSf https://github.com/PlasmaFAIR/fortitude/releases/latest/download/fortitude-installer.sh | sh

# On Windows:
powershell -c "irm https://github.com/PlasmaFAIR/fortitude/releases/latest/download/fortitude-installer.ps1 | iex"
```

These installers will tell you where they install to, but on Linux,
this is typically `~/.local/bin`. They will also tell you if you need
to modify your `$PATH` environment variable.

For **Arch Linux** and **AUR** users, Fortitude is available as [`fortitude-bin`](https://aur.archlinux.org/packages/fortitude-bin)
and can be download using `yay/paru` as

```bash
$ yay -S fortitude-bin
```
