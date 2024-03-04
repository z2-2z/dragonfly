#include <stdio.h>
#include <sys/socket.h>
#include <unistd.h>

int main (void) {
    int s = socket(AF_INET, SOCK_STREAM, 0);
    bind(s, NULL, 0);
    listen(s, 0);
    
    int c = accept(s, NULL, NULL);
    char buf[512];
    
    while (1) {
        ssize_t r = read(c, buf, sizeof(buf) - 1);
        
        if (r <= 0) {
            break;
        }
        
        buf[r] = 0;
        printf("Received: %s\n", buf);
    }
}
