program mod1
  
  use mod, only: abc, def

  use mod, only: abc, def, aaa, bbb, ccc => as, ddd, eee, ffff, gggg, hhh, aaaaa, ccccc, ddddd

contains

  subroutine return_val(abc)
      integer, intent(out) :: abc

      abc = 0
      call some_other(abc, abc, abc, abc, abc, f(3, 1234, abc), abc, abc, abc, abc, abc, abc, abc, abc, abc, abc, abc)

      call some_other(abc, 1 + 2 / 2, abc)

      variable = foo + bar * (thing + (jaff + zang) - yeet) + huff


      a = 1 + 2 / 3
      a = 1 / 2 + 3
      return

  end subroutine

end program mod1
