# Build the fuzzer
FROM rust:latest as fuzzer-builder
COPY fuzzer .
RUN cargo build --release

# Create the fuzzers environment
FROM archlinux:latest

RUN pacman --noconfirm -Sy git make clang llvm lld diffutils meson ninja coreutils libcap shadow

RUN git clone https://github.com/z2-2z/dragonfly
RUN cd dragonfly && \
    git submodule update --init \
        AFLplusplus \
        libdragonfly \
        evaluation/proftpd

# Build AFL++
WORKDIR dragonfly/AFLplusplus
RUN NO_PYTHON=1 NO_NYX=1 make source-only && test -f afl-clang-lto

# Build libdragonfly
WORKDIR ../libdragonfly
RUN meson setup ./build && \
    cd build && \
    meson configure -D desock_client=true \
        -D desock_server=true \
        -D max_conns=2 \
        -D desyscall_alarm=true \
        -D desyscall_rand=true \
        -D desyscall_random=true \
        -D desyscall_resolve=true \
        -D desyscall_signals=true \
        -D desyscall_sigprocmask=true \
        -D desyscall_symbol_version=GLIBC_2.2.5 && \
    meson compile

# Build ProFTPD
WORKDIR ../evaluation/proftpd
RUN git apply ../dragonfly/patch
RUN export LIBDRAGONFLY="$(realpath ../../libdragonfly/build)" && \
    CC="$(realpath ../../AFLplusplus/afl-clang-lto)" \
    CFLAGS="-I$LIBDRAGONFLY/include -O3" \
    LDFLAGS="-L$LIBDRAGONFLY -ldragonfly -Wl,-rpath=$LIBDRAGONFLY" \
    ./configure --disable-shadow --disable-auth-pam --disable-cap && \
    AFL_CC_COMPILER=LTO make
RUN setcap cap_sys_chroot+ep ./proftpd

# Setup FTP root
RUN mkdir -p /tmp/ftproot/uploads
RUN echo content > /tmp/ftproot/file

# Get the fuzzer
WORKDIR ../dragonfly
COPY --from=fuzzer-builder /target/release/fuzzer_dragonfly .

# Configure ProFTPD
RUN useradd -M -r user && \
    passwd -d user
RUN chmod 777 .
USER user
RUN USER=user ../gen-config.sh > fuzz.conf
