FROM rust:1.41 as builder

WORKDIR /usr/src/gitiful
COPY . .

RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies
COPY --from=builder /usr/local/cargo/bin/gitiful /usr/local/bin/gitiful

EXPOSE 8080

CMD ["gitiful"]
