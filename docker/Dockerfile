ARG TARGETPLATFORM

FROM --platform=linux/amd64 rust:slim AS builder

ARG TARGETPLATFORM

RUN dpkg --add-architecture arm64

RUN apt-get update
RUN apt-get --yes --no-install-recommends install \
  crossbuild-essential-arm64 \
  libssl-dev libssl-dev:arm64 \
  libzstd-dev libzstd-dev:arm64 \
  pkgconf pkgconf:arm64 \
  zlib1g-dev zlib1g-dev:arm64

WORKDIR /base

# Rust toolchain
## Use the toolchain settings from the project
COPY app/backend/rust-toolchain.toml .

## This will install the toolchain and components from rust-toolchain.toml
RUN rustup show

WORKDIR /src
COPY app/backend .

RUN \
  case ${TARGETPLATFORM} in \
    linux/amd64) \
      TRIPLET="x86_64-linux-gnu"; \
      TARGET="x86_64-unknown-linux-gnu"; \
      ;; \
    linux/arm64) \
      TRIPLET="aarch64-linux-gnu"; \
      TARGET="aarch64-unknown-linux-gnu"; \
      ;; \
        *) \
        echo "Unknown platform ${TARGETPLATFORM}" 1>&2; \
        exit 1; \
      ;; \
  esac; \
  rustup target add ${TARGET}; \
  CARGO_ARGS="--path . --target ${TARGET}"; \
  export OPENSSL_STATIC="1"; \
  export PKG_CONFIG_ALLOW_CROSS=1; \
  export PKG_CONFIG_LIBDIR=/usr/lib/${TRIPLET}/pkgconfig:/usr/share/pkgconfig; \
  export RUSTFLAGS="-C target-feature=+crt-static -C linker=${TRIPLET}-gcc"; \
  \
  if [ "$BUILD_TYPE" = "debug" ]; then \
    cargo install ${CARGO_ARGS} --debug; \
    mv $CARGO_HOME/bin/user-manager -o /user-manager; \
  else \
    cargo install ${CARGO_ARGS}; \
    ${TRIPLET}-strip --strip-all $CARGO_HOME/bin/user-manager -o /user-manager; \
  fi;

FROM gcr.io/distroless/static-debian12

COPY --from=builder /user-manager /
COPY app/backend/static /static

EXPOSE 27000
ENTRYPOINT [ "/user-manager" ]
