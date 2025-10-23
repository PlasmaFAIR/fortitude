# Preview

Some fortitude rules are only available through an opt-in preview
mode to give the community some time to evaluate them and provide
feedback. To enable preview rules, pass the `--preview` flag to
`check`,

Preview mode enables a collection of unstable features such as new lint rules
and fixes, interface updates, and more. Warnings about deprecated features may
turn into errors when using preview mode.

## Enabling preview mode

Preview mode can be enabled with the `--preview` flag on the CLI or by setting
`preview = true` in your Fortitude configuration file.

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    preview = true
    ```

=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    preview = true
    ```

=== "CLI"

    ```console
    fortitude check --preview
    ```


## Using rules that are in preview

If a rule is marked as preview, it can only be selected if preview mode is
enabled. For example, consider a hypothetical rule, `HYP001`. If `HYP001` were
in preview, it would _not_ be enabled by adding it to the selected rule set.

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    extend-select = ["HYP001"]
    ```

=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    extend-select = ["HYP001"]
    ```

=== "CLI"

    ```console
    fortitude check --extend-select HYP001
    ```


It also would _not_ be enabled by selecting the `HYP` category, like so:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    extend-select = ["HYP"]
    ```

=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    extend-select = ["HYP"]
    ```

=== "CLI"

    ```console
    fortitude check --extend-select HYP
    ```


Similarly, it would _not_ be enabled via the `ALL` selector:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    select = ["ALL"]
    ```

=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    select = ["ALL"]
    ```

=== "CLI"

    ```console
    fortitude check --select ALL
    ```

However, it _would_ be enabled in any of the above cases if you enabled preview mode:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    extend-select = ["HYP"]
    preview = true
    ```

=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    extend-select = ["HYP"]
    preview = true
    ```

=== "CLI"

    ```console
    fortitude check --extend-select HYP --preview
    ```

To see which rules are currently in preview, visit the [rules reference](rules.md).
