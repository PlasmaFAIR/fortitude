MODULE BoundingBox_Method

PRIVATE

INTERFACE
  MODULE PURE SUBROUTINE initiate_1(obj, nsd, lim)
    CLASS(BoundingBox_), INTENT(INOUT) :: obj
  END SUBROUTINE initiate_1
END INTERFACE

INTERFACE
  MODULE FUNCTION Constructor_2(Anotherobj) RESULT(Ans)
    CLASS(BoundingBox_), POINTER :: Ans
  END FUNCTION Constructor_2
END INTERFACE

END MODULE BoundingBox_Method
