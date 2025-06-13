program test
  implicit none (type, external)
  print*, "this looks like implicit "&
       &"concatenation but isn't"
  print*, "this looks like implicit "&  
       "concatenation but isn't"
  print*, 'this looks like implicit '&
       &'concatenation but isn''t'
  print*, 'this looks like implicit '&
       'concatenation but isn''t'
  print*, "this explicit concatenation "&
       // "is intended"
  print*, 'this explicit concatenation '&
       // 'is intended'
  print*, "this is a normal &
       &multiline string"
end program test
