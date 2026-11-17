program p
  use zeta, only: z, a, m => q, b
  use alpha, only: a, b, c
  use aliases, only: b => z, a => y
  use multiline, only: c, &
      a, b
  use commented, only: b, & ! comment
      a
end program p
