#define _GNU_SOURCE
#include <sys/select.h>
#include <signal.h>
#include <stdint.h>
#include <errno.h>
#include <string.h>
#include <unistd.h>

#include "desock.h"
#include "syscall.h"
#include "hooks.h"

static int filter_prioritized (int n, fd_set* set, int must_filter) {
    int ret = 0;
    fd_set result;
    FD_ZERO(&result);
    
    hook_check_connections();
    
    for (int i = 0; i < n; ++i) {
        if (set && FD_ISSET(i, set) && hook_is_next(i)) {
            FD_SET(i, &result);
            ret++;
        }
    }
    
    if (ret || must_filter) {
        memcpy(set, &result, sizeof(fd_set));
    }
    
    return ret;
}

static int internal_select (int n, fd_set* rfds, fd_set* wfds, fd_set* efds, int polling) {
    DEBUG_LOG ("[%d] desock::internal_select(%d, %p, %p, %p)", gettid (), n, rfds, wfds, efds);

    int ret = 0;
    int server_sock = -1;

    for (int i = 0; i < n; ++i) {
        if (rfds && FD_ISSET (i, rfds)) {
            if (VALID_FD (i) && fd_table[i].desock) {
                if (fd_table[i].listening) {
                    server_sock = i;
                }

                ++ret;
            } else {
                FD_CLR (i, rfds);
            }
        }

        if (wfds && FD_ISSET (i, wfds)) {
            if (VALID_FD (i) && fd_table[i].desock && !fd_table[i].listening) {
                ++ret;
            } else {
                FD_CLR (i, wfds);
            }
        }
    }

    if (efds) {
        explicit_bzero (efds, sizeof (fd_set));
    }
    
    int filtered = filter_prioritized(n, rfds, polling);
    
    if (filtered || polling) {
        ret = filtered;
        server_sock = -1;
    }

    if (server_sock > -1) {
        accept_block = 0;
        
        if (sem_trywait (&sem) == -1) {
            if (errno != EAGAIN) {
                _error ("desock::internal_select(): sem_trywait failed\n");
            }

            if (ret == 1) {
                sem_wait (&sem);
            } else {
                FD_CLR (server_sock, rfds);
                --ret;
            }
        }
    }

    DEBUG_LOG (" = %d\n", ret);
    return ret;
}

visible int select (int n, fd_set * restrict rfds, fd_set * restrict wfds, fd_set * restrict efds, struct timeval* restrict tv) {
    if (!rfds && !wfds && !efds) {
        return 0;
    } else {
        int polling = (tv && tv->tv_sec == 0 && tv->tv_usec == 0);
        return internal_select (n, rfds, wfds, efds, polling);
    }

}

visible int pselect (int n, fd_set * restrict rfds, fd_set * restrict wfds, fd_set * restrict efds, const struct timespec* restrict ts, const sigset_t * restrict mask) {
    if (!rfds && !wfds && !efds) {
        return 0;
    } else {
        int polling = (ts && ts->tv_sec == 0 && ts->tv_nsec == 0);
        return internal_select (n, rfds, wfds, efds, polling);
    }
}
