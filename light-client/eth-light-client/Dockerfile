FROM rustlang/rust:nightly as builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update --yes && \
    apt-get install --yes --no-install-recommends \
        libsqlite3-dev

WORKDIR /usr/src/app

COPY . .

RUN cargo build --locked --release


FROM debian:11 as production

ENV HOME /usr/src/app
WORKDIR $HOME

RUN apt-get update --yes && \
    apt-get install --yes --no-install-recommends \
        libsqlite3-dev

COPY --from=builder $HOME/target/release/eth-light-client ./target/release/eth-light-client

EXPOSE 5800

ENTRYPOINT [ "/usr/src/app/target/release/eth-light-client" ]
