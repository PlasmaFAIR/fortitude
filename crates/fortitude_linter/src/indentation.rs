use crate::display_settings;
use fortitude_macros::kind;
use ruff_macros::CacheKey;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU8;

/// Per-construct indents; `None` falls back to the resolved `check.indent-width`.
#[derive(Debug, Clone, Default, CacheKey)]
pub struct IndentationSettings {
    pub program_indent: Option<u8>,
    pub module_indent: Option<u8>,
    pub procedure_indent: Option<u8>,
    pub derived_type_indent: Option<u8>,
    pub control_flow_indent: Option<u8>,
    pub interface_indent: Option<u8>,
}

impl fmt::Display for IndentationSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_settings! {
            formatter = f,
            namespace = "check.incorrect-indentation",
            fields = [
                self.program_indent | optional,
                self.module_indent | optional,
                self.procedure_indent | optional,
                self.derived_type_indent | optional,
                self.control_flow_indent | optional,
                self.interface_indent | optional,
            ]
        }
        Ok(())
    }
}

/// The size of a tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, CacheKey)]
pub struct IndentWidth(NonZeroU8);

impl IndentWidth {
    pub fn as_usize(self) -> usize {
        self.0.get() as usize
    }

    pub fn as_u8(self) -> u8 {
        self.0.get()
    }
}

impl Default for IndentWidth {
    fn default() -> Self {
        Self(NonZeroU8::new(4).unwrap())
    }
}

impl fmt::Display for IndentWidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<NonZeroU8> for IndentWidth {
    fn from(tab_size: NonZeroU8) -> Self {
        Self(tab_size)
    }
}

impl From<usize> for IndentWidth {
    fn from(tab_size: usize) -> Self {
        Self(NonZeroU8::new(tab_size as u8).unwrap())
    }
}

impl From<IndentWidth> for NonZeroU8 {
    fn from(value: IndentWidth) -> Self {
        value.0
    }
}

/// The set of syntax kinds that should trigger an increase in indentation
/// on the following line. Note that some of these, such as `else`, are
/// also dedentors.
pub(crate) const POST_INDENTORS: [u16; 22] = [
    kind!("program_statement"),
    kind!("module_statement"),
    kind!("submodule_statement"),
    kind!("subroutine_statement"),
    kind!("function_statement"),
    kind!("derived_type_statement"),
    kind!("block_construct"),
    kind!("if_statement"),
    kind!("interface_statement"),
    kind!("select_case_statement"),
    kind!("select_type_statement"),
    kind!("select_rank_statement"),
    kind!("do_statement"),
    kind!("associate_statement"),
    kind!("where_statement"),
    kind!("contains_statement"),
    kind!("case_statement"),
    kind!("type_statement"),
    kind!("rank_statement"),
    kind!("else_clause"),
    kind!("elseif_clause"),
    kind!("elsewhere_clause"),
];

/// The set of syntax kinds that should trigger a decrease in indentation as
/// soon as they appear. Note that some of these, such as `else`, are also
/// post-indentors.
pub(crate) const DEDENTORS: [u16; 21] = [
    kind!("end_program_statement"),
    kind!("end_module_statement"),
    kind!("end_submodule_statement"),
    kind!("end_subroutine_statement"),
    kind!("end_function_statement"),
    kind!("end_type_statement"),
    kind!("end_block_construct_statement"),
    kind!("end_if_statement"),
    kind!("end_interface_statement"),
    kind!("end_select_statement"),
    kind!("end_do_loop_statement"),
    kind!("end_do_label_loop_statement"),
    kind!("end_associate_statement"),
    kind!("end_where_statement"),
    kind!("contains_statement"),
    kind!("case_statement"),
    kind!("type_statement"),
    kind!("rank_statement"),
    kind!("else_clause"),
    kind!("elseif_clause"),
    kind!("elsewhere_clause"),
];

pub(crate) struct IndentMap {
    default_indent: u8,
    program_indent: Option<u8>,
    module_indent: Option<u8>,
    procedure_indent: Option<u8>,
    derived_type_indent: Option<u8>,
    control_flow_indent: Option<u8>,
    interface_indent: Option<u8>,
}

impl IndentMap {
    pub fn default_indent(&self) -> u8 {
        self.default_indent
    }

    pub fn program_indent(&self) -> u8 {
        self.program_indent.unwrap_or(self.default_indent)
    }

    pub fn module_indent(&self) -> u8 {
        self.module_indent.unwrap_or(self.default_indent)
    }

    pub fn procedure_indent(&self) -> u8 {
        self.procedure_indent.unwrap_or(self.default_indent)
    }

    pub fn derived_type_indent(&self) -> u8 {
        self.derived_type_indent.unwrap_or(self.default_indent)
    }

    pub fn control_flow_indent(&self) -> u8 {
        self.control_flow_indent.unwrap_or(self.default_indent)
    }

    pub fn interface_indent(&self) -> u8 {
        self.interface_indent.unwrap_or(self.default_indent)
    }

    pub fn get_indent(&self, construct: &u16) -> u8 {
        match construct {
            kind!("program_statement") => self.program_indent(),
            kind!("module_statement") | kind!("submodule_statement") => self.module_indent(),
            kind!("subroutine_statement") | kind!("function_statement") => self.procedure_indent(),
            kind!("derived_type_statement") => self.derived_type_indent(),
            kind!("block_construct")
            | kind!("if_statement")
            | kind!("where_statement")
            | kind!("select_case_statement")
            | kind!("select_type_statement")
            | kind!("select_rank_statement")
            | kind!("do_statement")
            | kind!("associate_statement")
            | kind!("case_statement")
            | kind!("type_statement")
            | kind!("rank_statement")
            | kind!("else_clause")
            | kind!("elseif_clause")
            | kind!("elsewhere_clause") => self.control_flow_indent(),
            kind!("interface_statement") => self.interface_indent(),
            _ => self.default_indent(),
        }
    }

    pub fn from_settings(settings: &IndentationSettings, default_indent: u8) -> Self {
        Self {
            default_indent,
            program_indent: settings.program_indent,
            module_indent: settings.module_indent,
            procedure_indent: settings.procedure_indent,
            derived_type_indent: settings.derived_type_indent,
            control_flow_indent: settings.control_flow_indent,
            interface_indent: settings.interface_indent,
        }
    }
}

impl Default for IndentMap {
    fn default() -> Self {
        Self {
            default_indent: 4,
            program_indent: None,
            module_indent: None,
            procedure_indent: None,
            derived_type_indent: None,
            control_flow_indent: None,
            interface_indent: None,
        }
    }
}
