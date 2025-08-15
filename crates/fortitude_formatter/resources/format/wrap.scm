(included_items 
[
  ("only" . ":" @break-after)
  ((_) "," @break-after)
]
) @root

(argument_list
    ["(" ","] @break-after

) @root

(parameters
    ["(" ","] @break-after
) @root

(variable_declaration
  "::" "," @break-after
) @root

(math_expression
  left: (_) @break-after
  ["*" "+" "-" "/"]
  right: (_)
) @root

(math_expression
  left: (_) @break-after
  "**"
  right: (_)
) @root


(logical_expression
  (_) @break-after
  .
  (_)
) @root


(concatenation_expression
  (_) @break-after . "//"
) @root

(array_literal
  "," @break-after
) @root
