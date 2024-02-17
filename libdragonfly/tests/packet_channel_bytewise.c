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
        
        packet_channel_check_available_data();
        int next_0 = packet_channel_has_data(0);
        int next_1 = packet_channel_has_data(1);
        
        if (next_0) {
            size_t ret = packet_channel_read(0, &c, 1);
            
            if (ret == 0) {
                printf("(EOF 0) ");
                break;
            } else if (ret == 1) {
                printf("(0) %c ", c);
            } else {
                return 1;
            }
        }
        
        if (next_1) {
            size_t ret = packet_channel_read(1, &c, 1);
            
            if (ret == 0) {
                printf("(EOF 1) ");
            } else if (ret == 1) {
                printf("(1) %c ", c);
            } else {
                return 1;
            }
        }
        
        if (!next_0 && !next_1) {
            return 1;
        }
        
        printf("\n");
    }
    
    printf("\n");
    
    return 0;
}
