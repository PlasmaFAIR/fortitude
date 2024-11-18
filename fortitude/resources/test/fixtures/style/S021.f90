program test
  label1: do
    if (.true.) then
      EXIT
    end if
  end do label1

  label2: do
    if (.true.) exit
  end do label2

  label3: do
    exit label3
  end do label3

  label4: do
    if (.true.) exit label4
  end do label4

  label5: do i = 1, 2
    do j = 1, 2  ! unnamed inner loop: currently doesn't warn
      cycle
    end do
  end do label5

  label6: do i = 1, 2
    inner: do j = 1, 2
      if (.true.) CYCLE ! named inner loop: warns on inner loop
    end do inner
  end do label6

  label7: do
    cycle label7
  end do label7

  label8: do
    if (.true.) cycle label8
  end do label8

  do
    ! Don't warn on unnamed loops
    exit
  end do
end program test
