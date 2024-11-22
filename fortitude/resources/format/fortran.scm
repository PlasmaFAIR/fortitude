[
 (program_statement)
 (module_statement)
 (variable_declaration)
 (assignment_statement)
 (if_statement)
 (do_loop_statement)
 (elseif_clause)
 (else_clause)
 "then"
 "&"
] @append_hardline

(if_statement "if" @append_space)
(elseif_clause "elseif" @append_space)

[
 ","
 (parenthesized_expression)
] @append_space

[
 (program_statement)
 (module_statement)
 (subroutine_statement)
 (function_statement)
 "then"
 "contains"
] @append_indent_start

[
 (end_program_statement)
 (end_module_statement)
 (end_subroutine_statement)
 (end_function_statement)
 (else_clause)
 (elseif_clause)
 "contains"
] @prepend_indent_end

[
 (else_clause)
 (elseif_clause)
] @append_indent_end

[
  (name)
  "::"
  "="
  "<"
  "<="
  "=="
  "=>"
  ">"
  "/="
  "+"
  "-"
  "*"
  "/"
] @prepend_space @append_space

[
 ","
] @prepend_antispace
