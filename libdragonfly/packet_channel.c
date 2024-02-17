#include <stdio.h>
#include <stdlib.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <assert.h>
#include <string.h>

#ifndef MAX_CONNS
#error "MAX_CONNS not set"
#endif

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

static int conn_has_data[MAX_CONNS] = {0};
static int next_group = 0;

int packet_channel_has_data(size_t conn) {
    if (conn >= MAX_CONNS) {
        return 0;
    }
    
    return conn_has_data[conn];
}

void packet_channel_check_available_data(void) {
    /* Collect all cursor positions inside the current group */
    Packet* cursor_pos[MAX_CONNS] = {NULL};
    
    for (int i = 0; i < MAX_CONNS; ++i) {
        Packet* packet = cursors[i].packet;
        
        if (packet->type == TYPE_DATA && cursors[i].consumed >= packet->size) {
            packet = next_packet_for_conn(packet, (size_t) packet->conn);
        }
        
        cursor_pos[i] = packet;
    }
    
    /* Check if connections have data left */
    int have_data = 0;
    
    for (int i = 0; i < MAX_CONNS; ++i) {
        int is_data = (cursor_pos[i]->type == TYPE_DATA);
        conn_has_data[i] = is_data;
        have_data |= is_data;
    }
    
    /* Edge case: sent all data of the current group */
    if (!have_data) {
        if (next_group) {
            /* Signal that we can continue with next group */
            Packet* pointer = cursor_pos[0];
#ifdef DEBUG
            for (int i = 0; i < MAX_CONNS; ++i) {
                assert(cursor_pos[i] == pointer);
            }
#endif
            pointer = next_packet(pointer);
            
            if (pointer->type == TYPE_DATA) {
#ifdef DEBUG
                assert(pointer->conn < MAX_CONNS);
#endif
                conn_has_data[pointer->conn] = 1;
            } else {
                conn_has_data[0] = 1;
            }
            
            next_group = 0;
        } else {
            /* Signal EOF to all secondary connections */
            for (int i = 1; i < MAX_CONNS; ++i) {
                conn_has_data[i] = 1;
            }
            next_group = 1;
        }
    }
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
                if (!next_group) {
                    select_group(packet);
                    packet = cursor->packet;
                    break;
                } else {
                    return 0;
                }
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
                uint64_t final_size = (size < rem_bytes) ? size : rem_bytes;
                
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
