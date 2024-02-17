#include <stdlib.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <assert.h>
#include <string.h>

#ifndef MAX_CONNS
#error "MAX_CONNS not set"
#endif

#define min(a,b)             \
({                           \
    __typeof__ (a) _a = (a); \
    __typeof__ (b) _b = (b); \
    _a < _b ? _a : _b;       \
})

#define TYPE_DATA 1
#define TYPE_SEP  2
#define TYPE_EOF  3

typedef struct {
    uint32_t type;
    uint32_t conn;
    uint64_t size;
    char content[];
} __attribute__((packed)) Packet;

typedef struct {
    uint64_t consumed;
    Packet* packet;
} ConnState;

static ConnState cursors[MAX_CONNS] = {0};

static uint64_t align8 (uint64_t val) {
    uint64_t rem = val % 8;
    
    if (rem == 0) {
        return val;
    } else {
        return val + 8 - rem;
    }
}

static size_t packet_size(Packet* packet) {
    switch (packet->type) {
        case TYPE_SEP:
        case TYPE_EOF: {
            return sizeof(Packet);
        }
        
        case TYPE_DATA: {
            return sizeof(Packet) + align8(packet->size);
        }
        
        default: {
#ifdef DEBUG
            abort();
#else
            __builtin_unreachable();
#endif
        }
    }
}

static Packet* next_packet(Packet* packet) {
    if (packet->type == TYPE_EOF) {
        return packet;
    }
    
    return (Packet*) ((char*)packet + packet_size(packet));
}

static Packet* next_packet_for_conn(Packet* start, size_t conn) {
    Packet* cursor = start;
    
    while (1) {
        cursor = next_packet(cursor);
        
        switch (cursor->type) {
            case TYPE_SEP:
            case TYPE_EOF: {
                return cursor;
            }
            
            case TYPE_DATA: {
                if (cursor->conn == conn) {
                    return cursor;
                }
                
                break;
            }
            
            default: {
#ifdef DEBUG
                abort();
#else
                __builtin_unreachable();
#endif
            }
        }
    }
}

static void select_group(Packet* group_separator) {
#ifdef DEBUG
    assert(group_separator->type == TYPE_SEP);
#endif
    
    /* Reset global state */
    __builtin_memset(cursors, 0, sizeof(ConnState) * MAX_CONNS);
    
    /* Set all cursors to first packet for given connection in current group */
    char cursor_set[MAX_CONNS] = {0};
    Packet* cursor = group_separator;
    
    while (1) {
        cursor = next_packet(cursor);
        
        switch (cursor->type) {
            case TYPE_SEP:
            case TYPE_EOF: {
                for (int i = 0; i < MAX_CONNS; ++i) {
                    if (!cursor_set[i]) {
                        cursors[i].packet = cursor;
                    }
                }
                return;
            }
            
            case TYPE_DATA: {
                size_t conn = (size_t) cursor->conn;
                
                if (conn < MAX_CONNS && !cursor_set[conn]) {
                    cursors[conn].packet = cursor;
                    cursor_set[conn] = 1;
                }
                
                break;
            }
            
            default: {
#ifdef DEBUG
                abort();
#else
                __builtin_unreachable();
#endif
            }
        }
    }
}

void packet_channel_init(void* buffer) {
    if (buffer) {
        select_group((Packet*) buffer);
    } else {
        __builtin_memset(cursors, 0, sizeof(ConnState) * MAX_CONNS);
    }
}

int packet_channel_is_next(size_t conn) {
    if (conn >= MAX_CONNS) {
        return 0;
    }
    
    /* Collect all cursor positions */
    Packet* cursor_pos[MAX_CONNS] = {NULL};
    
    for (int i = 0; i < MAX_CONNS; ++i) {
        Packet* packet = cursors[i].packet;
        
        if (packet->type == TYPE_DATA && cursors[i].consumed >= packet->size) {
            packet = next_packet_for_conn(packet, (size_t) packet->conn);
        }
        
        cursor_pos[i] = packet;
    }
    
    /* Given connection contains the next packet iff its packet pointer is smaller than every other pointer */
    Packet* pointer = cursor_pos[conn];
    
    if (conn > 0 && pointer->type != TYPE_DATA) {
        return 1;
    }
    
    for (size_t i = 0; i < MAX_CONNS; ++i) {
        if (i != conn && cursor_pos[i] <= pointer) {
            return 0;
        }
    }
    
    return 1;
}

size_t packet_channel_read(size_t conn, char* buf, size_t size) {
    if (conn >= MAX_CONNS || !buf || !size) {
        return 0;
    }
    
    ConnState* cursor = &cursors[conn];
    Packet* packet = cursor->packet;
    
    while (1) {
        switch (packet->type) {
            case TYPE_SEP: {
                if (conn == 0) {
                    select_group(packet);
                    packet = cursor->packet;
                } else {
                    return 0;
                }
                
                break;
            }
            
            case TYPE_EOF: {
                return 0;
            }
            
            case TYPE_DATA: {
                if (cursor->consumed >= packet->size) {
                    packet = next_packet_for_conn(packet, conn);
                    
                    cursor->consumed = 0;
                    cursor->packet = packet;
                    
                    if (packet->type != TYPE_DATA) {
                        continue;
                    }
                }
                
                uint64_t rem_bytes = packet->size - cursor->consumed;
                uint64_t final_size = min(size, rem_bytes);
                
                memcpy(buf, (void*) &packet->content[cursor->consumed], final_size);
                cursor->consumed += final_size;
                return final_size;
            }
            
            default: {
    #ifdef DEBUG
                abort();
    #else
                __builtin_unreachable();
    #endif
            }
        }
    }
}
