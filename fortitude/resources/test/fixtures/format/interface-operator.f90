interface operator (+)
  module procedure char_plus_int
  module procedure int_plus_char
end interface operator (+)

interface operator (.not.)
  module procedure int_not
  function real_not(a)
    real :: real_not
    real,intent(in) :: a
  end function
end interface operator (.not.)
