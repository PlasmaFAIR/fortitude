module test
  implicit none (external, type)
  integer, target :: global
contains
  logical function bad(arg1, arg2)
    integer, optional, pointer, intent(in) :: arg1
    integer, optional, allocatable, dimension(:), intent(in) :: arg2
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

    if (present(arg1) .and. associated(arg1)) then
      bad = .true.
    end if

    ! Two-argument version is ok because it doesn't guard "definedness".
    if (associated(arg1, global) .or. arg1 > 1 ) then
      bad = .false.
    end if

    if (associated(arg1) .and. arg1 > 1) then
      bad = .true.
    end if

    if ((.not. allocated(arg2)) .or. (.not. present(arg2))) then
      bad = .true.
    end if

    if (allocated(arg2) .and. size(arg2) > 1) then
      bad = .true.
    end if

  end function bad
end module test
