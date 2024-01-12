/// Collection of useful patterns for interpretting Fortran code.

pub fn remove_line_wrapping(s: &str) -> String {
    String::from(s).replace("&\n", "");
}
