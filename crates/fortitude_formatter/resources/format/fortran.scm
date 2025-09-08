; Don't format strings or comments
[
 (string_literal)
 (comment)
 (preproc_function_def)
 (preproc_def)
] @leaf


[
 "&"
 (comment)
 (else_clause)
 (elseif_clause)
 (subroutine)
 (subroutine_call)
 (derived_type_statement)
 (print_statement)
 "then"
 (variable_declaration)
 (common_statement)
 (data_statement)
 (private_statement)
 (public_statement)
 (use_statement)
 (allocate_statement)
 (deallocate_statement)
 (pointer_association_statement)
 (assignment_statement)
 (sequence_statement)
 (preproc_function_def)
 (preproc_def)
 ";"
] @append_hardline @allow_blank_line_before

(preproc_if . ("#if" @append_space . (identifier) @append_hardline))

(
    do_loop_statement "do" @append_space 
    (loop_control_expression) @append_hardline @append_indent_start
    (end_do_loop_statement) @prepend_indent_end .
)

(if_statement "if" @append_space)
(elseif_clause "elseif" @append_space)
(elseif_clause "elseif" @append_space)
(enumerator_statement "enumerator" @append_space)
(assignment "assignment" @append_space)

(select_case_statement
    . "select" @append_space
    "case" (selector) @append_hardline
    (end_select_statement . "end" @append_space . "select" @append_hardline) .
) 

(select_case_statement
    (case_statement 
        . "case" . "(" . (_) . ")" @append_hardline @append_indent_start
        ) @append_indent_end
) 

(select_case_statement
    (case_statement
        . "case" @append_space . (default) @append_hardline @append_indent_start
        ) @append_indent_end
) 


[
 ","
 (parenthesized_expression)
 (abstract_specifier)
 (procedure_qualifier)
 (block_label_start_expression)
 (intrinsic_type)
 (derived_type)
] @append_space

; start end blocks (hardline)
[
; start
(program_statement)
(module_statement)
(submodule_statement)
(subroutine_statement)
(interface_statement)
(function_statement)
(procedure_statement)
(derived_type_definition)
(block_data_statement)
(enumeration_type_statement)
(assignment_statement)
(contains_statement)
(keyword_statement)
(enumerator_statement)
(do_loop_statement)
(if_statement)
; end
(end_program_statement)
(end_module_statement)
(end_submodule_statement)
(end_subroutine_statement)
(end_interface_statement)
(end_function_statement)
(end_type_statement)
(end_block_data_statement)
(end_enumeration_type_statement)
(implicit_statement)
(end_block_construct_statement)
] @append_hardline

; start end blocks (end space)
; TODO can we edit grammer.js to rename to end and _
; TODO this doesnt work as intended because tree-sitter currently combines end_
; into one node .e.g., it will keep "end program" separated but it wont
; separate "endprogram"
(end_program_statement "end" @append_space . "program")
(end_module_statement "end" @append_space . "module")
(end_submodule_statement "end" @append_space . "submodule")
(end_subroutine_statement "end" @append_space . "subroutine")
(end_interface_statement "end" @append_space . "interface")
(end_function_statement "end" @append_space . "function")
(end_type_statement "end" @append_space . "type")
(end_block_construct_statement "end" @append_space . "block")
(end_enumeration_type_statement "end" @append_space . "enumeration" @append_space)

("&" @prepend_space . "&")

(parenthesized_expression
  "(" @append_indent_start
  ")" @prepend_indent_end
)

(argument_list
  "(" @append_indent_start
  ")" @prepend_indent_end
)

(parameters
  "(" @append_indent_start
  ")" @prepend_indent_end
)

(included_items
  "only" ":" @append_indent_start
) @append_indent_end

(assignment_statement
    right: [(math_expression) (logical_expression)] @prepend_indent_start
) @append_indent_end

; TODO: How to do flattened math expresssions?

(enumeration_type_statement "enumeration" @append_space . "type")

(implicit_statement _ @append_space)
(derived_type_statement _ @append_space)
(block_data_statement _ @append_space)
(common_statement _ @append_space)
(data_statement _ @append_space)
(interface_statement _ @append_space)
(use_statement _ @append_space)
(included_items _ @append_space)
(private_statement _ @append_space)
(public_statement _ @append_space)
(end_interface_statement _ @append_space)

(procedure_kind
    _ @append_space
(#not-eq? @append_space "(")
)

(subroutine_call "call" @append_space)

(block_construct "block" @append_hardline @append_indent_start)

[
 (program_statement)
 (module_statement)
 (submodule_statement)
 (subroutine_statement)
 (interface_statement)
 (function_statement)
 (derived_type_statement)
 (block_data_statement)
 (enumeration_type_statement)
 "contains"
] @append_indent_start

[
 (end_program_statement)
 (end_module_statement)
 (end_subroutine_statement)
 (end_interface_statement)
 (end_function_statement)
 (end_type_statement)
 (end_block_data_statement)
 (end_block_construct_statement)
 (end_submodule_statement)
 (end_enumeration_type_statement)
 "contains"
] @prepend_indent_end

(if_statement
    "if" @append_indent_start
    (end_if_statement) @prepend_indent_end .
)

(if_statement
    [
        (elseif_clause)
        (else_clause "else" @append_indent_start @append_hardline)
    ] @prepend_indent_end
)


[
 (function_result)
 (language_binding)
 (block_label)
] @prepend_space

[
 (assumed_size)
] @leaf

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
  "*" ;TODO format if not format_identifier or assumed_size
  "/"
] @prepend_space @append_space

(logical_expression
    left: (_) @append_space
    right: (_) @prepend_space
)

[
 (format_identifier)
] @append_antispace

[
 (included_items)
] @prepend_antispace

(subroutine_statement "(" @prepend_antispace)
(subroutine_statement (parameters) @prepend_antispace)
(function_statement "(" @prepend_antispace)
(function_statement (parameters) @prepend_antispace)
(submodule_statement (module_name) @prepend_antispace @append_antispace)
("/" . (_) @prepend_antispace @append_antispace . "/")
; remove space around operators and assignment
(assignment "(" . _ @prepend_antispace @append_antispace . ")")
(operator "(" . _ @prepend_antispace @append_antispace . ")")

; make sure anything preceeding a comma "," has no space
((_) @append_antispace . ",")

; make sure anything preceeding a semicolon ";" has no space
((_) @append_antispace . ";")

(derived_type_statement
_ @append_antispace
.
","
)
