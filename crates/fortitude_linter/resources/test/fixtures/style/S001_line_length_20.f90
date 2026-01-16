! SPDX-License-Identifier: too-long-but-dont-flag
! This comment is too long
! https://dont-flag-urls
program test
  use some_really_long_module_name, only : integer_working_precision
  use &
    some_other_really_long_module_but_only_word_on_line_so_dont_flag
  implicit none
  integer(integer_working_precision), parameter, dimension(1) :: a = [1]
end program test
