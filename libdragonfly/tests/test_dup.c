#include <stdio.h>
#include <sys/socket.h>
#include <unistd.h>

int main (void) {
    int s = socket(AF_INET, SOCK_STREAM, 0);
    bind(s, NULL, 0);
    listen(s, 0);
    
    int c = accept(s, NULL, NULL);
    char buf[512];
    
    dup2(c, 0);
    dup2(c, 1);
    close(c);
    
    while (1) {
        fd_set readfds;
        fd_set writefds;
        
        FD_ZERO(&readfds);
        FD_SET(0, &readfds);
        FD_ZERO(&writefds);
        FD_SET(1, &writefds);
        
        if (select(2, &readfds, &writefds, NULL, NULL) < 0) {
            return 1;
        }
        
        if (FD_ISSET(0, &readfds)) {
            ssize_t r = read(0, buf, sizeof(buf) - 1);
        
            if (r <= 0) {
                break;
            }
            
            buf[r] = 0;
            fprintf(stderr, "Received: %s\n", buf);
        }
        
        if (FD_ISSET(1, &writefds)) {
            fprintf(stderr, "1 writable\n");
        }
    }
}
