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
meson configure -D max_conns=2 -D fd_table_size=16 -D desock_client=true
meson compile
```

Build ProFTPD:
```sh
git submodule update --init ./proftpd
cd proftpd
git apply ../patches/patch
git apply ../patches/pool.patch
# Optionally apply all ../patches/bug-*.patch files in the right order
CC="afl-clang-lto" \
    CFLAGS="-g -Ofast -march=native -flto -fsanitize=address" \
    LDFLAGS="-lcrypt -flto -fsanitize=address" \
    ./configure --disable-auth-pam --disable-cap && \
    make clean proftpd
mv proftpd proftpd-fuzzing

CC="clang" \
    CFLAGS="-fno-pie -g -O0 -fno-omit-frame-pointer -fsanitize=address" \
    LDFLAGS="-no-pie -fsanitize=address" \
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
docker run --rm -d -v "$PWD/ftproot:/ftproot" -v "$PWD/output:/output" proftpd
```

## Findings
1. mod_auth.c:2898: `dir_canonical_path()` may return NULL on invalid paths
2. mod_ls.c:493: `pstrndup()` returns NULL because `p` is NULL
3. mod_ls.c:1105: `tail` may be NULL
4. data.c:393: `session.d` belongs to the `init_conn pool` and gets used in `pr_inet_set_nonblock` after the pool was destroyed by `pr_inet_close`
