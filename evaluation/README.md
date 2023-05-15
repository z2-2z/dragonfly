# Evaluation

## Building ProFTPD
```
git submodule update --init ./proftpd
cd ./proftpd
#TODO: patch
./configure --disable-shadow --disable-auth-pam --disable-cap
make
sudo setcap cap_sys_chroot+ep /usr/sbin/chroot
cd ..
./gen-config.sh > fuzz.conf
```

## Running
```
./proftpd/proftpd -d 10 -c $PWD/fuzz.conf -n
```
