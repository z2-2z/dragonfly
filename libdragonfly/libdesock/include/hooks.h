#ifndef HOOKS_H
#define HOOKS_H

#include <stdlib.h>

ssize_t hook_input (int, char*, size_t);
ssize_t hook_output (int, char*, size_t);
void hook_shutdown_write(int);
void hook_shutdown_read(int);
int hook_open(int);
int hook_dup (int, int);
int hook_is_next (int);
void hook_check_connections (void);

#endif