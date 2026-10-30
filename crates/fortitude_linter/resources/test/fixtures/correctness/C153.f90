program test
  implicit none
  integer :: nkeys, x
  logical :: flag

  ! Violations
  if (nkeys > 0) then
    call next_record()
    if (flag) then
      goto 100
    else
    end if
  end if

  if (flag) then
  else
    x = 1
  end if

  if (flag) then
    x = 1
  else if (nkeys > 1) then
  else
    x = 2
  end if

  if (flag) then
    ! x = 1
  end if

  if (flag) then
    x = 1
  else
    ! x = 2
  end if

  ! No violations
  if (flag) then
    x = 1
  else
    continue
  end if

  if (flag) then
    x = 1
  else if (nkeys > 1) then
    x = 2
  else
    x = 3
  end if

  if (flag) x = 1

100 continue
end program test
