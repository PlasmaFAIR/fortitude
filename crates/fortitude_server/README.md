# The Fortitude Language Server

`fortitude server` is a language server that powers Fortitude's editor integrations.

The job of the language server is to listen for requests from the client (in this case, the code editor of your choice)
and call into Fortitude's linter and formatter crates to construct real-time diagnostics or formatted code, which is then
sent back to the client. It also tracks configuration files in your editor's workspace, and will refresh its in-memory
configuration whenever those files are modified.

Refer to the [documentation](https://fortitude.readthedocs.io/en/stable/editors/) for more information on
how to set up the language server with your editor and configure it to your liking.

## Acknowledgements

`fortitude server` is (lightly) adapted from `ruff server`.

