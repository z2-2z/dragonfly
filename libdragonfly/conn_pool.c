#include <stdlib.h>
#include <stddef.h>

#ifndef MAX_CONNS
#error "MAX_CONNS has not been set"
#endif

#ifdef DEBUG
#define ABORT abort()
#else
#define ABORT __builtin_unreachable()
#endif

#define NUM_FDS 1024
#define NO_CONN ((size_t) -1LL)

static size_t connections[MAX_CONNS];
static size_t fd_map[NUM_FDS] = {NO_CONN};

static inline size_t get_next_conn(void) {
    for (size_t conn = 0; conn < MAX_CONNS; ++conn) {
        if (connections[conn] == 0) {
            return conn;
        }
    }
    
    ABORT;
}

void conn_pool_open (int fd) {
    if (fd >= NUM_FDS) {
        ABORT;
    }
    
    size_t conn = get_next_conn();
    fd_map[fd] = conn;
    connections[conn] += 1;
}

int conn_pool_close (int fd) {
    if (fd >= NUM_FDS) {
        return 0;
    }
    
    size_t conn = fd_map[fd];
    
    if (conn == NO_CONN) {
        return 0;
    }
    
    fd_map[fd] = NO_CONN;
    
    if (connections[conn] > 0) {
        connections[conn] -= 1;
    }
    
    return 1;
}

void conn_pool_dup (int old, int new) {
    if (old >= NUM_FDS || new >= NUM_FDS) {
        ABORT;
    }
    
    conn_pool_close(new);
    
    size_t conn = fd_map[old];
    
    if (conn == NO_CONN) {
        ABORT;
    }
    
    fd_map[new] = conn;
    connections[conn] += 1;
}

size_t conn_pool_map_fd (int fd) {
    if (fd >= NUM_FDS) {
        return NO_CONN;
    }
    
    size_t conn = fd_map[fd];
    
    return conn;
}

int conn_pool_has_open_connections (void) {
    for (size_t conn = 0; conn < MAX_CONNS; ++conn) {
        if (connections[conn] != 0) {
            return 1;
        }
    }
    
    return 0;
}
