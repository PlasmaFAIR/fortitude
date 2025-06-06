program myprog
  implicit none
  integer :: a = 1
  integer, allocatable :: b(:)
  if(a ==1)then  ! A space should be added after if and before then
    a = 2
  else if  (a == 2)then ! A space should be removed after if and added before then
    a = 3
  else if &
    (a == 3) then ! This should be unchanged
    a = 4
  else if(a == 4)  then ! A space should be added after if and removed before then
    a = 5
  end if
  allocate (b(3))  ! This should be unchanged
  read(*,*) b  ! A space should be added after write
  write(*,*) "b = ", b  ! This should be unchanged
  read (*,*) b  ! This should be unchanged
  write (*,*) "b = ", b  ! This should be unchanged
  read  (*,*) b  ! This should be unchanged
  write  (*,*) "b = ", b  ! This should be unchanged
  deallocate (b)  ! This should be unchanged
  allocate(b(4))  ! A space should be added after allocate
  READ(*,*) b  ! A space should be added after write
  WRITE(*,*) "b = ", b  ! A space should be added after write
  READ (*,*) b  ! This should be unchanged
  WRITE (*,*) "b = ", b  ! This should be unchanged
  READ  (*,*) b  ! A space should be removed after READ
  WRITE  (*,*) "b = ", b  ! A space should be removed after READ
  deallocate(b)  ! A space should be added after deallocate
  allocate  (b(4))  ! A space should be removed after allocate
  deallocate  (b)  ! A space should be removed after deallocate
  ALLOCATE(b(4))  ! A space should be added after allocate
  DEALLOCATE(b)  ! A space should be added after deallocate
  ALLOCATE (b(3))  ! This should be unchanged
  DEALLOCATE (b)  ! This should be unchanged
  ALLOCATE  (b(4))  ! A space should be removed after ALLOCATE
  DEALLOCATE  (b)  ! A space should be removed after DEALLOCATE
end program myprog
