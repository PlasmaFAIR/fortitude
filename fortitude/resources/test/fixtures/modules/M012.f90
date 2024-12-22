module my_module
    use :: iso_fortran_env, only: real32
    use, intrinsic :: iso_c_binding
    use, non_intrinsic :: iso_c_binding
    use :: my_other_module
end module my_module
