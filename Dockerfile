FROM rust:1.71 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/todo-ish todoish
ENV DATABASE_URL=""
ENV SERVER_PORT="8080"
CMD [ "./todoish" ]