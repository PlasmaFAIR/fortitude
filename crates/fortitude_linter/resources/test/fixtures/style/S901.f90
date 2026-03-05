program test_program_too_complex ! should trigger
  implicit none(external, type)
  real :: x
  integer :: a, b, c, d, e
  x = 10.0
  if (x > 1.0) a = 1
  if (x > 2.0) b = 2
  if (x > 3.0) c = 3
  if (x > 4.0) d = 4
  if (x > 5.0) e = 5
end program test_program_too_complex

module test_module_too_complex
  implicit none(external, type)
  private
contains
  subroutine test_simple(x, y) ! should not trigger
    real, intent(in) :: x
    integer, intent(out) :: y
    if (x < 0.0) then
      y = 1
    else
      y = 2
    end if
  end subroutine test_simple

  subroutine inline_ifs(x, a, b, c, d, e) ! should trigger
    real, intent(in) :: x
    integer, intent(out) :: a, b, c, d, e
    if (x > 1.0) a = 1
    if (x > 2.0) b = 2
    if (x > 3.0) c = 3
    if (x > 4.0) d = 4
    if (x > 5.0) e = 5
  end subroutine inline_ifs

  subroutine test_loops(n, m, arr) ! should not trigger
    integer, intent(in) :: n, m
    real, intent(out) :: arr(n, m)
    integer :: i, j
    do i = 1, n
      do j = 1, m
        if (i == j) then
          arr(i, j) = 1.0
        else if (i > j) then
          arr(i, j) = 2.0
        else
          arr(i, j) = 0.0
        end if
      end do
    end do
  end subroutine test_loops

  subroutine test_nested_loops_complex(n, m, arr, result, flag) ! should trigger
    integer, intent(in) :: n, m, flag
    real, intent(in) :: arr(n, m)
    real, intent(out) :: result(n, m)
    integer :: i, j
    do i = 1, n
      do j = 1, m
        if (flag == 0) then
          if (arr(i, j) > 0.0) then
            result(i, j) = arr(i, j)*2.0
          else if (arr(i, j) < -100.0) then
            result(i, j) = -100.0
          else
            result(i, j) = 0.0
          end if
        else if (flag == 1) then
          if (arr(i, j) > 100.0) then
            result(i, j) = 100.0
          else
            result(i, j) = arr(i, j)
          end if
        else
          result(i, j) = 0.0
        end if
      end do
    end do
  end subroutine test_nested_loops_complex

  subroutine test_too_complex(x, y) ! should trigger
    real, intent(in) :: x
    integer, intent(out) :: y
    if (x < 0.0) then
      if (x < -100.0) then
        y = 1
      else if (x < -10.0) then
        y = 2
      else
        y = 3
      end if
    else if (x == 0.0) then
      y = 4
    else
      if (x > 100.0) then
        y = 5
      else if (x > 10.0) then
        y = 6
      else
        y = 7
      end if
    end if
  end subroutine test_too_complex

  subroutine test_simple_enough(x, y) ! should not trigger
    real, intent(in) :: x
    integer, intent(out) :: y
    if (x < 0.0) then
      y = -1
    else
      y = 1
    end if
  end subroutine test_simple_enough

  integer function test_function(x, threshold, scale, offset, flag) ! should trigger
    real, intent(in) :: x, threshold, scale, offset
    integer, intent(in) :: flag
    if (flag == 0) then
      if (x > threshold) then
        classify_sign = 1
      else if (x < -threshold) then
        classify_sign = -1
      else
        classify_sign = 0
      end if
    else if (flag == 1) then
      if (x*scale > offset) then
        classify_sign = 2
      else
        classify_sign = -2
      end if
    else
      classify_sign = 99
    end if
  end function test_function

  subroutine test_where(n, x, y, z, w, v) ! should trigger
    integer, intent(in) :: n
    real, intent(inout) :: x(n), y(n), z(n), w(n), v(n)
    where (x > 0.0)
      y = x*2.0
    end where
    where (y > 0.0)
      z = y*2.0
    end where
    where (z > 0.0)
      w = z*2.0
    end where
    where (w > 0.0)
      v = w*2.0
    end where
    if (n > 10) x(1) = 0.0
    do i = 1, n
    end do
  end subroutine test_where

  subroutine test_do_loops(n, m, p, arr) ! should trigger
    integer, intent(in) :: n, m, p
    real, intent(inout) :: arr(n, m, p)
    integer :: i, j, k
    do i = 1, n
      do j = 1, m
        do k = 1, p
          if (arr(i, j, k) > 0.0) then
            arr(i, j, k) = arr(i, j, k)*2.0
          else if (arr(i, j, k) < -1.0) then
            arr(i, j, k) = -1.0
          end if
        end do
      end do
    end do

    do while (x > 0.0)
      x = x - 1.0
    end do
  end subroutine test_do_loops

  subroutine test_do_while(x, y, z, w, tol) ! should trigger
    real, intent(inout) :: x, y, z, w
    real, intent(in) :: tol
    do while (x > tol)
      x = x*0.5
    end do
    do while (y > tol)
      y = y*0.5
    end do
    do while (z > tol)
      z = z*0.5
    end do
    do while (w > tol)
      w = w*0.5
    end do
    if (x < 0.0) x = 0.0
    if (y < 0.0) y = 0.0
  end subroutine test_do_while

  subroutine test_select_case(x, y, z, w, tol) ! should trigger
    CHARACTER(LEN=4) :: Title
    INTEGER, save :: DrMD = 0, PhD = 0, MS = 0, BS = 0, MR = 0, Others = 0
    SELECT CASE (Title)
    CASE ("DrMD")
      DrMD = DrMD + 1
    CASE ("PhD")
      PhD = PhD + 1
    CASE ("MS")
      MS = MS + 1
    CASE ("BS")
      BS = BS + 1
    CASE ("MR")
      MR = MR + 1
    CASE DEFAULT
      Others = Others + 1
    END SELECT
  end subroutine test_select_case

  subroutine test_logical_expression(a, b, c, d, e, f, g, h, i, x) ! should trigger
    real, intent(in) :: a, b, c, d, e, f, g, h, i
    integer, intent(out) :: x
    if (a > 0.0 .and. b > 0.0 .and. c > 0.0) then   ! if +1, .and. +1, .and. +1 = 4
      x = 1
    else if (d > 0.0 .or. e > 0.0 .or. f > 0.0) then ! elseif +1, .or. +1, .or. +1 = 7
      x = 2
    else if (g > 0.0 .and. h > 0.0 .or. i > 0.0) then ! elseif +1, .and. +1, .or. +1 = 10
      x = 3
    else
      x = 4
    end if
  end subroutine test_logical_expression

  !! Full test example, should trigger
  subroutine classify_orbit(semi_major, eccentricity, inclination, &
                            period, body_mass, orbit_type, stability)
    real, intent(in)               :: semi_major, eccentricity
    real, intent(in)               :: inclination, period, body_mass
    character(len=20), intent(out) :: orbit_type
    logical, intent(out)           :: stability

    real    :: velocity, escape_vel, hill_radius
    integer :: inc_class

    stability = .true.
    orbit_type = "unknown"

    ! Classify by eccentricity
    select case (nint(eccentricity*10))
    case (0)                                              ! case +1 = 2
      orbit_type = "circular"
    case (1:6)                                            ! case +1 = 3
      orbit_type = "elliptical"
    case (7:9)                                            ! case +1 = 4
      orbit_type = "highly_elliptical"
    case default
      orbit_type = "hyperbolic"
      stability = .false.
    end select

    ! Check orbital stability conditions
    hill_radius = semi_major*(body_mass/3.0)**(1.0/3.0)
    escape_vel = sqrt(2.0*6.674e-11*body_mass/semi_major)
    velocity = sqrt(6.674e-11*body_mass/semi_major)

    if (eccentricity >= 1.0 .or. semi_major < 0.0) then    ! if +1, .or. +1 = 6
      stability = .false.
      orbit_type = "escape_trajectory"
      return
    end if

    ! Inclination classification
    inc_class = 0
    if (inclination < 30.0) then                           ! if +1 = 7
      inc_class = 1
    else if (inclination < 90.0) then                      ! elseif +1 = 8
      inc_class = 2
    else if (inclination < 150.0) then                     ! elseif +1 = 9
      inc_class = 3
    else
      inc_class = 4
    end if

    ! Resonance check with simple iteration
    do while (velocity > escape_vel*0.1 .and. &
              hill_radius > semi_major*0.01)             ! do +1, .and. +1 = 11
      velocity = velocity*0.99
      hill_radius = hill_radius*0.99
      if (velocity < escape_vel*0.05) then               ! if +1 = 12
        stability = .false.
        exit
      end if
    end do

    ! Final stability check based on inclination class
    if (inc_class == 4 .and. eccentricity > 0.8) then      ! if +1, .and. +1 = 14
      stability = .false.
    end if

  end subroutine classify_orbit
end module test_module_too_complex
