FROM rust:1.35.0-slim AS build_environment
USER root
WORKDIR /build
COPY . .
RUN apt-get update && apt-get install -y libmariadbclient-dev-compat
RUN cargo build --release

FROM rust:1.35.0-slim
COPY --from=build_environment /build/target/release/weather_station_backend /usr/bin/weather_station_backend
RUN apt-get update && apt-get install -y libmariadbclient-dev-compat
EXPOSE 8000
CMD ["/usr/bin/weather_station_backend"]
