real(8) function current_time()
  ! returns the wall clock time in seconds.
  use mpi
  current_time = mpi_wtime()
end function

type(object) function new_object
  ! returns the current date string
  TYPE(object) :: obj
  obj%counter = 0
  new_object = obj
end function

integer(8) function exported_fun(arg1, arg2) bind(c)
end function

complex(8) function exported_fun2() bind(c, name="f90_func")
end function

function exported_fun3() result(res) bind(c, name="fun3")
end function

function exported_fun4() bind(c, name="fun3") result(res)
end function
