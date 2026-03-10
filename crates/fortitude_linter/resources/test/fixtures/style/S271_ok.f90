module test_module
                                use, intrinsic :: iso_fortran_env, ONLY: int32
    USE module_a, only: fun_a !! some_comments
					use module_b_with_tab, only: fun_b
  use module_c, only: fun_c3, &
                      fun_c2, & !! some_comments
                      fun_c1
    use module_no_only
    USE module_z_last, only: fun_z

  use module_single, only: fun_single

        use, intrinsic :: iso_c_binding, only: fun_i
use, intrinsic :: iso_fortran_env, only: real64
        use aa_non_intrinsic_module_d, only: fun_d

  implicit none (type, external)

  private
  contains

  real function compute_something(x, y)
    use, intrinsic :: ieee_arithmetic, only: fun_a
    use, intrinsic :: iso_fortran_env, only: real64, int32
    use another_package, only: helper, util
    use custom_math, only: fun_m
    real(real64), intent(in) :: x, y

    compute_something = helper(x) + util(y) + real(ieee_max(x,y), real64)
  end function compute_something

  subroutine test_comments()
    ! fun_a is used for...
    use module_a, only: fun_a
    ! fun_b is used for...
    use module_b, only: fun_b
    !! fun_c is used for...
    !! and also for..
    use module_c, only: fun_c
  end subroutine test_comments
end module test_module
