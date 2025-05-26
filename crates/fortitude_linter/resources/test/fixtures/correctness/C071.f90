subroutine assumed_size_dimension(array, n, m, l, o, p, options, thing, q)
  integer, intent(in) :: n, m
  integer, dimension(n, m, *), intent(in) :: array
  integer, intent(in) :: l(*), o, p(*)
  ! warning must be on the array part for characters
  character(len=*), dimension(*) :: options
  character(*) :: thing(*)
  ! this is ok
  character(*), intent(in) :: q
  ! following are ok because they're parameters
  integer, dimension(*), parameter :: param = [1, 2, 3]
  character(*), dimension(*), parameter :: param_char = ['hello']
end subroutine assumed_size_dimension
