"""Run the compiled executable.

Adapted from the ``ruff`` launcher script.
"""

import subprocess
import sys
import sysconfig
from pathlib import Path


def find_exe() -> Path:
    """Return the compiled ``fortitude`` executable path."""

    exe_name = "fortitude" + sysconfig.get_config_var("EXE")

    scripts_path = Path(sysconfig.get_path("scripts"))
    if (exe := scripts_path / exe_name).is_file():
        return exe

    user_scheme = sysconfig.get_preferred_scheme("user")
    user_path = Path(sysconfig.get_path("scripts", scheme=user_scheme))
    if (exe := user_path / exe_name).is_file():
        return exe

    msg = "Could not locate compiled fortitude executable"
    raise RuntimeError(msg)


if __name__ == "__main__":
    completed_process = subprocess.run([str(find_exe()), *sys.argv[1:]])
    sys.exit(completed_process.returncode)
