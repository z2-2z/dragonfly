LIBDRAGONFLY=../../../../libdragonfly
AFL=../../../../AFLplusplus

test: test.c ./build/libdragonfly.so $(AFL)/afl-clang-lto
	$(AFL)/afl-clang-lto -o $@ -Wall -Wextra -Wpedantic -Werror test.c -I$(LIBDRAGONFLY)/include -L./build -ldragonfly

./build/libdragonfly.so: $(LIBDRAGONFLY)
	meson setup ./build $(LIBDRAGONFLY)
	meson configure ./build -Ddebug_desock=true
	cd ./build && meson compile
	
$(AFL)/afl-clang-lto: $(AFL)
	$(MAKE) -C $(AFL) source-only
