FROM rust:slim-bullseye as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/tictactoe-rs /usr/local/bin/tictactoe-rs
CMD ["tictactoe-rs"]
EXPOSE 3000
