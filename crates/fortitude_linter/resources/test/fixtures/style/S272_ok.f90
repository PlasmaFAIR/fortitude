program p
  use alpha, only: a, b, c
  use aliases, only: a => z, b => y
  use multiline, only: a, &
      b, c
  use commented, only: a, & ! comment
      b
end program p
