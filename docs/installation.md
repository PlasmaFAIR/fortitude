# Installation

Fortitude is available as
[`fortitude-lint`](https://pypi.org/project/fortitude-lint) on PyPI:

```bash
# With uv:
uv tool install fortitude-lint@latest

# With pip:
pip install fortitude-lint
```

Starting with version `0.7.0`, Fortitude can be installed with our
standalone installers:

```bash
# On macOS and Linux:
curl -LsSf https://github.com/PlasmaFAIR/fortitude/releases/latest/download/fortitude-installer.sh | sh

# On Windows:
powershell -c "irm https://github.com/PlasmaFAIR/fortitude/releases/latest/download/fortitude-installer.psi | iex"
```

For **Arch Linux** and **AUR** users, Fortitude is available as [`fortitude-bin`](https://aur.archlinux.org/packages/fortitude-bin)
and can be download using `yay/paru` as

```bash
$ yay -S fortitude-bin
```
