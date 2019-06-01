FROM rustlang/rust:nightly
COPY . /root
WORKDIR /root/
RUN cargo install diesel_cli --no-default-features --features postgres
RUN diesel setup
RUN cargo build
CMD ["cargo", "run"]
