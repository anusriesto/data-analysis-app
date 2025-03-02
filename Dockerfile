FROM messense/rust-musl-cross:x86_64-musl as builder
# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ENV OPENAI_API_KEY=true
WORKDIR /data-analysis-app
#copy the source code
COPY . .
# Ensure target directory exists
#RUN mkdir -p /data-analysis-app/target/release

#Build the application
RUN cargo build --release --target x86_64-unknown-linux-musl

#Creating an image to run the binary
FROM debian:bullseye-slim
# Install OpenSSL runtime libraries
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /data-analysis-app/target/x86_64-unknown-linux-musl/release/data-analysis-app /data-analysis-app
ENTRYPOINT ["./data-analysis-app"]
EXPOSE 8000