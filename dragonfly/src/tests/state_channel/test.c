#include <string.h>
#include <unistd.h>
#include <assert.h>
#include <sys/socket.h>

#include "dragonfly.h"

void connection (int fd) {
    char buf[32];
    ssize_t n;
    
    while ((n = read(fd, buf, sizeof(buf))) > 0) {
        dragonfly_feed_state(buf, n);
        dragonfly_push_state();
    }
    
    close(fd);
}

int main (void) {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    bind(fd, NULL, 0);
    listen(fd, 0);
    
    __AFL_INIT();
    
    connection(
        accept(fd, NULL, NULL)
    );
    close(fd);
}
