# ビルド環境
FROM rust:latest as builder
WORKDIR /app
COPY . .
# 依存関係のキャッシュを活用しつつビルド
RUN cargo build --release

# 実行環境 (軽量なDebian)
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ev_charging_api /usr/local/bin/
CMD ["ev_charging_api"]
