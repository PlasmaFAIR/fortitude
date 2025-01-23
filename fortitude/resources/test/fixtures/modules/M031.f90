module test

include "common_variables"

contains

integer function func1()
    include "variable_declaration"

    func1 = i + 3
end function

end module test