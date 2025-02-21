real function my_func(a, b, c, d, e)       ! catch
  real, intent(in) :: a                    ! catch
  real(4), intent(in) :: b                 ! ignore
  integer, intent(in) :: c                 ! ignore
  complex, intent(in) :: d                 ! catch
  complex(8), intent(in) :: e              ! ignore

  myfunc = a
end function my_func
