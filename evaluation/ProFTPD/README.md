# Fuzzing ProFTPD

## Setup
Build dragonfly:
```
cd dragonfly
cargo build
```

Build libdragonfly:
```sh
cd libdragonfly
meson setup ./build
cd build
meson configure -D max_conns=2 -D fd_table_size=16
meson compile
```

Build ProFTPD:
```sh
git submodule update --init ./proftpd
cd proftpd
git apply ../patch
# Optionally apply all bug-*.patch files
CC="afl-clang-lto" \
    CFLAGS="-g -O3 -flto" \
    LDFLAGS="-lcrypt -flto" \
    ./configure --disable-auth-pam --disable-cap && \
    make clean proftpd
mv proftpd proftpd-fuzzing

CC="gcc" \
    CFLAGS="-fno-pie -g -O0 -fno-omit-frame-pointer" \
    LDFLAGS="-no-pie" \
    ./configure --disable-auth-pam --disable-cap && \
    make clean proftpd
mv proftpd proftpd-debug
```

Build the fuzzer:
```
cd fuzzer
cargo build --release
```

Build the image:
```
docker pull archlinux
docker build -t proftpd -f Dockerfile ../..
```

## Running a 24h campaign
```
docker run -d -v "$PWD/ftproot:/ftproot" -v "$PWD/output:/output" proftpd
```

## Findings
1. mod_auth.c:2898: `dir_canonical_path()` may return NULL on invalid paths
