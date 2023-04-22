FROM rust:latest

WORKDIR /usr/src/mylib

COPY . .

RUN cargo build --release

CMD ["cargo", "test", "--release", "--", "--nocapture"]