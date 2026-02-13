PROGRAM test
    implicit none
    integer :: x(1), k, i

    ! Permutations on three blocks
    if (k) 1, 2, 3
1   continue
    print*, "<"
    i = 0
    goto 4
2   continue
    print*, "=="
    i = 1
    goto 4
3   continue
    print*, ">"
    i = 2
4   continue

    if (k) 13, 12, 11
11  continue
    print*, ">"
    i = 0
    goto 14
12  continue
    print*, "=="
    i = 1
    goto 14
13  continue
    print*, "<"
    i = 2
14  continue

    if (k) 23, 21, 22
21  continue
    print*, "=="
    i = 0
    goto 24
22  continue
    print*, ">"
    i = 1
    goto 24
23  continue
    print*, "<"
    i = 2
24  continue

    ! Permutations on two blocks
    if (k) 31, 31, 32
31  continue
    print*, "<="
    i = 0
    goto 34
32  continue
    print*, ">"
    i = 2
34  continue

    if (k) 41, 42, 42
41  continue
    print*, "<"
    i = 0
    goto 44
42  continue
    print*, ">="
    i = 2
44  continue

    if (k) 51, 52, 54
51  continue
    print*, "<"
    i = 0
    goto 54
52  continue
    print*, "=="
    i = 2
54  continue

    ! Single block
    if (k) 62, 61, 62
61  print*, "=="
62  continue

    if (k) 71, 72, 71
71  continue
    print*, "/="
72  continue

85  do 110 k=1,9
87     if(i-1) 100,90,100
90     print*, "=="
       goto 110
100    continue
110 continue

   ! This should become:
    do 210 k=1,9
       if (i-1 == 0) then
         print*, "=="
         ! retain goto because this should reall be `cycle`
         goto 210
       end if
210 continue


contains
  ! These should be fixable, but we have to not consume the returns
  subroutine blocks_end_in_control_flows(M)
    integer, intent(in) :: M
    if (k) 1, 2, 3
1   continue
    print*, "<"
    i = 0
    return
2   continue
    print*, "=="
    i = 1
    return
3   continue
    print*, ">"
    i = 2
    continue
  end subroutine blocks_end_in_control_flows

  subroutine bar(n)
    integer, intent(inout) :: n
    ! TEST LESS THAN TWO POINTS
    if(n-1)  90,90,80
80  continue
    ! infinite loop, but never mind that
30  n = 1
    goto 30
90  return
  end subroutine bar

  subroutine unfixable_versions(x)
    integer, intent(inout) :: x
    if (x <= 0.0) goto 160
    x = 0
    ! Probably can't auto-fix because first block falls-through into second
    if (x) 130,170,140
130 continue
    x = 1
140 continue
    x = 2
    goto 170
160 continue
    x = 3
170 continue
  end subroutine unfixable_versions

end program test
