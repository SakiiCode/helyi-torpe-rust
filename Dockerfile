FROM rust:1.84 as builder
WORKDIR /usr/src/helyi-torpe
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim

RUN groupadd -r torpe && useradd -r -g torpe torpe

RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/helyi-torpe /usr/local/bin/helyi-torpe

USER torpe

ENTRYPOINT ["helyi-torpe"]
