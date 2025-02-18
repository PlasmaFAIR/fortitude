# missing-exit-or-cycle-label (S021)
Fix is sometimes available.

This rule is turned on by default.

## What does it do?
When using `exit` or `cycle` in a named `do` loop, the `exit`/`cycle` statement
should use the loop name

## Example
```f90
name: do
  exit name
end do name
```

Using named loops is particularly useful for nested or complicated loops, as it
helps the reader keep track of the flow of logic. It's also the only way to `exit`
or `cycle` outer loops from within inner ones.