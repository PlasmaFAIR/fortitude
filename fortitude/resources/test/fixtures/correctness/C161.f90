module test
  implicit none (external, type)
contains
  logical function bad(arg1, arg2)
    integer, optional, intent(in) :: arg1, arg2(2)
    bad = .false.
    if (.not. present(arg1)) then
      bad = .false.
    end if

    if (present(arg1) .and. arg1 > 1) then
      bad = .true.
    end if

    if (present(arg1) .or. present(arg2)) then
      bad = .false.
    end if

    if (present(arg2) .and. (size(arg2) > 4)) then
      bad = .true.
    end if

    if ((present(arg1) .and. .not. present(arg2)) .or. (.not. present(arg1) .and. present(arg2))) then
      bad = .false.
    end if

    if ((.not. present(arg2)) .or. (present(arg1) .and. present(arg2))) then
      bad = .false.
    end if

  end function bad
end module test
