# Evaluation

## Building ProFTPD
```
git submodule update --init ./proftpd
cd ./proftpd
#TODO: apply patch
export LD_LIBRARY_PATH="$(realpath ../../libdragonfly/build)"
CC=clang LDFLAGS="-L$LD_LIBRARY_PATH -ldesyscall -Wl,-rpath=$LD_LIBRARY_PATH" ./configure --disable-shadow --disable-auth-pam --disable-cap
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
- remove /dev/urandom
- remove host reverse lookup
- move certain files into memory ?
- hook ftproot disk I/O with shared lib. discard writes => better than state reset
- does select() on connections individually
