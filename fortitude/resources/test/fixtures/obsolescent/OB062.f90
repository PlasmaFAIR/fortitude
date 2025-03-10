program p

  implicit none (type, external)

  character*5 :: hello
  character(len=5) :: world
  character*   5 foo
  CHARACTER   *5 :: bar
  charaCTeR   *  5 :: baz
  character*10 x, y, z

  hello = "hello"
  world = "world"

  write (*, *) hello, world

end program p
