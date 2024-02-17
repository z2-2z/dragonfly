
packet_channel_bytewise: packet_channel_bytewise.c ../packet_channel.c
	clang -Werror -Wall -Wextra -Wpedantic -DDEBUG -DMAX_CONNS=2 -O0 -g -fsanitize=address,undefined -I../ -o $@ $^
