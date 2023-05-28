# Evaluation

## Building libdragonfly
```
meson setup ./libdragonfly ../libdragonfly
cd libdragonfly
meson configure -D desock_client=true \
                -D desock_server=true \
                -D max_conns=2 \
                -D desyscall_alarm=true \
                -D desyscall_rand=true \
                -D desyscall_random=true \
                -D desyscall_resolve=true \
                -D desyscall_signals=true \
                -D desyscall_sigprocmask=true \
                -D desyscall_symbol_version=GLIBC_2.2.5
meson compile
```

## Building ProFTPD
```
git submodule update --init ./proftpd
cd ./proftpd
git apply ../patch
export LD_LIBRARY_PATH="$(realpath ../libdragonfly)"
export AFL_CC_COMPILER=LTO
CC="$(realpath ../../AFLplusplus/afl-clang-lto)" CFLAGS="" LDFLAGS="-L$LD_LIBRARY_PATH -ldragonfly -Wl,-rpath=$LD_LIBRARY_PATH" ./configure --disable-shadow --disable-auth-pam --disable-cap
make
sudo setcap cap_sys_chroot+ep ./proftpd
cd ..
./gen-config.sh > fuzz.conf
```

## Setup FTP root
```
mkdir -p /tmp/ftproot/uploads
echo content > /tmp/ftproot/file
```

## Running
```
./proftpd/proftpd -d 10 -X -c $PWD/fuzz.conf -n
```

## Notes
- move certain files into memory ?
- hook ftproot disk I/O with shared lib. discard writes => better than state reset
- does select() on connections individually
- desyscall: gethostbyname(), sleep(), usleep()
- track state
