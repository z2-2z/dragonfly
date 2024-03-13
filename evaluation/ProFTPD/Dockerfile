FROM archlinux:latest

RUN useradd -M -r ftp && passwd -d ftp

WORKDIR /proftpd
COPY config .
COPY proftpd/proftpd .
COPY ftp.dict .
COPY ../../libdragonfly/build/libdragonfly.so .
COPY fuzzer/target/debug/fuzzer fuzzer-debug
COPY fuzzer/target/release/fuzzer fuzzer-release

VOLUME /ftproot
