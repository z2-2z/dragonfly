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
```
git submodule update --init ./proftpd
cd proftpd
git apply ../patch
CC="afl-clang-lto" \
    CFLAGS="-g -O3 -flto -fsanitize=address,undefined" \
    LDFLAGS="-lcrypt -flto -fsanitize=address,undefined" \
    ./configure --disable-auth-pam --disable-cap && \
    make clean proftpd
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
docker run -v "$PWD/ftproot:/ftproot" -v "$PWD/output:/output" proftpd
```
