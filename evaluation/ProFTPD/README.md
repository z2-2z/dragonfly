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
5. netio.c:1737: When `pbuf->current` points to the beginning of the buffer, `pbuf->current - 1` points out of bounds
6. data.c:1197: Not a bug but a patch is still necessary: When aborting a LIST command, `resp_err_list` temporarily keeps pointers and accesses chunks in the freelist.
   The pointers are cleared later on in all paths through the program but it still generated a crash.
7. mod_ls.c:1194: UAF in `outputfiles()`. Every `sendline()` invocation can cause another command from the control connection to be evaluated before continuing
   with sending the LIST output due to `poll_ctrl()`. STAT and LIST both use the same global state, so if a STAT comes in while LIST output is being sent then the
   STAT commands run and clears the global state at the end and then LIST continues using the cleared state. This includes resetting values and freeing chunks.
