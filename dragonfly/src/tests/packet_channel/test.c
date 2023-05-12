#include <string.h>
#include <unistd.h>
#include <assert.h>
#include <sys/socket.h>

void connection (int fd) {
    __AFL_INIT();
    
    char buf[32];
    
    ssize_t n = read(fd, buf, sizeof(buf));
    assert(n == 5);
    assert(!memcmp(buf, "Hello", 5));
    
    n = read(fd, buf, sizeof(buf));
    assert(n == 1);
    assert(!memcmp(buf, "x", 1));
    
    n = read(fd, buf, sizeof(buf));
    assert(n == 5);
    assert(!memcmp(buf, "World", 5));
    
    assert(read(fd, buf, sizeof(buf)) == 0);
    
    close(fd);
}

int main (void) {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    bind(fd, NULL, 0);
    listen(fd, 0);
    connection(
        accept(fd, NULL, NULL)
    );
    close(fd);
}
