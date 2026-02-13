PROGRAM TEST

INTEGER, PARAMETER :: N = 10
REAL, DIMENSION(N, N) :: A, B, C, D
INTEGER :: I, J

FORALL(I = 1:N, J = 1:N, A(I, J) .NE. 0.0) B(I, J) = 1.0 / A(I, J)

FORALL(J=1:8)  PATTERN(J)%P => OBJECT(1+IEOR(J-1,2))


FORALL (I = 1:N, J = 1:N)
  WHERE(A(I, J) .NE. 0.0) B(I, J) = 1.0/A(I, J)
END FORALL

FORALL(I = 3:N + 1, J = 3:N + 1, A(I, J) > 0.0)
  C(I, J) = C(I, J + 2) + C(I, J - 2) + C(I + 2, J) + C(I - 2, J)
  D(I, J) = C(I, J)
END FORALL

END PROGRAM
