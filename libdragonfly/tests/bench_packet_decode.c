#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <time.h>
#include "packet_channel.h"

char packet_buf[16 * 1024 * 1024];


struct timespec diff_timespec(const struct timespec *time0, const struct timespec *time1) {
    struct timespec diff = {
        .tv_sec = time1->tv_sec - time0->tv_sec,
        .tv_nsec = time1->tv_nsec - time0->tv_nsec,
    };
    
    if (diff.tv_nsec < 0) {
        diff.tv_nsec += 1000000000;
        diff.tv_sec--;
    }
    
    return diff;
}

size_t run_one (char* packet_buf, size_t amount) {
    char buf[amount];
    size_t total_bytes = 0;
    
    packet_channel_init(packet_buf);
    
    while (!packet_channel_eof()) {
        size_t num_read = 0;
        
        packet_channel_check_available_data();
        for (size_t i = 0; i < MAX_CONNS; ++i) {
            if (packet_channel_has_data(i)) {
                num_read += packet_channel_read(i, buf, amount);
            }
        }
        
        total_bytes += num_read;
    }
    
    return total_bytes;
}

int main (int argc, char** argv) {
    size_t amount = 1;
    size_t total_bytes = 0;
    struct timespec start, end;
    
    if (argc > 1) {
        amount = atoi(argv[1]);
    }
    
    if (read(0, packet_buf, sizeof(packet_buf)) < 0) {
        return 1;
    }
    
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    while (total_bytes < 1 * 1024 * 1024 * 1024) {
        total_bytes += run_one(packet_buf, amount);
    }
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    struct timespec diff = diff_timespec(&start, &end);
    
    printf("Time to 1 GiB: %lds + %ldns\n", diff.tv_sec, diff.tv_nsec);
    
    return 0;
}
