FROM archlinux:latest AS build_environment
USER root
WORKDIR /build
RUN pacman -Sy && pacman --noconfirm -S rustup clang
RUN rustup install nightly
COPY . .
RUN cargo build --release

FROM archlinux:latest
COPY --from=build_environment /build/target/release/weather_station_backend /usr/bin/weather_station_backend
EXPOSE 8000
CMD ["/usr/bin/weather_station_backend", "run"]
