# Weather Station (Backend)

![Build and test](https://github.com/flying7eleven/weather_station_backend/workflows/Build%20and%20test/badge.svg)

## Development (or just building the tool)

### Prerequisites
Ensure you have the Rust toolchain installer `rustup` installed (e.g. through your
package manager). Now you have to ensure that a Rust toolchain is installed. To do that,
just type the following commands:

```text
rustup update
rustup toolchain install nightly
rustup override set nightly
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

### Build a container, exporting and importing it

```
docker build -t weatherstation:0.4.1 .
docker image save weatherstation:0.4.1 | xz -zc - > weather_station_backend_0_4_1.tar.xz
```
