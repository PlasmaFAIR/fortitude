# Editor Integrations

Fortitude can be integrated with various editors and IDEs to provide a seamless development experience.
This section provides instructions on [how to set up Fortitude with your editor](./setup.md) and [configure it to your
liking](./settings.md).

## Language Server Protocol

The editor integration is mainly powered by the Fortitude Language Server which implements the
[Language Server Protocol](https://microsoft.github.io/language-server-protocol/). The server is
written in Rust and is available as part of the `fortitude` CLI via `fortitude server`.

The server supports surfacing Fortitude diagnostics, providing Code Actions to fix
them. Currently, the server is intended to be used alongside another Fortran Language
Server (such as [`fortls`](https://github.com/fortran-lang/fortls/) in order to support
features like navigation and autocompletion.
