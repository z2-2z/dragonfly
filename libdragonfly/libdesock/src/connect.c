#ifdef DESOCK_CONNECT
#define _GNU_SOURCE
#include <unistd.h>
#include <sys/socket.h>
#include <errno.h>

#include "syscall.h"
#include "desock.h"
#include "hooks.h"

visible int connect (int fd, const struct sockaddr* addr, socklen_t len) {
    if (VALID_FD (fd) && DESOCK_FD (fd)) {
        DEBUG_LOG ("[%d] desock::connect(%d, %p, %d)", gettid (), fd, addr, len);
        
        if (sem_trywait (&sem) == -1) {
            if (errno != EAGAIN) {
                _error ("desock::connect(): sem_trywait failed\n");
            }

            sem_wait (&sem);
        }
        
        fd_table[fd].desock = 1;
        
        if (hook_open(fd) != fd) {
            DEBUG_LOG (" = -1\n");
            return -1;
        } else {
            DEBUG_LOG (" = 0\n");
            return 0;
        }
    } else {
        return socketcall_cp (connect, fd, addr, len, 0, 0, 0);
    }
}
#endif
