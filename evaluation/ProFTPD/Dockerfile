FROM archlinux:latest

RUN pacman -Sy --noconfirm libidn
RUN userdel -f ftp && useradd -M -r ftp && passwd -d ftp
RUN useradd -M -r user && (echo user; echo user) | passwd user

WORKDIR /proftpd
COPY evaluation/ProFTPD/config ./config
COPY evaluation/ProFTPD/proftpd/proftpd ./proftpd
COPY evaluation/ProFTPD/ftp.dict ./ftp.dict
COPY libdragonfly/build/libdragonfly.so ./libdragonfly.so
COPY evaluation/ProFTPD/fuzzer/target/release/fuzzer ./fuzzer
COPY dragonfly/target/debug/packets ./packets

VOLUME /ftproot
VOLUME /output
