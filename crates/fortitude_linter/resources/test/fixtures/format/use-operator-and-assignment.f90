module test
  use other, only: operator(==), assignment(=)
  public :: assignment(=)
  private operator(==)
end module
