# FROM alpine:latest
FROM debian:latest

ADD "https://github.com/aws/aws-lambda-runtime-interface-emulator/releases/latest/download/aws-lambda-rie" /var/runtime/rie

RUN chmod +x /var/runtime/rie
RUN apt-get update
RUN apt-get install -y ca-certificates

ENV AWS_LAMBDA_FUNCTION_NAME="test"
ENV AWS_LAMBDA_FUNCTION_MEMORY_SIZE="3008"
ENV AWS_LAMBDA_FUNCTION_VERSION="1"
ENV AWS_LAMBDA_RUNTIME_API="localhost:9000"
ENV AWS_DEFAULT_REGION="us-west-2"
ENV RUST_BACKTRACE="1"

COPY "target/debug/axum-lambda" "/var/runtime/bootstrap"
CMD ["/var/runtime/rie", "/var/runtime/bootstrap"]
