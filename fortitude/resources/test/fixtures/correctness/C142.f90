program test
  label1: do
    if (.true.) then
      ! shouldn't warn
      EXIT
    end if
  end do label1

  label2: do
    ! shouldn't warn
    if (.true.) exit
  end do label2

  label3: do
    ! shouldn't warn
    exit label3
  end do label3

  label4: do
    ! shouldn't warn
    if (.true.) exit label4
  end do label4

  label5: do i = 1, 2
    ! should warn
    do j = 1, 2
      cycle
    end do
  end do label5

  label6: do i = 1, 2
    inner: do j = 1, 2
      ! shouldn't warn
      if (.true.) CYCLE
    end do inner
  end do label6

  label7: do
    ! shouldn't warn
    cycle label7
  end do label7

  label8: do
    ! shouldn't warn
    if (.true.) cycle label8
  end do label8

  do
    ! should warn, except if `nested-loops-only` is true
    exit
  end do

  do
    do
      ! should warn
      exit
    end do
  end do

  do
    do
      label9: do
        ! shouldn't warn in either case, this is missing-exit-or-cycle-label!
        exit
      end do label9
    end do
  end do

end program test
