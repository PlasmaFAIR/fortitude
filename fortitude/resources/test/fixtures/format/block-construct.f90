program block_demo
  implicit none
  real :: y = 1
  print*, y

  block
    real :: x = 3.142
    print*, x
    y = x
    inner: block
      real :: y = 12.1
      print*, y
    end block inner
  end block

  print*, y
end program block_demo
