#pragma once

#include <stddef.h>

void packet_channel_init(void* buffer);
void packet_channel_check_available_data(void);
int packet_channel_has_data(size_t conn);
size_t packet_channel_read(size_t conn, char* buf, size_t size);
int packet_channel_eof(void);
