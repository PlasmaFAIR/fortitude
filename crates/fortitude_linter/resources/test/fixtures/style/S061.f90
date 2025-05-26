module mymod1
  implicit none
  type mytype
    integer :: x
    ! catch this
  end type
contains
  subroutine mysub1()
    write (*,*) 'hello world'
    ! catch this
  end subroutine
  subroutine mysub2()
    write (*,*) 'hello world'
    ! ignore this
  end subroutine mysub2
  ! catch this
end
module mymod2
  implicit none
  type mytype
    integer :: x
    ! ignore this
  end type mytype
contains
  integer function myfunc1()
    myfunc1 = 1
    ! catch this
  end function
  integer function myfunc2()
    myfunc2 = 1
    ! ignore this
  end function myfunc2
  ! catch this
end module
module mymod3
  interface
    module function foo() result(x)
      integer :: x
      ! ignore this
    end function foo
    module function bar() result(x)
      integer :: x
      ! ignore this
    end function bar
    module function baz() result(x)
      integer :: x
      ! ignore this
    end function baz
  end interface
end module mymod3
submodule (mymod3) mysub1
contains
  module procedure foo
    x = 1
    ! catch this
  end procedure
  ! catch this
end
submodule (mymod3) mysub2
contains
  module procedure bar
    x = 1
    ! ignore this
  end procedure bar
  ! catch this
end submodule
submodule (mymod3) mysub3
contains
  module procedure baz
    x = 1
    ! ignore this
  end procedure baz
  ! ignore this
end submodule mysub3
program myprog
  implicit none
  write (*,*) 'hello world'
  ! catch this
end

! uppercase versions, check preserve case
MODULE MYMOD1_UPPER
  IMPLICIT NONE
  TYPE MYTYPE
    INTEGER :: X
    ! CATCH THIS
  END TYPE
CONTAINS
  SUBROUTINE MYSUB1()
    WRITE (*,*) 'HELLO WORLD'
    ! CATCH THIS
  END SUBROUTINE
  SUBROUTINE MYSUB2()
    WRITE (*,*) 'HELLO WORLD'
    ! IGNORE THIS
  END SUBROUTINE MYSUB2
  ! CATCH THIS
END
MODULE MYMOD2_UPPER
  IMPLICIT NONE
  TYPE MYTYPE
    INTEGER :: X
    ! IGNORE THIS
  END TYPE MYTYPE
CONTAINS
  INTEGER FUNCTION MYFUNC1()
    MYFUNC1 = 1
    ! CATCH THIS
  END FUNCTION
  INTEGER FUNCTION MYFUNC2()
    MYFUNC2 = 1
    ! IGNORE THIS
  END FUNCTION MYFUNC2
  ! CATCH THIS
END MODULE
