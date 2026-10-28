program test
  etime: block
    real, dimension(2) :: tarray
    real :: result
    call ETIME(tarray, result)
    print *, result
    print *, tarray(1)
    print *, tarray(2)
  end block etime

  idate: block
    integer, dimension(3) :: tarray
    call idate(tarray)
    print *, tarray(1)
    print *, tarray(2)
    print *, tarray(3)
  end block idate

  fdate: block
    character(len=30) :: date
    call fdate(date)
    print *, 'Program started on ', date
  end block fdate
end program test
