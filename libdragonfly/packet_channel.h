#pragma once

#include <stddef.h>

void packet_channel_init(void* buffer);
int packet_channel_is_next(size_t conn);
size_t packet_channel_read(size_t conn, char* buf, size_t size);
