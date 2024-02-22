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

size_t conn_pool_open (int fd) {
    if (fd >= NUM_FDS) {
        ABORT;
    }
    
    size_t conn = 0;
    
    for (; conn < MAX_CONNS; ++conn) {
        if (connections[conn] == 0) {
            break;
        }
    }
    
    if (conn >= MAX_CONNS) {
        ABORT;
    }
    
    fd_map[fd] = conn;
    connections[conn] = 1;
    return conn;
}

void conn_pool_close (int fd) {
    if (fd >= NUM_FDS) {
        ABORT;
    }
    
    size_t conn = fd_map[fd];
    
    if (conn == NO_CONN) {
        return;
    }
    
    fd_map[fd] = NO_CONN;
    
    if (connections[conn] > 0) {
        connections[conn] -= 1;
    }
}

void conn_pool_dup (int old, int new) {
    if (old >= NUM_FDS || new >= NUM_FDS) {
        ABORT;
    }
    
    conn_pool_close(new);
    
    size_t conn = fd_map[old];
    fd_map[new] = conn;
    connections[conn] += 1;
}

size_t conn_pool_map_fd (int fd) {
    if (fd >= NUM_FDS) {
        ABORT;
    }
    
    size_t conn = fd_map[fd];
    
    if (conn == NO_CONN) {
        ABORT;
    }
    
    return conn;
}
