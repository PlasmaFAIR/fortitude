program p
  implicit none
  real(8), allocatable :: w(:), x(:), y(:), z(:)
  integer :: status, status_w, status_x, status_y, status_z
  integer :: if_option, select_option

  ! Don't check the status of the allocation.
  ! This is fine.
  allocate (w(10))
  deallocate (w)

  ! Set stat, but don't check it until after another allocate.
  ! Should raise an error.
  allocate (x(10), stat=status)

  ! Set same stat and check it.
  ! This is fine.
  allocate (y(10), stat=status)
  print *, "Something between allocation and checking"
  if (status /= 0) then
    print *, "Error allocating y"
    stop 1
  end if

  deallocate (x)
  deallocate (y)

  ! Repeat, but the second allocate is not a direct sibling
  allocate (x(10), stat=status)
  if (.true.) then
    allocate (y(10), stat=status)
    if (status /= 0) then
      print *, "Error allocating y"
      stop 1
    end if
  end if

  deallocate (x)
  deallocate (y)

  ! Repeat, but they use different stat parameters. Should be okay.
  allocate (x(10), stat=status_x)
  if (.true.) then
    allocate (y(10), stat=status_y)
    if (status_y /= 0) then
      print *, "Error allocating y"
      stop 1
    end if
  end if
  if (status_x /= 0) then
    print *, "Error allocating x"
    stop 1
  end if

  deallocate (x)
  deallocate (y)

  ! Repeat, but using a custom error handler
  allocate (x(10), stat=status_x)
  if (.true.) then
    allocate (y(10), stat=status_y)
    call handle_error(stat=status_y)
  end if

  deallocate (x)
  deallocate (y)

  ! Stat is checked in an aunt/uncle node, if/else branch.
  ! This is fine.
  if_option = 5
  if (if_option == 1) then
    allocate (x(10), stat=status)
  else if (if_option == 5) then
    allocate (x(20), stat=status)
  else
    allocate (x(30), stat=status)
  end if
  if (status /= 0) then
    print *, "Error allocating x"
    stop 1
  end if
  deallocate (x)

  ! Stat is checked in an aunt/uncle node, select case
  ! This is fine.
  select_option = 5
  select case (select_option)
  case(1)
    allocate (x(10), stat=status)
  case(5)
    allocate (x(20), stat=status)
  case default
    allocate (x(30), stat=status)
  end select
  if (status /= 0) then
    print *, "Error allocating x"
    stop 1
  end if
  deallocate (x)

  ! Stat is not checked in any cases. Should raise for all four.
  if(if_option == 1) then
    allocate (x(10), stat=status_x)
  else
    allocate (x(30), stat=status_x)
  end if
  select case (select_option)
  case(1)
    allocate (y(10), stat=status_y)
  case default
    allocate (y(20), stat=status_y)
  end select

  ! Set stat, but overwrite it before checking it.
  ! Should raise an error.
  allocate (w(10), stat=status_w)
  status_w = 0
  if (status_w /= 0) then
    print *, "Error allocating w"
    stop 1
  end if
  deallocate (w)

  ! Set stat, but don't check it until the end of the program.
  ! Should raise an error.
  allocate (z(10), stat=status_z)

  print *, "Something between allocate and the end of the program"
  allocate (w(20))

contains

  subroutine handle_error(stat)
    integer, intent(in) :: stat
    if (stat /= 0) then
      print *, "allocation error"
      stop 1
    end if
  end subroutine handle_error

  subroutine io()
    integer :: status
    character(len=100) :: line
    real(8), allocatable :: x(:)
    logical :: file_exist
    inquire(unit=10, iostat=status, exist=file_exist)
    open (file="file.txt", unit=10, iostat=status, asynchronous="YES")
    write(10, *, iostat=status) line
    flush(10, iostat=status)
    backspace(10, iostat=status)
    rewind(10, iostat=status)
    read(10, *, iostat=status, asynchronous="YES") line
    wait(10, iostat=status)
    endfile(10, iostat=status)
    close(10, iostat=status)
    ! Something other than IO statement, reuse status
    allocate(x(10), stat=status)
    deallocate(x, stat=status)
  end subroutine io

  integer function cmd_line() result(status)
    call execute_command_line("ls", cmdstat=status)
  end function cmd_line

end program p
