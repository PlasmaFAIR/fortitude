! If you touch this file, be careful your editor doesn't auto-correct
! the horrible whitespace!
program invalid_tab
	implicit none
    print*, "Don't flag this:	"
    ! Or this:|	|
    if	(.true.)	then
    	print*, "Mixed tab/space"
    else
		print*, "Two tabs"
    end if
end program invalid_tab
