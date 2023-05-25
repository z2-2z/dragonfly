#include <sys/socket.h>
#include <unistd.h>
#include <assert.h>
#include <string.h>
#include <stdlib.h>

#include "dragonfly.h"

int main (void) {
    int server = socket(AF_INET, SOCK_STREAM, 0);
    assert(server != -1);
    assert(bind(server, NULL, 0) != -1);
    assert(listen(server, 0) != -1);
    
    __AFL_INIT();
    
    int conn = accept(server, NULL, NULL);
    assert(conn != -1);
    
    long sum = 0;
    
    while (1) {
        char buf[32];
        
        long state = sum % 8;
        dragonfly_feed_state(&state, sizeof(state));
        dragonfly_push_state();
        
        ssize_t n = read(conn, buf, sizeof(buf));
        assert(n != -1);
        
        if (n == 0) {
            break;
        } else if (n == 4) {
            if (memcmp(buf, "add4", 4) == 0) {
                sum += 4;
            } else if (memcmp(buf, "sub1", 4) == 0) {
                sum -= 1;
            } else if (memcmp(buf, "negs", 4) == 0) {
                sum = -sum;
            } else {
                abort();
            }
        } else {
            abort();
        }
    }
    
    write(conn, &sum, sizeof(long));
    
    close(conn);
    close(server);
    
    _Exit(0);
}
