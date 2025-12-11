use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use std::path::Path;
use tree_sitter::Node;

/// ## What it does
/// Checks for the use of the old deprecated `mpif.h` include.
///
/// ## Why is this bad?
/// MPI used to use the `mpif.h` include file to define its subroutines and
/// constants, however that method has now been replaced by either the (not-recommended)
/// `mpi` module, or the newer `mpi_f08` module. The `mpif.h` include is
/// deprecated as of MPI 4.1.
///
/// The `mpi` module is a drop-in replacement for the `mpif.h` include (except
/// for its placement in the Fortran file), while the `mpi_f08` module requires
/// more code changes to use the MPI derived types for the constants.
///
/// ## Examples
///
/// This MPI code using the `mpif.h` include
///
/// ```f90
/// program demo_mpi
///
/// implicit none
///
/// include "mpif.h"
///
/// integer :: mpicomm
/// integer :: mpiint
/// integer myrank, nproc, ierr
///
/// mpicomm = MPI_COMM_WORLD
/// mpiint = MPI_INTEGER
///
/// call MPI_Init(ierr)
/// call MPI_Comm_rank(mpicomm, myrank, ierr)
/// call MPI_Comm_size(mpicomm, nproc, ierr)
///
/// call MPI_finalize(ierr)
/// end
/// ```
///
/// can be converted to use the `mpi` module by simply changing the `include` statement
/// to a `use` statement
///
/// ```f90
/// program demo_mpi
/// use mpi
///
/// implicit none
///
/// integer :: mpicomm
/// integer :: mpiint
/// integer myrank, nproc, ierr
///
/// mpicomm = MPI_COMM_WORLD
/// mpiint = MPI_INTEGER
///
/// call MPI_Init(ierr)
/// call MPI_Comm_rank(mpicomm, myrank, ierr)
/// call MPI_Comm_size(mpicomm, nproc, ierr)
///
/// call MPI_finalize(ierr)
/// ```
///
/// or can be modernized to use the `mpi_f08` module
///
/// ```f90
/// program demo_mpi_f08
/// use mpi_f08
///
/// implicit none
///
/// type(MPI_comm) :: mpicomm
/// type(MPI_datatype) :: mpiint
/// integer myrank, nproc, ierr
///
/// mpicomm = MPI_COMM_WORLD
/// mpiint = MPI_INTEGER
///
/// call MPI_Init(ierr)
/// call MPI_Comm_rank(mpicomm, myrank, ierr)
/// call MPI_Comm_size(mpicomm, nproc, ierr)
///
/// call MPI_finalize(ierr)
/// end
/// ```
///
/// ## References
/// - Message Passing Interface Forum, MPI: A Message-Passing Interface Standard, Jun. 2025.
///   https://www.mpi-forum.org/docs/mpi-5.0/mpi50-report.pdf.
#[derive(ViolationMetadata)]
pub(crate) struct DeprecatedMPIInclude {}

impl Violation for DeprecatedMPIInclude {
    #[derive_message_formats]
    fn message(&self) -> String {
        "mpif.h include is deprecated, use mpi or mpi_f08 module instead".to_string()
    }
}

impl AstRule for DeprecatedMPIInclude {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        _src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let include_file = node
            .child_with_name("filename")?
            .to_text(_src.source_text())?
            .to_lowercase();

        // Strip quotes from the include file name
        let include_file = include_file.trim_matches('"').trim_matches('\'');

        if Path::new(&include_file).file_name() == Some("mpif.h".as_ref()) {
            return some_vec![Diagnostic::from_node(DeprecatedMPIInclude {}, node)];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["include_statement"]
    }
}
