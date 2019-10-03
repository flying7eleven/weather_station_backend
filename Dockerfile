FROM alpine:latest AS build_environment
USER root
WORKDIR /build
RUN apk update && apk add rust cargo
COPY . .
RUN cargo build --release

FROM alpine:latest
COPY --from=build_environment /build/target/release/weather_station_backend /usr/bin/weather_station_backend
RUN apk update && apk add gcompat libgcc
EXPOSE 8000
CMD ["/usr/bin/weather_station_backend", "run"]
