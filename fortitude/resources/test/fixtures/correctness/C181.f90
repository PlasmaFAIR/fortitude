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
  ALLOCATE (x(10), stat=status)

  ! Set same stat and check it.
  ! This is fine.
  allocate (y(10), stat=Status)
  print *, "Something between allocation and checking"
  if (status /= 0) then
    print *, "Error allocating y"
    stop 1
  end if

  deallocate (x)
  Deallocate (y)

  ! Repeat, but the second allocate is not a direct sibling
  ALLOCATE (x(10), Stat=status)
  if (.true.) then
    ALLOCATE (y(10), STAT=statUs)
    if (status /= 0) then
      print *, "Error allocating y"
      stop 1
    end if
  end if

  DEALLOCATE (x)
  deallocate (y)

  ! Repeat, but they use different stat parameters. Should be okay.
  allocate (x(10), STAT=status_x)
  if (.true.) then
    allocate (y(10), stat=status_y)
    if (STATUS_Y /= 0) then
      print *, "Error allocating y"
      stop 1
    end if
  end if
  if (Status_x /= 0) then
    print *, "Error allocating x"
    stop 1
  end if

  deallocate (X)
  Deallocate (Y)

  ! Repeat, but using a custom error handler
  allocate (x(10), stat=STATUS_X)
  if (.true.) then
    allocate (y(10), STAT=status_y)
    call handle_error(stat=STATUS_Y)
  end if

  deallocate (x)
  deallocate (y)

  ! Stat is checked in an aunt/uncle node, if/else branch.
  ! This is fine.
  if_option = 5
  if (if_option == 1) then
    allocate (x(10), stat=STATUS)
  else if (if_option == 5) then
    allocate (x(20), stat=status)
  else
    allocate (x(30), STAT=status)
  end if
  if (Status /= 0) then
    print *, "Error allocating x"
    stop 1
  end if
  deallocate (x)

  ! Stat is checked in an aunt/uncle node, select case
  ! This is fine.
  select_option = 5
  select case (select_option)
  case(1)
    allocate (x(10), stat=STATUS)
  case(5)
    allocate (x(20), STAT=status)
  case default
    allocate (x(30), stat=Status)
  end select
  if (status /= 0) then
    print *, "Error allocating x"
    stop 1
  end if
  deallocate (x)

  ! Stat is not checked in any cases. Should raise for all four.
  if(if_option == 1) then
    allocate (x(10), stat=STATUS_X)
  else
    allocate (x(30), stat=status_x)
  end if
  select case (select_option)
  case(1)
    allocate (y(10), stat=Status_y)
  case default
    allocate (y(20), stat=status_y)
  end select

  ! Set stat, but overwrite it before checking it.
  ! Should raise an error.
  allocate (w(10), STAT=status_w)
  STATUS_W = 0
  if (Status_w /= 0) then
    print *, "Error allocating w"
    stop 1
  end if
  deallocate (w)

  ! Set stat, but don't check it until the end of the program.
  ! Should raise an error.
  allocate (z(10), stat=status_z)

  print *, "Something between allocate and the end of the program"
  ALLOCATE (w(20))

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
    open (file="file.txt", unit=10, IOSTAT=Status, asynchronous="YES")
    write(10, *, iostat=status) line
    flush(10, iostat=STATUS)
    backspace(10, iostat=Status)
    rewind(10, IOSTAT=status)
    read(10, *, iostat=status, asynchronous="YES") line
    wait(10, Iostat=STATUS)
    endfile(10, iostat=status)
    close(10, iostat=STATUS)
    ! Something other than IO statement, reuse status
    allocate(x(10), STAT=Status)
    deallocate(x, stat=status)
  end subroutine io

  integer function cmd_line() result(status)
    call execute_command_line("ls", cmdstat=Status)
    call execute_command_line("ls", Cmdstat=status)
  end function cmd_line

end program p
