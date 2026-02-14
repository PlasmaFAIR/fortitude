! This is a long comment that starts at the begining of the line and exceeds the line length setting is ignored
program test
  use some_really_really_really_really_really_really_really_really_really_really_really_long_module_name, only : integer_working_precision ! This code is not ignored
  implicit none (type, external)                                                                  !This comment is ignored
  integer(integer_working_precision), parameter, dimension(1) :: a = [1]                               ! This comment is ignored
  character(len=101) :: string_literal_with_exclamation = '! some_really_really_really_really_really_really_really_really_really_really_really_long string literal'
  end program test
