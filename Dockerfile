FROM rust:1.35.0-slim AS build_environment
USER root
WORKDIR /build
COPY . .
RUN cargo build --release

FROM rust:1.35.0-slim
COPY --from=build_environment /build/target/release/weather_station_backend /usr/bin/weather_station_backend
EXPOSE 8000
CMD ["/usr/bin/weather_station_backend"]
