ARG version_default=v1

FROM rust:1.92.0-alpine3.23 AS build

RUN set -ex && \
    apk add --no-progress --no-cache curl

WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs /app/
COPY ./src /app/src

COPY ./auth/Cargo.toml ./auth/Cargo.lock /app/auth/
COPY ./auth/src /app/auth/src

RUN cargo build --release

FROM alpine:3.23 AS run

RUN apk add --no-progress --no-cache tzdata curl jq

ENV UID=65532
ENV GID=65532
ENV USER=nonroot
ENV GROUP=nonroot

RUN addgroup -g $GID $GROUP && \
    adduser --shell /sbin/nologin --disabled-password \
    --no-create-home --uid $UID --ingroup $GROUP $USER

WORKDIR /app/
COPY --from=build /app/target/release/ttp-idm ./
COPY ./app.yaml ./
COPY ./resources/matching_config.xml ./resources/matching_config.xml
USER $USER
EXPOSE 3000

HEALTHCHECK --interval=1m --timeout=10s CMD curl -s http://localhost:3000/status | jq -e .healthy || exit 1

ENTRYPOINT ["/app/ttp-idm"]
