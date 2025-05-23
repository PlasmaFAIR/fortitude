program foo
  use, intrinsic :: iso_fortran_env, only: team_type
  implicit none (external, type)

  integer :: i
  type(team_type) :: parent

  label1: associate(x => 1 + 1)
  end associate

  label2: block
  end block

  label3: critical
  end critical

  form team(1, parent)
  label4: change team(parent)
  end team

  label5: do i = 1, 10
  end do

  label6: forall(i=1:3)
  end forall

  label7: if (.true.) then
  end if

  label8: select case(i)
  end select

  label9: select rank(i)
  end select

  label10: select type(i)
  end select

  label11: where(i > 0)
  end where
  
  label_yes1: associate(x => 1 + 1)
  end associate label_yes1

  label_yes2: block
  end block label_yes2

  label_yes3: critical
  end critical label_yes3

  form team(1, parent)
  label_yes4: change team(parent)
  end team label_yes4

  label_yes5: do i = 1, 10
  end do label_yes5

  label_yes6: forall(i=1:3)
  end forall label_yes6

  label_yes7: if (.true.) then
  end if label_yes7

  label_yes8: select case(i)
  end select label_yes8

  label_yes9: select rank(i)
  end select label_yes9

  label_yes10: select type(i)
  end select label_yes10

  label_yes11: where(i > 0)
  end where label_yes11

end program foo
