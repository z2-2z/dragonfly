#include <stdio.h>
#include <unistd.h>
#include "packet_channel.h"

char packet_buf[16 * 1024 * 1024];

int main (void) {
    if (read(0, packet_buf, sizeof(packet_buf)) < 0) {
        return 1;
    }
    
    packet_channel_init(packet_buf);
    
    while (1) {
        char c = 0;
        
        int next_0 = packet_channel_is_next(0);
        int next_1 = packet_channel_is_next(1);
        
        if (next_1) {
            size_t ret = packet_channel_read(1, &c, 1);
            
            if (ret == 0) {
                printf("(EOF 1)\n");
            } else if (ret == 1) {
                printf("(1) %c\n", c);
            } else {
                return 1;
            }
        }
        
        if (next_0) {
            size_t ret = packet_channel_read(0, &c, 1);
            
            if (ret == 0) {
                printf("(EOF 0)\n");
                break;
            } else if (ret == 1) {
                printf("(0) %c\n", c);
            } else {
                return 1;
            }
        }
        
        if (!next_0 && !next_1) {
            return 1;
        }
        
        printf("---\n");
    }
    
    return 0;
}
