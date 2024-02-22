#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include "packet_channel.h"

char packet_buf[16 * 1024 * 1024];

int main (int argc, char** argv) {
    size_t amount = 1;
    
    if (argc > 1) {
        amount = atoi(argv[1]);
    }
    
    if (read(0, packet_buf, sizeof(packet_buf)) < 0) {
        return 1;
    }
    
    packet_channel_init(packet_buf);
    
    while (!packet_channel_eof()) {
        char buf[amount + 1];
        
        packet_channel_check_available_data();
        int next_0 = packet_channel_has_data(0);
        int next_1 = packet_channel_has_data(1);
        
        if (next_0) {
            size_t ret = packet_channel_read(0, buf, amount);
            
            if (ret == 0) {
                printf("(EOF 0) ");
            } else {
                buf[ret] = 0;
                printf("(0) \"%s\" ", buf);
            }
        }
        
        if (next_1) {
            size_t ret = packet_channel_read(1, buf, amount);
            
            if (ret == 0) {
                printf("(EOF 1) ");
            } else {
                buf[ret] = 0;
                printf("(1) \"%s\" ", buf);
            }
        }
        
        if (!next_0 && !next_1) {
            return 1;
        }
        
        printf("\n");
    }
    
    return 0;
}
