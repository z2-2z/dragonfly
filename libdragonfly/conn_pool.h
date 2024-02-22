#pragma once

#include <stddef.h>

size_t conn_pool_open (int fd);
void conn_pool_dup (int old, int new);
void conn_pool_close (int fd);
size_t conn_pool_map_fd (int fd);
