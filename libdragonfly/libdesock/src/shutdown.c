#define _GNU_SOURCE
#define __USE_GNU
#include <sys/socket.h>
#include <unistd.h>

#include "desock.h"
#include "syscall.h"
#include "hooks.h"

visible int shutdown (int fd, int how) {
    if (VALID_FD (fd) && fd_table[fd].desock) {
        DEBUG_LOG ("[%d] desock::shutdown(%d, %d) = 0\n", gettid (), fd, how);
        
        switch (how) {
            case SHUT_RD: {
                hook_shutdown_read(fd);
                break;
            }
            case SHUT_WR: {
                hook_shutdown_write(fd);
                break;
            }
            case SHUT_RDWR: {
                hook_shutdown_read(fd);
                hook_shutdown_write(fd);
                break;
            }
        }
        
        return 0;
    } else {
        return socketcall (shutdown, fd, how, 0, 0, 0, 0);
    }
}
