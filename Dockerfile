FROM debian:bullseye

RUN apt update && \
    apt install -y libssl-dev curl

WORKDIR /app
ADD target/release/chart-parser /app/chart-parser
CMD ["/app/chart-parser"]
