program test
  implicit none (type, external)

  print*, 'This isn''t necessary'
  print*, "This ""is not"" necessary"
  print*, 'This "isn''t" unnecessary'
  print*, "This ""isn't"" unnecessary"
  print*, 4_"Does this ""break""?"
end program test
