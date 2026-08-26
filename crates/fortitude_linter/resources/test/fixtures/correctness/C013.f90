program test
  implicit none (type, external)

  ! This should be flagged by C013
  select rank(A)
  rank (0)
    ! Scalar
    call scalarVersion(A)
  rank (1)
    call vectorVersion(A)
  end select

  ! This should pass C013
  select rank(A)
  rank (0)
    ! Scalar
    call scalarVersion(A)
  rank (1)
    call vectorVersion(A)
  rank default
    call handle_error("Unsupported rank: ", rank(A))
  end select
end program test
