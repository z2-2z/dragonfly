# Evaluation

## Building ProFTPD
```
git submodule update --init ./proftpd
cd ./proftpd
#TODO: patch
./configure --disable-shadow --disable-auth-pam --disable-cap
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
./proftpd/proftpd -d 10 -c $PWD/fuzz.conf -n
```
