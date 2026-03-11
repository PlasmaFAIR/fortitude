module test_module
	use module_z_with_tab, only: fun_z_with_tab
  use module_c, only: fun_c3, &
                      fun_c2, & !! fun_c2 comments
                      fun_c1
  use, intrinsic :: iso_fortran_env, ONLY: int32
    USE module_a, only: fun_a !! module_a inline comments
  USE module_z, only: fun_z
  use module_no_only

  use module_single, only: fun_single

  use, intrinsic :: iso_fortran_env, only: real64
  use aa_non_intrinsic_module_d, only: fun_d
  use, intrinsic :: iso_c_binding, only: fun_i

  implicit none (type, external)

  private
  contains
  real function compute_something(x, y)
    use custom_math, only: fun_m
    use another_package, only: helper, util
    use, intrinsic :: iso_fortran_env, only: real64, int32
    use, intrinsic :: ieee_arithmetic, only: fun_a
    real(real64), intent(in) :: x, y

    compute_something = helper(x) + util(y) + real(ieee_max(x,y), real64)
  end function compute_something

  subroutine test_comments_as_separator()
    !! fun_c is used for...
    use module_c, only: fun_c
    use module_a, only: fun_a
    ! fun_b is used for...
    use module_b, only: fun_b
  end subroutine test_comments_as_separator
end module test_module
