FROM rust:1.84-alpine as builder
WORKDIR /usr/src/helyi-torpe
RUN apk add musl-dev openssl-dev openssl-libs-static
COPY . .
RUN cargo install --path .

FROM alpine

RUN addgroup -S torpe && adduser -S torpe -G torpe

COPY --from=builder /usr/local/cargo/bin/helyi-torpe /usr/local/bin/helyi-torpe

USER torpe

ENTRYPOINT ["helyi-torpe"]
