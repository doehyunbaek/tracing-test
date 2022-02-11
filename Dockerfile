FROM rustlang/rust:nightly-stretch as builder
RUN apt-get update && apt-get -y install ca-certificates cmake build-essential libssl-dev && rm -rf /var/lib/apt/lists/*

# # ssh which can be skipped if there's no need to ssh to github 
# # WORKDIR /tmp
# ARG SSH_PRIVATE_KEY
# RUN mkdir ~/.ssh/
# RUN echo "$SSH_PRIVATE_KEY" >> ~/.ssh/id_rsa && chmod 600 ~/.ssh/id_rsa
# RUN touch ~/.ssh/known_hosts
# RUN ssh-keyscan -t rsa github.com >> ~/.ssh/known_hosts

# rust
RUN rustup toolchain remove nightly && rustup toolchain install nightly 
RUN rustup component add rustfmt
RUN rustup target add x86_64-unknown-linux-gnu
# Sets the environment variable for the cargo build command that follows.
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

#RUN rustup nightly
RUN mkdir /app
WORKDIR /app
RUN USER=root cargo new --bin tracing-test
ADD ./Cargo.lock /app/tracing-test
ADD ./Cargo.toml /app/tracing-test
WORKDIR /app/tracing-test

# build dependency first
# RUN eval `ssh-agent` && ssh-add -k ~/.ssh/id_rsa && cargo update --aggressive
# RUN eval `ssh-agent` && ssh-add -k ~/.ssh/id_rsa && cargo build --target x86_64-unknown-linux-gnu --release

RUN cargo update --aggressive
RUN cargo build --target x86_64-unknown-linux-gnu --release

# renew main.rs && rebuild
COPY ./src /app/tracing-test/src/
RUN echo "" >> /app/tracing-test/src/main.rs
# RUN eval `ssh-agent` && ssh-add -k ~/.ssh/id_rsa && cargo build --target x86_64-unknown-linux-gnu --release
RUN cargo build --target x86_64-unknown-linux-gnu --release



# lighter image
FROM frolvlad/alpine-glibc:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder /app/tracing-test/target/x86_64-unknown-linux-gnu/release/tracing-test .
ENTRYPOINT ["./tracing-test"]