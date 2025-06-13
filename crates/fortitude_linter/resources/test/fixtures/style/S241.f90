program p
  implicit none (type, external)

  print *, "Hello, World!"
  print *, 'Hello, World!'
  print *, 'Hello, "World"!'
  print *, "Hello, ""World""!"
  print *, 'Hello, ''World''!'
  print *, "Hello, &
            & World!"
  print *, 'Hello, &
            & World!'
  print *, 'Hello, &
            & "World"!'
  print *, "Hello, &
            & ""World""!"
  print *, 'Hello, &
            & ''World''!'

  ! TODO: Add tests for multiline strings with a comment line in the middle

end program p
