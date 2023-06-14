LIBDRAGONFLY=../../../../libdragonfly
AFL=../../../../AFLplusplus

all: test baseline

%: %.c ./build/libdragonfly.so $(AFL)/afl-clang-lto
	$(AFL)/afl-clang-lto -o $@ -Wall -Wextra -Wpedantic -Werror $< -I$(LIBDRAGONFLY)/include -L./build -ldragonfly

./build/libdragonfly.so: $(LIBDRAGONFLY)
	meson setup ./build $(LIBDRAGONFLY)
	cd ./build && meson compile

$(AFL)/afl-clang-lto: $(AFL)
	$(MAKE) -C $(AFL) source-only

.PHONY: clean
clean:
	@rm -rfv ./build baseline test
