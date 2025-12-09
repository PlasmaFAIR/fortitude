use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

/// ## What it does
/// Checks for the use of the old not-recommended `mpi` module.
///
/// ## Why is this bad?
/// MPI version 3.0 and later includes two modules: `mpi` and `mpi_f08`.
/// The original `mpi` module is inconsistent with the Fortran standard, and
/// is not recommended by the MPI standard. Instead, the `mpi_f08` module is
/// recommended because it is consistent with the Fortran standard (starting
/// with Fortran 2008).
///
/// The older `mpi` module and `mpif.h` include used integers for named
/// constants and the various MPI objects, which could lead to passing the wrong
/// constants into arguments (e.g., passing a communicator into an operation).
/// The new `mpi_f08` module uses derived types, allowing these errors to be
/// caught at compile-time.
///
/// Note: Switching from the `mpi` module to the `mpi_f08` module will require more
/// source code changes than just changing the `use` statement, since it uses
/// custom types for the operations and constants.
///
/// ## Examples
///
/// This MPI code using the `mpi` module
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
/// end
/// ```
///
/// can be converted to use the `mpi_f08` module
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
pub(crate) struct OldMPIModule {}

impl Violation for OldMPIModule {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Use of mpi module discouraged, use mpi_f08 instead".to_string()
    }
}

impl AstRule for OldMPIModule {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        _src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let module_name = node
            .child_with_name("module_name")?
            .to_text(_src.source_text())?
            .to_lowercase();

        if module_name == "mpi" {
            return some_vec![Diagnostic::from_node(OldMPIModule {}, node)];
        }
        None
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["use_statement"]
    }
}
