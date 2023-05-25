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
    
    // do no work and exit immediately
    
    close(server);
}
