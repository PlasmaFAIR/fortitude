! Everything in caps is actually an identifier of some kind, so should be left
! alone, and we should raise no diagnostics
program d
  use MODULE, only: DATA => WHERE
  implicit none(type, external)

  common /WHERE/ DATA

  type WHERE
    real :: DATA
  end type WHERE

  type(WHERE) :: a
  procedure(DATA) :: nonsense

  a%DATA = 1

  DATA: if (DATA) then
  end if DATA

  WHERE: do DATA = 1, 10
  end do WHERE

end program d
