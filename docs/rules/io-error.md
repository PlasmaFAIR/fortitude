# io-error (E000)
## What it does
This is not a regular diagnostic; instead, it's raised when a file cannot be read
from disk.

## Why is this bad?
An `IoError` indicates an error in the development setup. For example, the user may
not have permissions to read a given file, or the filesystem may contain a broken
symlink.

## Example
On Linux or macOS:
```shell
$ echo -e 'print*, "hello world!"\nend' > a.f90
$ chmod 000 a.f90
$ fortitude check a.f90
a.f90:1:1: E902 Permission denied (os error 13)
Found 1 error.
```

## References
- [UNIX Permissions introduction](https://mason.gmu.edu/~montecin/UNIXpermiss.htm)
- [Command Line Basics: Symbolic Links](https://www.digitalocean.com/community/tutorials/workflow-symbolic-links)