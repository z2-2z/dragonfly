#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <stdint.h>
#include <assert.h>
#include <errno.h>
#include <sys/shm.h>

#include "desock.h"
#include "hooks.h"
#include "syscall.h"

#include "conn_pool.h"
#include "packet_channel.h"

static int active_channel = 0;
static void* packet_channel = NULL;

#ifdef DEBUG
static unsigned char packet_buf[16 * 1024 * 1024];
#endif

__attribute__((constructor))
static void attach_packet_channel (void) {
    char* shm_id = getenv("__LIBDRAGONFLY_PACKET_CHANNEL");
    
    if (shm_id) {
        char* endptr = NULL;
        unsigned long id = strtoul(shm_id, &endptr, 0);
        
        if (endptr == NULL || *endptr != 0) {
            _error("Invalid shm id in __LIBDRAGONFLY_PACKET_CHANNEL: %s\n", shm_id);
        }
        
        packet_channel = shmat(id, NULL, 0);
        
        if (packet_channel == NULL || packet_channel == (void*) -1) {
            _error("Could not attach to shared memory: %ld\n", id);
        }
        
        DEBUG_LOG("Attached to packet channel %lu @ %p\n", id, packet_channel);
    }
#ifdef DEBUG
    else {
        syscall_cp (SYS_read, 0, packet_buf, sizeof(packet_buf));
        packet_channel = (void*) packet_buf;
        DEBUG_LOG("Read packets from stdin\n");
    }
#endif
}

void hook_shutdown_write (int fd) {
    (void) fd;
}

void hook_shutdown_read (int fd) {
    conn_pool_close(fd);
    
    if (active_channel) {
        active_channel = conn_pool_has_open_connections();
    }
}

int hook_open (int fd) {
    conn_pool_open(fd);
    
    if (!active_channel && packet_channel) {
        packet_channel_init(packet_channel);
        active_channel = 1;
    }
    
    return fd;
}

ssize_t hook_input (int fd, char* buf, size_t size) {
    if (active_channel) {
        size_t conn = conn_pool_map_fd(fd);
        
        if (conn < MAX_CONNS) {
            size_t ret = packet_channel_read(conn, buf, size);
#ifdef DEBUG
            fprintf(stderr, "\n< ");
            fwrite(buf, 1, ret, stderr);
            fprintf(stderr, "\n");
#endif
            return ret;
        }
    }
    
    errno = EBADF;
    return -1;
}

ssize_t hook_output (int fd, char* buf, size_t size) {
    (void) fd;
    (void) buf;
#ifdef DEBUG
    fprintf(stderr, "\n> ");
    fwrite(buf, 1, size, stderr);
    fprintf(stderr, "\n");
#endif
    return (ssize_t) size;
}

int hook_dup (int old, int new) {
    conn_pool_dup(old, new);
    return new;
}

void hook_check_connections (void) {
    if (active_channel) {
        packet_channel_check_available_data();
    }
}

int hook_is_next (int fd) {
    if (active_channel) {
        size_t conn = conn_pool_map_fd(fd);
        
        if (conn < MAX_CONNS) {
            return packet_channel_has_data(conn);
        } else {
            return 0;
        }
    }
    
    return 1;
}
