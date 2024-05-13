project('libdragonfly', 'c', license: 'MIT', version: '1.1')

if get_option('interpreter') == ''
    r = run_command('sh', '-c', 'readelf -l /bin/ls | grep "program interpreter" | cut -d: -f2  | sed -e "s/^.//" -e "s/.$//"', check: true)
    interpreter = r.stdout().strip()
else
    interpreter = get_option('interpreter')
endif

libdesock_args = [
    '-Ofast',
    '-march=native',
    '-fno-stack-protector',
    '-fomit-frame-pointer',
    '-fvisibility=hidden',
    '-Wno-stringop-truncation',
    '-Wno-unknown-warning-option',
    '-D DESOCKARCH="@0@"'.format(get_option('arch')),
    '-D MAX_CONNS=@0@'.format(get_option('max_conns')),
    '-D FD_TABLE_SIZE=@0@'.format(get_option('fd_table_size')),
    '-D INTERPRETER="@0@"'.format(interpreter),
]

if get_option('debug_desock')
    libdesock_args += '-D DEBUG'
endif
    
if get_option('desock_client')
    libdesock_args += '-D DESOCK_CONNECT'
endif
    
if get_option('desock_server')
    libdesock_args += '-D DESOCK_BIND'
endif

libdesock_sources = [
    'libdesock/src/accept.c',
    'libdesock/src/bind.c',
    'libdesock/src/close.c',
    'libdesock/src/connect.c',
    'libdesock/src/desock.c',
    'libdesock/src/dup.c',
    'libdesock/src/epoll.c',
    'libdesock/src/getpeername.c',
    'libdesock/src/getsockname.c',
    'libdesock/src/listen.c',
    'libdesock/src/peekbuffer.c',
    'libdesock/src/poll.c',
    'libdesock/src/read.c',
    'libdesock/src/select.c',
    'libdesock/src/sendfile.c',
    'libdesock/src/shutdown.c',
    'libdesock/src/socket.c',
    'libdesock/src/sockopt.c',
    'libdesock/src/syscall.c',
    'libdesock/src/write.c',
    'libdesock/src/hooks.c',
    'libdesock/src/main.c',
    'packet_channel.c',
    'conn_pool.c',
]

libdesock_includes = [
    include_directories('libdesock/include'),
    include_directories('libdesock/include/arch/' + get_option('arch')),
    include_directories('.'),
]

shared_library('dragonfly', 
    sources : libdesock_sources,
    include_directories : libdesock_includes,
    c_args : libdesock_args,
    link_args : ['-Wl,-e,desock_main'],
    install : false,
    dependencies : [
        dependency('threads'),
    ]
)
