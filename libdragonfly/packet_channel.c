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

typedef enum {
    TYPE_DATA = 1,
    TYPE_SEP = 2,
    TYPE_EOF = 3,
} PacketType;

typedef struct {
    PacketType type;
    uint32_t conn;
    uint64_t size;
    char content[];
} __attribute__((packed)) Packet;

typedef struct {
    uint64_t consumed;
    Packet* packet;
} ConnState;

static ConnState cursors[MAX_CONNS] = {0};
static int conn_has_data[MAX_CONNS] = {0};
static int signal_eof = 0;

static uint64_t align8 (uint64_t val) {
    uint64_t rem = val % 8;
    
    if (rem == 0) {
        return val;
    } else {
        return val + 8 - rem;
    }
}

static inline size_t packet_size (Packet* packet) {
    switch (packet->type) {
        case TYPE_SEP: {
            return sizeof(Packet);
        }
        
        case TYPE_EOF: {
            return 0;
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

static Packet* next_packet (Packet* packet) {
    return (Packet*) ((char*)packet + packet_size(packet));
}

static Packet* next_packet_for_conn (Packet* start, size_t conn) {
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

static void select_group (Packet* group_separator) {
#ifdef DEBUG
    assert(group_separator->type == TYPE_SEP);
#endif
    
    /* Reset global state */
    __builtin_memset(cursors, 0, sizeof(ConnState) * MAX_CONNS);
    
    /* Set all cursors to first packet for given connection in current group */
    Packet* cursor = group_separator;
    
    while (1) {
        cursor = next_packet(cursor);
        
        switch (cursor->type) {
            case TYPE_SEP:
            case TYPE_EOF: {
                for (int i = 0; i < MAX_CONNS; ++i) {
                    if (cursors[i].packet == NULL) {
                        cursors[i].packet = cursor;
                    }
                }
                return;
            }
            
            case TYPE_DATA: {
                size_t conn = (size_t) cursor->conn;
                
                if (conn < MAX_CONNS && cursors[conn].packet == NULL) {
                    cursors[conn].packet = cursor;
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

void packet_channel_init (void* buffer) {
    if (buffer) {
        select_group((Packet*) buffer);
    } else {
        __builtin_memset(cursors, 0, sizeof(ConnState) * MAX_CONNS);
    }
}

int packet_channel_has_data (size_t conn) {
    if (conn >= MAX_CONNS) {
        return 0;
    }
    
    return conn_has_data[conn];
}

void packet_channel_check_available_data (void) {
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
    
    if (have_data) {
        /* There is still is some data left in this group to send, select the next packet */
        Packet* value = (Packet*)(size_t)-1LL;
        int index = 0;
        
        for (int i = 0; i < MAX_CONNS; ++i) {
            printf("value %p\n", (void*) value);
            
            if (cursor_pos[i] < value) {
                value = cursor_pos[i];
                index = i;
                
            }
        }
        
        __builtin_memset(conn_has_data, 0, MAX_CONNS);
        conn_has_data[index] = 1;
        
    } else {
        /* Edge case: sent all data of the current group */
        if (signal_eof) {
            /* EOF done, signal that we can continue with next group */
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
            
            signal_eof = 0;
        } else {
            /* Signal EOF to all secondary connections */
            for (int i = 0; i < MAX_CONNS; ++i) {
                conn_has_data[i] = (i > 0);
            }
            signal_eof = 1;
        }
    }
}

size_t packet_channel_read (size_t conn, char* buf, size_t size) {
    if (conn >= MAX_CONNS || !buf || !size) {
        return 0;
    }
    
    ConnState* cursor = &cursors[conn];
    Packet* packet = cursor->packet;
    
    while (1) {
        switch (packet->type) {
            case TYPE_SEP: {
                if (signal_eof) {
                    return 0;
                } else {
                    select_group(packet);
                    packet = cursor->packet;
                    break;
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

int packet_channel_eof (void) {
    int eof = 1;
    
    for (int i = 0; i < MAX_CONNS; ++i) {
        eof &= (cursors[i].packet->type == TYPE_EOF);
    }
    
    return eof;
}
