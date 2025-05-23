program p
  implicit none
  real(8), allocatable :: x(:)
  integer :: status
  character(256) :: message
  logical :: file_exist

  ! no stat params
  allocate (x(10))
  deallocate (x)
  open (10, file="test.txt")
  write (10,*) "Allocation and deallocation without stat parameters completed successfully."
  inquire(unit=10, exist=file_exist)
  wait (10)
  flush(10)
  close (10)
  call execute_command_line("ls")

  ! stat params, no message
  allocate (x(10), Stat=status)
  deallocate (x, &
    stat=STATUS)
  open (10, file="test.txt", iostat=status)
  write (10,*,iosTAT=Status) "Allocation and deallocation without stat parameters completed successfully."
  inquire(unit=10, &
    iostat=status, exist=file_exist)
  wait (10, iostat=status)
  flush(10, IOSTAT=&
    STATUS)
  close (10, Iostat=Status)
  call execute_command_line("ls", cmdstat=Status)

  ! stat params, with message
  allocate (x(10), Stat=status, errmsg=message)
  deallocate (x, stat=STATUS, ERRMSG=Message)
  open (10, file="test.txt", iostat=status, iomsg=message)
  write (10,*, &
    iosTAT=Status, &
    IOMSG=&
      MESSAGE) "Allocation and deallocation without stat parameters completed successfully."
  inquire(unit=10, iostat=status, exist=file_exist, ioMsG=MesSaGe)
  wait (10, iostat=status, iomsg=message)
  flush(10, IOSTAT=STATUS, iomsg = Message)
  close (10, Iostat=Status, ioMSG = &
    Message)
  call execute_command_line("ls", cmdstat=Status, &
    cmdmsg=message)

end program p
