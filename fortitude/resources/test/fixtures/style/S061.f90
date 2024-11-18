module mymod1
  implicit none
  type mytype
    integer :: x
  end type                      ! catch this
contains
  subroutine mysub1()
    write (*,*) 'hello world'
  end subroutine                ! catch this
  subroutine mysub2()
    write (*,*) 'hello world'
  end subroutine mysub2         ! ignore this
end                             ! catch this
module mymod2
  implicit none
  type mytype
    integer :: x
  end type mytype               ! ignore this
contains
  integer function myfunc1()
    myfunc1 = 1
  end function                  ! catch this
  integer function myfunc2()
    myfunc2 = 1
  end function myfunc2          ! ignore this
end module                      ! catch this
module mymod3
  interface
    module function foo() result(x)
      integer :: x
    end function foo            ! ignore this
    module function bar() result(x)
      integer :: x
    end function bar            ! ignore this
    module function baz() result(x)
      integer :: x
    end function baz            ! ignore this
  end interface
end module mymod3
submodule (mymod3) mysub1
contains
  module procedure foo
    x = 1
  end procedure                 ! catch this
end                             ! catch this
submodule (mymod3) mysub2
contains
  module procedure bar
    x = 1
  end procedure bar             ! ignore this
end submodule                   ! catch this
submodule (mymod3) mysub3
contains
  module procedure baz
    x = 1
  end procedure baz             ! ignore this
end submodule mysub3            ! ignore this
program myprog
  implicit none
  write (*,*) 'hello world'
end                             ! catch this
