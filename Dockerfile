FROM rust:1.35.0
RUN mkdir -p /tmp/code
COPY ./ /tmp/code/
RUN cd /tmp/code/; \
    cargo build --release; \
    cp target/release/weather_station_backend /usr/bin/weather_station_backend; \
    cd /; \
    rm -rf /tmp/code/
CMD ["/usr/bin/weather_station_backend"]
