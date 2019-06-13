FROM rustlang/rust:nightly
ADD https://github.com/ufoscout/docker-compose-wait/releases/download/2.5.0/wait /wait
RUN curl -fsSL -o /dbmate https://github.com/amacneil/dbmate/releases/download/v1.6.0/dbmate-linux-amd64 && chmod +x /wait && chmod +x /dbmate
COPY Cargo.toml /root/Cargo.toml
COPY build.rs /root/build.rs
COPY .gitmodules /root/.gitmodules
COPY .git /root/.git
WORKDIR /root
RUN git submodule update --init
RUN mkdir src && echo "fn main() {} // dummy" > src/main.rs && SKIP_BUILDRS=1 cargo build
COPY . /root
CMD /wait && /dbmate up && cargo run
