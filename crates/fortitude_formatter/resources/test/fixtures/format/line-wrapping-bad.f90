program test_wrap_messy
    implicit none

    real :: AReallyReallyReallyLongVariableNameThatKeepsGoingForeverAndEver,anotherRidiculouslyLongName
    real :: res

    !--------------------------------------------------------------
    ! 7. Derived type init with no spacing and very long fields
    !--------------------------------------------------------------
    type some_type
        integer :: field_with_extremely_long_name
        character(len=:),allocatable :: another_field_that_is_absurdly_long
    end type some_type

    type(some_type) :: obj

    !--------------------------------------------------------------
    ! 1. No spaces around operators, very long identifiers
    !--------------------------------------------------------------
    AReallyReallyReallyLongVariableNameThatKeepsGoingForeverAndEver=anotherRidiculouslyLongName+123456789*987654321/111111111-222222222**10

    !--------------------------------------------------------------
    ! 2. Function call squashed together with no spaces
    !--------------------------------------------------------------
    callSomeProcedureWithAbsurdlyLongNameAndNoSpacesBetweenArguments(argOne,argTwo,argThree,argFour,argFive,argSix,argSeven,argEight,argNine,argTen) ! trailing comment at the end of a very long line

    !--------------------------------------------------------------
    ! 3. Array constructor messy formatting
    !--------------------------------------------------------------
    res=[1.0,2.0 ,3.0 ,  4.0,   5.0,6.0 ,7.0, 8.0 ,9.0,10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0,19.0,20.0,21.0,22.0]

    !--------------------------------------------------------------
    ! 4. Mixed operators with no whitespace
    !--------------------------------------------------------------
    res=(1+2*3/4-5**6+7*8/9-10)+(11*12-13/14+15**16)-(17*18/19+20**21-22) ! comment right after expression

    !--------------------------------------------------------------
    ! 5. Nested parentheses with inconsistent spacing
    !--------------------------------------------------------------
    res=(((AReallyReallyReallyLongVariableNameThatKeepsGoingForeverAndEver+anotherRidiculouslyLongName)*(res-123))/(456+789))**(2-3+4) ! trailing inline comment

    !--------------------------------------------------------------
    ! 6. Long string with concatenation and spacing mess
    !--------------------------------------------------------------
    print*, "This is a long string "   //    "with lots of unnecessary spacing "   //  "and it keeps going on and on just to exceed line length"//"and still more text" ! comment here

    obj=some_type(field_with_extremely_long_name=123456789,another_field_that_is_absurdly_long="RidiculouslyLongStringThatNeverSeemsToEndAndStillGoingAndGoing")

    !--------------------------------------------------------------
    ! 8. IF condition with no spaces and too many operators
    !--------------------------------------------------------------
    if(AReallyReallyReallyLongVariableNameThatKeepsGoingForeverAndEver>anotherRidiculouslyLongName.and.res<123.or.(res==456.and.res/=789).or.res>=101112.and.res<=131415)then;print*,"ugly branch";end if

end program test_wrap_messy
