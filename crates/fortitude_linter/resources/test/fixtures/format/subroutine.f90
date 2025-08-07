pure recursive subroutine static_method(arg)
   integer, intent(in) :: arg
   REAL(8), SAVE :: counter
   counter = counter + arg
end subroutine static_method

subroutine parens_but_no_args()
end subroutine

subroutine exported_fun(arg1, arg2) bind(c)
end subroutine

subroutine exported_fun2() bind(c, name="f90_func")
end subroutine

SUBROUTINE WITH_INTERNAL_PROC
  integer :: cmd_stat

  CALL CHECK_RETURN_VAL("ls *", cmd_stat)

  CONTAINS
    SUBROUTINE CHECK_RETURN_VAL(cmd, title, istat)
      integer, intent(out) :: istat
      character(*), intent(in) :: cmd
      character*(*), intent(in) :: title
      istat = SYSTEM(cmd)
      RETURN
    END SUBROUTINE
END SUBROUTINE
