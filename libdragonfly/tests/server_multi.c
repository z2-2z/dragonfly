#include <stdio.h>
#include <sys/socket.h>
#include <unistd.h>
#include <sys/select.h>

int main (void) {
    int s = socket(AF_INET, SOCK_STREAM, 0);
    bind(s, NULL, 0);
    listen(s, 0);
    
    int conns[MAX_CONNS];
    char buf[512];
    int run = 1;
    
    for (int i = 0; i < MAX_CONNS; ++i) {
        conns[i] = -1;
    }
    
    while (run) {
        /* Reconnect */
        for (int i = 0; i < MAX_CONNS; ++i) {
            if (conns[i] == -1) {
                conns[i] = accept(s, NULL, NULL);
            }
        }
        
        /* Build fd set */
        fd_set readfds;
        int nfds = 0;
        FD_ZERO(&readfds);
        
        for (int i = 0; i < MAX_CONNS; ++i) {
            if (conns[i] >= 0) {
                FD_SET(conns[i], &readfds);
                nfds = conns[i];
            }
        }
        
        if (select(nfds + 1, &readfds, NULL, NULL, NULL) <= 0) {
            break;
        }
        
        for (int i = 0; i < MAX_CONNS; ++i) {
            if (FD_ISSET(conns[i], &readfds)) {
                ssize_t r = read(conns[i], buf, sizeof(buf) - 1);
                
                if (r <= 0) {
                    close(conns[i]);
                    conns[i] = -1;
                } else {
                    buf[r] = 0;
                    printf("Received on [%d]: %s\n", conns[i], buf);
                }
            }
        }
        
        for (int i = 0; i < MAX_CONNS; ++i) {
            run &= (conns[i] == -1);
        }
        run = !run;
    }
}
