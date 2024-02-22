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
        
        for (size_t i = 0; i < MAX_CONNS; ++i) {
            if (packet_channel_has_data(i)) {
                size_t ret = packet_channel_read(i, buf, amount);
            
                if (ret == 0) {
                    printf("(EOF %lu) ", i);
                } else {
                    buf[ret] = 0;
                    printf("(%lu) \"%s\" ", i, buf);
                }
            }
        }
        
        printf("\n");
    }
    
    return 0;
}
