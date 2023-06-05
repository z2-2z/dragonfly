
#ifndef __ftp_generator_GENERATOR_H
#define __ftp_generator_GENERATOR_H

#include <stddef.h>

size_t ftp_generator_generate(unsigned char* buf, size_t len);
void ftp_generator_seed(size_t initial_seed);

#endif /* __ftp_generator_GENERATOR_H */
