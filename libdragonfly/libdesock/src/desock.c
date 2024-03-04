#define __USE_GNU
#define GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>
#include <semaphore.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <sys/un.h>

#include "syscall.h"
#include "desock.h"

const struct sockaddr_in stub_sockaddr_in = {
    .sin_family = AF_INET,
    .sin_port = 53764,
    .sin_addr.s_addr = 0x100007f
};

const struct sockaddr_in6 stub_sockaddr_in6 = {
    .sin6_family = AF_INET6,
    .sin6_port = 53764,
    .sin6_flowinfo = 0,
    .sin6_addr.s6_addr = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1},
    .sin6_scope_id = 0
};

/* Given an fd that is being desocketed fill the given sockaddress structure
   with the right sockaddr stub from above.
 */
void fill_sockaddr (int fd, struct sockaddr* addr, socklen_t * addr_len) {
    if (addr && addr_len) {
        switch (fd_table[fd].domain) {
        case AF_INET:{
                struct sockaddr_in* ptr = (struct sockaddr_in *) addr;
                ptr->sin_family = AF_INET;
                if (*addr_len >= sizeof (struct sockaddr_in)) {
                    ptr->sin_port = stub_sockaddr_in.sin_port;
                    ptr->sin_addr = stub_sockaddr_in.sin_addr;
                    *addr_len = sizeof (struct sockaddr_in);
                }
                break;
            }

        case AF_INET6:{
                *addr_len = MIN (*addr_len, sizeof (stub_sockaddr_in6));
                memcpy (addr, &stub_sockaddr_in6, *addr_len);
                break;
            }

        default:{
                _error ("desock::fill_sockaddr(): Invalid domain %d\n", fd_table[fd].domain);
            }
        }
    }
}

#ifdef DEBUG
void _debug (char* fmt_string, ...) {
    va_list args;
    va_start (args, fmt_string);
    vfprintf (stderr, fmt_string, args);
    va_end (args);
    fflush (stderr);
}
#endif

void _error (char* fmt_string, ...) {
    va_list args;
    va_start (args, fmt_string);
    vfprintf (stderr, fmt_string, args);
    va_end (args);
    
    while (1) {
        abort ();
    }
}

/* Highest file descriptor number seen so far */
int max_fd = 0;

/* Indicates whether the next call to accept() should block or not */
int accept_block = 1;

/* Table that holds metadata about desocketed file descriptors */
struct fd_entry fd_table[FD_TABLE_SIZE];

/* Semaphore for synchronization of the connection pool in multi-threaded
   applications.
 */
sem_t sem;

__attribute__ ((constructor))
void desock_init (void) {
    if (sem_init (&sem, 1, MAX_CONNS) == -1) {
        _error ("desock::error: sem_init failed\n");
    }
}

#ifdef DEBUG
visible void clear_fd_table_entry (int idx) {
    fd_table[idx].epfd = -1;
    fd_table[idx].desock = 0;
    fd_table[idx].listening = 0;
}
#endif
