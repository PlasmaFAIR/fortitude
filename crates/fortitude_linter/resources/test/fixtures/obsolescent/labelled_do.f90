program test
    integer :: i, j, k, l, m, foo(10), A(10, 10), B(10, 10, 10)

    ! No gotos, so we can autofix
    do 10 i = 1, 10
        foo(i) = i
10  continue

    ! Has a goto which means we can't remove label on `end do`
    do 20 i = 1, 10
        if (i > 5) goto 20
        foo(i) = 2 * i
20  end do

    ! Has a goto which means we can't remove label on `end do`
    ! We still have to replace `continue` with `end do`
    do 25 i = 1, 10
        if (i > 5) goto 25
        foo(i) = 2 * i
25  continue

    ! Doesn't end in `continue` or `end do`
    do 30 i = 1,10
30  foo(i) = 3 * i

    ! Doesn't end in `continue` or `end do`
    ! and has arithmetic `if` so we need to keep label
    do 40 i = 1,10
    if (i - 5) 39, 39, 40
39  continue
40  foo(i) = 3 * i

    ! Shared termination
    do 100 i = 1,10
      do 100 j = 1,10
        do 200 k = 1,10
          do 200 l = 1,10
            do 200 m = 1,10
            A(i,j) = A(i,j) + B(k,l,m)
200 Continue
100 End Do

    double_loop: do 400, i=1,10
      inner: do 500, j=1,10
        A(i, j) = i + j
500   enddo inner
400 enddo double_loop

    ! This is obviously fine
    do i = 1, 10
        foo(i) = i
    end do

    ! This only has a `goto` that could be a `cycle`
    do i = 1, 10
        if (i > 5) goto 91
        foo(i) = 2 * i
91  end do

    do i = 1,10
      do j = 1,10
        do k = 1,10
          do l = 1,10
            do m = 1,10
              A(i,j) = A(i,j) + B(k,l,m)
            end do
          end do
        end do
      end do
    end do

end program test
