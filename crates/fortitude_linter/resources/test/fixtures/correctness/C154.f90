program test
  implicit none
  logical :: ready, done

  ready = .false.
  done = .true.

  ! Left operand is a constant.
  if (.true. .and. ready) print *, "left and"
  if (.false. .or. ready) print *, "left or"

  ! Right operand is a constant.
  if (ready .and. .false.) print *, "right and"
  if (ready .or. .true.) print *, "right or"

  ! Nested, so the inner expression is reported too.
  if (ready .and. (.false. .or. done)) print *, "nested"

  ! Both sides constant: two diagnostics.
  if (.true. .or. .false.) print *, "both"

  ! The spelling of the keyword and of the literal does not matter, and a
  ! keyword split by a line continuation is still the same operator.
  if (.FALSE. .AND. ready) print *, "upper"
  if (ready .&
       &or. .true.) print *, "spliced"

  ! A kind suffix does not hide the literal, and parentheses do not either.
  if (ready .or. .true._4) print *, "kinded"
  if (ready .and. (.false.)) print *, "parenthesised"

  ! Not reported: no constant operand, or the constant is not an operand of
  ! `.and.`/`.or.`.
  if (ready .and. done) print *, "clean"
  if (ready .eqv. .true.) print *, "eqv"
  if (.not. ready) print *, "not"
end program test
