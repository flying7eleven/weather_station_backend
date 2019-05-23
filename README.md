# Weather Station (Backend)

## Development (or just building the tool)

### Prerequisites
Ensure you have the Rust toolchain installer `rustup` installed (e.g. through your
package manager). Now you have to ensure that a Rust toolchain is installed. To do that,
just type the following commands:

```text
rustup update
rustup toolchain install nightly
rustup default nightly
```

After running those commands, your Rust environment is set up. Since this project is using
`clippy` as a linter and `rustfmt` as a formatter, the next commands are required to install
them as default `cargo` commands:

```text
rustup component add clippy
rustup component add rustfmt
```

Perfect! At least your Rust environment is now set up properly and you can proceed :)

### Building the tool
For building a release version of the tool, just run

```text
cargo build --release
```

and wait until all dependencies are fetched, build and then the application is build. After
building it should be placed in `target/release`.
