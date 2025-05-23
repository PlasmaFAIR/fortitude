program test
  use, intrinsic :: iso_c_binding, only: c_char
  implicit none (type, external)

  print*, 'This isn''t necessary'
  print*, "This ""is not"" necessary"
  print*, 'This "isn''t" unnecessary'
  print*, "This ""isn't"" unnecessary"
  print*, 4_"Does this ""break""?"
  print*, c_char_"Does this ''break''?"
  print*, 'This &
       &isn''t &
       &necessary'
  print*, 'This &
       &""isn''t"" &
       &unnecessary'

  ! Multiple quotes
  print*, ''                    ! empty
  print*, ""                    ! empty
  print*, ''''                  ! '
  print*, """"                  ! "
  print*, ''''''                ! ''
  print*, ''''''''              ! '''
  print*, """"""                ! ""
  print*, """"""""              ! """

end program test
