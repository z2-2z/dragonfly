# Evaluation

To evaluate dragonfly's performance, it is compared against AFLNet
in a 24h fuzzing campaign

## Dragonfly harness
```
cd dragonfly
docker pull rust
docker pull archlinux
docker build -t evaluation-dragonfly .
```



OLD:

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
CC="$(realpath ../../AFLplusplus/afl-clang-lto)" CFLAGS="-I$LD_LIBRARY_PATH/include" LDFLAGS="-L$LD_LIBRARY_PATH -ldragonfly -Wl,-rpath=$LD_LIBRARY_PATH" ./configure --disable-shadow --disable-auth-pam --disable-cap
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

## Manual communication
```
./libdragonfly/packet_write '0:s:USER ftp\r\n' '0:s:PASS x\r\n' '0:s:CWD uploads\r\n' '0:s:EPSV\r\n' '0::STOR packetio.txt\r\n' '1::successful' '1:s:' '0:s:QUIT\r\n' "./proftpd/proftpd -d 10 -X -c $PWD/fuzz.conf -n"
```

## State
- mod_auth.c
    - logged_in
    - auth_tries (bucketed)
    - saw_first_user_cmd
    - authenticated_without_pass
    - auth_client_connected
    - auth_pass_resp_code
- mod_core.c
    - core_cmd_count (bucketed)
- mod_xfer.c
    - retr_fh (is_null)
    - stor_fh (is_null)
    - displayfilexfer_fh (is_null)
    - have_rfc2228_data
    - have_type
    - have_zmode
    - xfer_logged_sendfile_decline_msg
- mod_ls.c
    - opt_* (boolean)
    - lst_sort_by
- main.c
    - session
        - sf_flags
        - sp_flags
        - c (is_null)
        - d (is_null)
        - anon_user (is_null)
        - curr_phase ?
        - prev_server (is_null)
        - disconnect_reason

## Input
- sequence of TextTokens
    - Constant: constant that shall not be internally mutated
    - Number: A number in decimal string representation
    - Whitespace: one or more of whitespaces
    - Text: just a container for ascii bytes that can be mutated
    - Blob: like Text but also binary content allowed
- "USER ftp\r\n" => Constant("USER"), Whitespace(" "), Text("ftp"), Constant("\r\n")
- "PORT 127,0,0,1,123,234\r\n" => Constant("PORT"), Whitespace(" "), Number("127"), Text(","), Number("0"), ...
- utf-{8,16,32} support

## Mutators
- ~~number interesting mutator~~
- ~~split up~~
- AFL mutators for binary
- ~~duplicate~~
- ~~swap tokens~~
- ~~copy token to some random position~~
- ~~delete~~
- ~~dictionary insert~~
- ~~random any insert with random content~~
- crossover
    - inside tokenstream
    - between tokenstreams
    - between packets
    - between inputs
- ~~invert case~~
- all uppercase
- all lowercase
- ~~insert special chars~~
- ~~replace special chars~~
- ~~stretch out (repeat char)~~
- ~~rotate alphabet~~
- ~~random insert~~
- ~~delete content in tokens~~
- ~~scanner mutator => scan for numbers/whitespace in text~~
- ~~transform constant into text or blob~~
- replacement mutators also for subslice not entire token
- ~~more transforms between types~~

## TODO
- move certain files into memory ?
- desyscall: gethostbyname(), sleep(), usleep()
