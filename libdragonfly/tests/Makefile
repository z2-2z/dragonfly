all: packet_channel_bytewise bench_packet_decode check_perf server_single server_multi test_dup read_shm

packet_channel_bytewise: packet_channel_bytewise.c ../packet_channel.c
	clang -Werror -Wall -Wextra -Wpedantic -DDEBUG -DMAX_CONNS=8 -O0 -g -fsanitize=address,undefined -I../ -o $@ $^

bench_packet_decode: bench_packet_decode.c ../packet_channel.c
	clang -Werror -Wall -Wextra -Wpedantic -funroll-loops -O3 -flto -DMAX_CONNS=2 -g -I../ -o $@ $^

check_perf: bench_packet_decode.c ../packet_channel.c
	clang -Werror -Wall -Wextra -Wpedantic -fno-inline -Og -DMAX_CONNS=2 -g -I../ -o $@ $^

server_single: server_single.c
	clang -Werror -Wall -Wextra -Wpedantic -DMAX_CONNS=2 -o $@ $^

server_multi: server_multi.c
	clang -Werror -Wall -Wextra -Wpedantic -DMAX_CONNS=2 -o $@ $^

test_dup: test_dup.c
	clang -Werror -Wall -Wextra -Wpedantic -DMAX_CONNS=2 -o $@ $^

read_shm: read_shm.c
	$(AFL_PATH)/afl-clang-lto -Werror -Wall -Wextra -Wpedantic -DMAX_CONNS=2 -o $@ $^
