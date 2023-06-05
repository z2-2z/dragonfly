FROM ubuntu:18.04

RUN apt-get -y update && \
    apt-get -y install \ 
        apt-utils \
        build-essential \
        openssl \
        clang \
        graphviz-dev \
        libcap-dev \
        git \
        libcap2-bin

# Build AFLNet
RUN git clone https://github.com/aflnet/aflnet.git
RUN cd aflnet && \
    export LLVM_CONFIG="llvm-config-6.0" && \
    make clean all && \
    make -C llvm_mode
ENV AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1
ENV AFL_SKIP_CPUFREQ=1

# Build ProFTPD
RUN git clone https://github.com/z2-2z/dragonfly
WORKDIR dragonfly/evaluation/proftpd
RUN git submodule update --init . && \
    git apply ../aflnet/patch
RUN CC=/aflnet/afl-clang-fast ./configure --disable-shadow --disable-auth-pam --disable-cap && make
RUN setcap cap_sys_chroot+ep ./proftpd

# Setup FTP root
RUN mkdir -p /tmp/ftproot/uploads
RUN echo content > /tmp/ftproot/file

# Configure ProFTPD
WORKDIR ../aflnet
RUN useradd -M -r user && \
    passwd -d user
RUN USER=user ../gen-config.sh > fuzz.conf

# Fuzzer setup
RUN mkdir corpus && \
    echo -en ' \r\n' > corpus/empty.raw && \
    chmod 664 corpus/empty.raw
RUN chmod 666 /dev/null
RUN mkdir -p /output

# Autostart the fuzzing campaign
VOLUME ["/output"]
RUN echo 'set -e; chmod 777 /output; su user -c "exec $*"' > /entrypoint.sh
ENTRYPOINT ["/bin/sh", "/entrypoint.sh"]
CMD ["timeout", "24h", "/aflnet/afl-fuzz", "-i", "corpus", "-o", "/output", "-m", "none", "-x", "../ftp.dict", "-N", "tcp://127.0.0.1/2121", "-P", "FTP", "-E", "-R", "--", "../proftpd/proftpd", "-d", "0", "-q", "-X", "-c", "/dragonfly/evaluation/aflnet/fuzz.conf", "-n"]
