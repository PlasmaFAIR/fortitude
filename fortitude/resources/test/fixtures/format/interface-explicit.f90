module unit_test_module
  private

  use iso_c_binding

  interface assert_equals
    module procedure assert_equals_int
    module procedure assert_equals_real8
  end interface

  real(8), save :: eps = epsilon(1.0d0)/epsilon(1.0)
  logical,save :: all_pass, passing
  character(5) :: nc = achar(27)//'[00m' ! reset color
  character(7) :: gr = achar(27)//'[0;32m' ! green
  character(7) :: rd = achar(27)//'[1;31m' ! red

  public :: assert_equals
  public :: all_pass

  contains

  function assert_equals_real8(val1, val2) result(passed)
    real(8) :: val1, val2
  end function

  function assert_equals_logi(val1, val2) result(passed)
    logical :: val1, val2
  end function
end module
