#define __USE_GNU
#define GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>

const char elf_interpreter[] __attribute__ ((section (".interp"))) = INTERPRETER;

void desock_main (void) {
    printf ("libdragonfly.so: A helper library for stateful fuzzing\n" "\n" "This library can desock\n" "    servers = "
#ifdef DESOCK_BIND
            "yes"
#else
            "no"
#endif
            "\n" "    clients = "
#ifdef DESOCK_CONNECT
            "yes"
#else
            "no"
#endif
            "\n\n" "Compilation options:\n" "    - DEBUG = "
#ifdef DEBUG
            "yes"
#else
            "no"
#endif
            "\n" "    - MAX_CONNS = %d\n" "    - FD_TABLE_SIZE = %d\n" "    - ARCH = %s\n" "\n", 
            MAX_CONNS, FD_TABLE_SIZE, DESOCKARCH);

    exit (0);
}
