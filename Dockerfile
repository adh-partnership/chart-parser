FROM alpine:3.18

WORKDIR /app
ADD target/release/chart-parser /app/chart-parser
