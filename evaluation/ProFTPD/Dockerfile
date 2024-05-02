FROM archlinux:latest

RUN pacman -Sy --noconfirm libidn gdb
RUN userdel -f ftp && useradd -M -r ftp && passwd -d ftp
RUN useradd -M -r user && (echo user; echo user) | passwd user

WORKDIR /proftpd
COPY evaluation/ProFTPD/config ./config
COPY evaluation/ProFTPD/proftpd/proftpd-fuzzing ./proftpd-fuzzing
COPY evaluation/ProFTPD/proftpd/proftpd-debug ./proftpd-debug
COPY evaluation/ProFTPD/ftp.dict ./ftp.dict
COPY libdragonfly/build/libdragonfly.so ./libdragonfly.so
COPY evaluation/ProFTPD/fuzzer/target/release/fuzzer ./fuzzer
COPY dragonfly/target/debug/packets ./packets
COPY evaluation/ProFTPD/entrypoint.sh ./entrypoint.sh

VOLUME /ftproot
VOLUME /output

ENV RUST_BACKTRACE=full
ENTRYPOINT ["/bin/bash", "./entrypoint.sh"]
