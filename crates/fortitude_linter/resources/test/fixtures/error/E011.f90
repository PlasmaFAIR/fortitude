program invalid_tab
  implicit none
  logical :: français = .true.
  print*, "Don't flag this: é"
  ! Or this:|é|
end program invalid_tab
