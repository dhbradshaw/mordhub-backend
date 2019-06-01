FROM rustlang/rust:nightly
ADD https://github.com/ufoscout/docker-compose-wait/releases/download/2.5.0/wait /wait
RUN chmod +x /wait && cargo install diesel_cli --no-default-features --features postgres
COPY Cargo.toml /root/Cargo.toml
COPY build.rs /root/build.rs
WORKDIR /root
RUN mkdir src && echo "fn main() {} // dummy" > src/main.rs && SKIP_BUILDRS=1 cargo build
COPY . /root
CMD /wait && diesel setup && cargo run
