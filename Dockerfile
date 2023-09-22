FROM rust:1.72-bullseye as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN rm -rf /var/lib/apt/lists/*

ARG config_location=/usr/local

COPY --from=builder /app/target/release/network-tic-tac-toe /usr/local/bin/network-tic-tac-toe
COPY configuration ${config_location}/configuration

ENV NTTT__CONFIG_LOCATION=${config_location} NTTT__ENVIRONMENT=production NTTT__HOST=0.0.0.0

CMD ["network-tic-tac-toe"]
