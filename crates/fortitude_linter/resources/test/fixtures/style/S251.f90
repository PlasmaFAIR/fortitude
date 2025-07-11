program test
  implicit none (type, external)
  print*, 1. + 2. * 3. - 4. / 5.;
  print*, 1.*(cos(2.) - 3. + (4.*5.-6.*sin(7.))*sin(8.))
  print*, 1. * 2. / 3. * 4.;
  print*, -1.**2

  ! These are all fine
  print*, 1. + (2. * 3.) - (4. / 5.);
  print*, (1. * 2.) / (3. * 4.);
  print*, (-1.)**2

end program test
