program test
  type simple
  end type
  type, public :: custom_type
    public
    sequence
    private
    real(8) :: x,y,z
    integer :: w,h,l
    real(eb), allocatable, dimension(:, :, :) :: vals
    procedure(), pointer, nopass :: do_nothing => null()
    contains
       procedure, nopass, non_overridable :: static_method ! static method
       procedure instance_method ! instance method
       procedure, public, pass(self) :: pass_method
       generic, pass :: binding_name => method_name, method_name2
       generic, private :: assignment(=) => assign_method
       generic, private :: operator(+) => add_method
       procedure :: one, two => three
       final :: finalize
  end type custom_type

  type, abstract :: matrix(k, d)
    integer, kind :: k = kind(0.0)
    integer (selected_int_kind(12)), len :: d
    real(k) :: element(d, d)
  end type
    
end program
