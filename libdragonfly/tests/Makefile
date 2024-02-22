all: packet_channel_bytewise bench_packet_decode

packet_channel_bytewise: packet_channel_bytewise.c ../packet_channel.c
	clang -Werror -Wall -Wextra -Wpedantic -DDEBUG -DMAX_CONNS=8 -O0 -g -fsanitize=address,undefined -I../ -o $@ $^

bench_packet_decode: bench_packet_decode.c ../packet_channel.c
	clang -Werror -Wall -Wextra -Wpedantic -funroll-loops -O3 -flto -DMAX_CONNS=2 -g -I../ -o $@ $^
