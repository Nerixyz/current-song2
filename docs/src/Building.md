# Building

You need to have [Rust (and Cargo)](https://www.rust-lang.org/learn/get-started) for the server and [Node.js (and npm)](https://nodejs.org) for the overlay and extension installed.

If you don't have `pnpm` installed, install it either using the [instruction on their documentation](https://pnpm.io/installation) or though `npm`:

```sh
npm i -g pnpm
```

1. Install the dependencies

    ```sh
    pnpm i --frozen-lockfile
    ```

2. Build the extension and overlay

    ```sh
    pnpm run --if-present -r build
    ```

    This will build both projects. You can build a specific one by running `pnpm run build` in the respective folder. For example, to build the overlay, run `cd js/client && pnpm run build`. Or, you can use the `--filter` (e.g. `pnpm run --if-present -r --filter client build`).

3. Build the executable

    ```sh
    cargo build -r --locked
    ```

    This will build the executable in release mode. To build in debug mode, leave out the `-r`. For local development, it's useful to not bundle the overlay inside the executable and be able to stop the app with <kbd>CTRL</kbd> + <kbd>C</kbd>. To achieve this, add `--no-default-features` (this will disable the `single-executable` and `win32-executable` features). Furthermore, it's useful to run the app in the project-directory (instead of `target/{debug,release}`). To do this, add `--current-dir` when running. A good command-line for development is:

    ```sh
    cargo r --no-default-features -- --current-dir
    ```

## Feature Flags

By default, both features are enabled. To disable them, use `--no-default-features` (see `cargo b -h` for more details).

### `win32-executable`

When enabled, the executable will be compiled with the WINDOWS subsytem over the default CONSOLE subsystem. This effectively causes the application to not spawn a console window when started, but when started from the console, <kbd>CTRL</kbd> + <kbd>C</kbd> won't be honored.

### `single-executable`

When enabled, the overlay will be bundled into the executable.

## Command Line Arguments

### `--current-dir`

This flag causes the server to use the current working directory. By default (when not specified), it will use the directory of the executable as the working directory.

### `--remove-autostart`

This flag instructs the server to remove the autostart entry. After removing the entry, the server exits.
