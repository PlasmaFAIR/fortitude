      SUBROUTINE CEBCHVXX( THRESH, PATH )
      CHARACTER FACT
      COMPLEX            ZDUM
!     .. Statement Functions ..
      REAL               CABS1
!     ..
!     .. Statement Function Definitions ..
      CABS1( ZDUM ) = ABS( REAL( ZDUM ) ) + ABS( AIMAG( ZDUM ) )

!     .. Parameters ..
      INTEGER            NWISE_I, CWISE_I
      PARAMETER          (NWISE_I = 1, CWISE_I = 1)

      FACT = 'E'
      END SUBROUTINE CEBCHVXX
