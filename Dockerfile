FROM debian:bullseye as build_environment
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
USER root
RUN apt update && \
    apt install -y curl build-essential libssl-dev pkg-config git && \
    curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain stable -y
ENV PATH=/root/.cargo/bin:$PATH
RUN mkdir /build
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bullseye
RUN apt update && apt install -y openssl ca-certificates
COPY --from=build_environment /build/target/release/weather_station_backend /usr/bin/weather_station_backend
EXPOSE 5471
ENV RUST_BACKTRACE=full
CMD ["/usr/bin/weather_station_backend"]
