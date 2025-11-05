ARG ARCH
ARG VERSION
ARG VARIANT

FROM --platform=linux/amd64 flecspublic.azurecr.io/flecs-build AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        libssl-dev \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /base

# Rust toolchain
## Use the toolchain settings from the project
COPY app/backend/rust-toolchain.toml .

## This will install the toolchain and components from rust-toolchain.toml
RUN rustup show

## Targets have to be added additionally
RUN rustup target add x86_64-unknown-linux-gnu

ARG TARGET=x86_64-unknown-linux-gnu
ARG BUILD_TYPE="release"
ENV CARGO_ARGS="--target ${TARGET} --path ."

WORKDIR /src
COPY app/backend .

ENV RUSTFLAGS="-C target-feature=+crt-static"

RUN if [ "$BUILD_TYPE" = "debug" ]; then \
      cargo install $CARGO_ARGS --debug; \
    else \
      cargo install $CARGO_ARGS; \
    fi

# Move resulting binary to hardcoded path to avoid constructing path in last stage
RUN mv $CARGO_HOME/bin/user-manager /

FROM gcr.io/distroless/static-debian12

COPY --from=builder /user-manager /
COPY app/backend/static /static

EXPOSE 27000
ENTRYPOINT [ "/user-manager" ]
