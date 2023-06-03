FROM archlinux:latest

RUN pacman --noconfirm -Sy git

RUN git clone https://github.com/z2-2z/dragonfly
RUN cd dragonfly && \
    git submodule update --init \
        AFLplusplus \
        libdragonfly \
        evaluation/proftpd

# Build AFL++
RUN pacman --noconfirm -S make clang llvm lld diffutils
WORKDIR dragonfly/AFLplusplus
RUN NO_PYTHON=1 NO_NYX=1 make source-only && test -f afl-clang-lto

# Build libdragonfly
RUN pacman --noconfirm -S meson ninja
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
RUN pacman --noconfirm -S coreutils libcap
WORKDIR ../evaluation/proftpd
RUN git apply ../dragonfly/patch
RUN export LIBDRAGONFLY="$(realpath ../../libdragonfly/build)" && \
    AFL_CC_COMPILER=LTO \
    CC="$(realpath ../../AFLplusplus/afl-clang-lto)" \
    CFLAGS="-I$LIBDRAGONFLY/include -O3" \
    LDFLAGS="-L$LIBDRAGONFLY -ldragonfly -Wl,-rpath=$LIBDRAGONFLY" \
    ./configure --disable-shadow --disable-auth-pam --disable-cap && \
    make
RUN setcap cap_sys_chroot+ep ./proftpd

# Configure ProFTPD
