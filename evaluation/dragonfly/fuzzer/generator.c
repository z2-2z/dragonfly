
/************************************
     Auto-generated by Chameleon

              Parameters
             ~~~~~~~~~~~~
  Grammar: grammar.chm
  Forbid cycles: false
  Global endianness: native
  Global scheduling: random
  Depth: unlimited
 ************************************/
 
#include <stddef.h>
#include <stdint.h>
#include <endian.h>

#define UNLIKELY(x) __builtin_expect(!!(x), 0)
#define LIKELY(x) __builtin_expect(!!(x), 1)

#ifndef __clang__
#define __builtin_memcpy_inline __builtin_memcpy
#endif

// Mark globals as thread local only if we are doing multithreading
#ifdef MULTITHREADING
#define THREAD_LOCAL __thread
#else
#define THREAD_LOCAL
#endif

// Define the compile-time seed
#ifndef SEED
#define SEED 0x35c6be9ba2548264
#endif

// Define endianness helper functions
#define LITTLE_ENDIAN_16(x) htole16((uint16_t) (x))
#define BIG_ENDIAN_16(x)    htobe16((uint16_t) (x))
#define LITTLE_ENDIAN_32(x) htole32((uint32_t) (x))
#define BIG_ENDIAN_32(x)    htobe32((uint32_t) (x))
#define LITTLE_ENDIAN_64(x) htole64((uint64_t) (x))
#define BIG_ENDIAN_64(x)    htobe64((uint64_t) (x))

// RNG: xorshift64
static THREAD_LOCAL uint64_t rand_state = SEED;

#ifndef DISABLE_rand
static uint64_t rand() {
    uint64_t x = rand_state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    return rand_state = x;
}
#else
uint64_t rand();
#endif

#ifndef DISABLE_seed
void ftp_generator_seed(size_t s) {
    if (s) {
        rand_state = (uint64_t) s;
    } else {
        rand_state = SEED;
    }
}
#else
void ftp_generator_seed(size_t);
#endif

// Helper method that writes random data into a buffer
#define MASK_BYTES 0xFFFFFFFFFFFFFFFFUL
#define MASK_STRING 0x7F7F7F7F7F7F7F7FUL
#ifndef DISABLE_random_buffer
static void random_buffer (unsigned char* buf, uint32_t len, uint64_t mask) {
    while (len >= 8) {
        *(uint64_t*)buf = rand() & mask;
        buf += 8; len -= 8;
    }
    
    while (len >= 4) {
        *(uint32_t*)buf = (uint32_t) (rand() & mask);
        buf += 4; len -= 4;
    }
    
    while (len >= 2) {
        *(uint16_t*)buf = (uint16_t) (rand() & mask);
        buf += 2; len -= 2;
    }
    
    while (len >= 1) {
        *buf = (unsigned char) (rand() & mask);
        buf += 1; len -= 1;
    }
}
#else
void random_buffer (unsigned char* buf, uint32_t len, uint64_t mask);
#endif

// Strings from grammar
static const unsigned char string_357634223873241833[10] = {0x4c, 0x49, 0x53, 0x54, 0x20, 0x61, 0x62, 0x63, 0xd, 0xa};
static const unsigned char string_452836965304285521[10] = {0x44, 0x45, 0x4c, 0x45, 0x20, 0x61, 0x62, 0x63, 0xd, 0xa};
static const unsigned char string_553001306382777075[15] = {0x41, 0x4c, 0x4c, 0x4f, 0x20, 0x34, 0x30, 0x39, 0x36, 0x20, 0x52, 0x20, 0x31, 0xd, 0xa};
static const unsigned char string_1322821659644260744[10] = {0x53, 0x54, 0x41, 0x54, 0x20, 0x61, 0x62, 0x63, 0xd, 0xa};
static const unsigned char string_1787227683893679301[10] = {0x52, 0x4e, 0x54, 0x4f, 0x20, 0x78, 0x79, 0x7a, 0xd, 0xa};
static const unsigned char string_3137864909032443780[10] = {0x54, 0x59, 0x50, 0x45, 0x20, 0x41, 0x20, 0x4e, 0xd, 0xa};
static const unsigned char string_3407260272413593858[14] = {0x53, 0x4d, 0x4e, 0x54, 0x20, 0x75, 0x70, 0x6c, 0x6f, 0x61, 0x64, 0x73, 0xd, 0xa};
static const unsigned char string_3888699756855861392[9] = {0x52, 0x4d, 0x44, 0x20, 0x61, 0x62, 0x63, 0xd, 0xa};
static const unsigned char string_4041482461632183779[10] = {0x55, 0x53, 0x45, 0x52, 0x20, 0x66, 0x74, 0x70, 0xd, 0xa};
static const unsigned char string_4892258942051522414[11] = {0x52, 0x45, 0x54, 0x52, 0x20, 0x66, 0x69, 0x6c, 0x65, 0xd, 0xa};
static const unsigned char string_4918681801202486344[9] = {0x4d, 0x4b, 0x44, 0x20, 0x61, 0x62, 0x63, 0xd, 0xa};
static const unsigned char string_6117162860463527439[6] = {0x50, 0x41, 0x53, 0x53, 0xd, 0xa};
static const unsigned char string_6139412818626637881[10] = {0x48, 0x45, 0x4c, 0x50, 0x20, 0x3f, 0x3f, 0x3f, 0xd, 0xa};
static const unsigned char string_6140873944760370179[10] = {0x41, 0x50, 0x50, 0x45, 0x20, 0x61, 0x62, 0x63, 0xd, 0xa};
static const unsigned char string_6470792744394151105[13] = {0x43, 0x57, 0x44, 0x20, 0x75, 0x70, 0x6c, 0x6f, 0x61, 0x64, 0x73, 0xd, 0xa};
static const unsigned char string_6810841031584417163[6] = {0x50, 0x41, 0x53, 0x56, 0xd, 0xa};
static const unsigned char string_7115538435105230547[6] = {0x43, 0x44, 0x55, 0x50, 0xd, 0xa};
static const unsigned char string_7750727661420411684[6] = {0x53, 0x59, 0x53, 0x54, 0xd, 0xa};
static const unsigned char string_8853320453514720409[10] = {0x52, 0x4e, 0x46, 0x52, 0x20, 0x61, 0x62, 0x63, 0xd, 0xa};
static const unsigned char string_9866723023257838052[13] = {0x52, 0x45, 0x53, 0x54, 0x20, 0x6d, 0x61, 0x72, 0x6b, 0x65, 0x72, 0xd, 0xa};
static const unsigned char string_10273663291660329175[10] = {0x53, 0x54, 0x4f, 0x52, 0x20, 0x61, 0x62, 0x63, 0xd, 0xa};
static const unsigned char string_10882850770906693452[6] = {0x4e, 0x4f, 0x4f, 0x50, 0xd, 0xa};
static const unsigned char string_11229452647410007184[8] = {0x53, 0x54, 0x52, 0x55, 0x20, 0x46, 0xd, 0xa};
static const unsigned char string_11337447964492719760[6] = {0x52, 0x45, 0x49, 0x4e, 0xd, 0xa};
static const unsigned char string_13522589807571429534[6] = {0x53, 0x54, 0x4f, 0x55, 0xd, 0xa};
static const unsigned char string_14143458678229299797[6] = {0x41, 0x42, 0x4f, 0x52, 0xd, 0xa};
static const unsigned char string_14490114064326997348[8] = {0x4d, 0x4f, 0x44, 0x45, 0x20, 0x53, 0xd, 0xa};
static const unsigned char string_14735348259973616835[10] = {0x53, 0x49, 0x54, 0x45, 0x20, 0x3f, 0x3f, 0x3f, 0xd, 0xa};
static const unsigned char string_15511907849683514462[22] = {0x50, 0x4f, 0x52, 0x54, 0x20, 0x31, 0x32, 0x37, 0x2c, 0x30, 0x2c, 0x30, 0x2c, 0x31, 0x2c, 0x31, 0x32, 0x2c, 0x33, 0x34, 0xd, 0xa};
static const unsigned char string_17397722986415283233[10] = {0x4e, 0x4c, 0x53, 0x54, 0x20, 0x61, 0x62, 0x63, 0xd, 0xa};
static const unsigned char string_17621312972018287284[6] = {0x51, 0x55, 0x49, 0x54, 0xd, 0xa};
static const unsigned char string_17675720925268162070[10] = {0x41, 0x43, 0x43, 0x54, 0x20, 0x66, 0x74, 0x70, 0xd, 0xa};
static const unsigned char string_18269505720688649869[5] = {0x50, 0x57, 0x44, 0xd, 0xa};

// Forward declarations of containers
static size_t container_0(unsigned char*, size_t);
static size_t container_1(unsigned char*, size_t);

// Definition of containers
static size_t container_0(unsigned char* buf, size_t len) {
    // This container is struct Root
    size_t original_len = len;
    {
        size_t container_len = container_1(buf, len);
        buf += container_len; len -= container_len;
    }
    return original_len - len;
}
static size_t container_1(unsigned char* buf, size_t len) {
    size_t original_len = len;
    uint64_t oneof_selector = rand() % 33;
    switch(oneof_selector) {
        case 0: {
            if (UNLIKELY(len < sizeof(string_4041482461632183779))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_4041482461632183779, sizeof(string_4041482461632183779));
            buf += sizeof(string_4041482461632183779); len -= sizeof(string_4041482461632183779);
            break;
        }
        case 1: {
            if (UNLIKELY(len < sizeof(string_6117162860463527439))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_6117162860463527439, sizeof(string_6117162860463527439));
            buf += sizeof(string_6117162860463527439); len -= sizeof(string_6117162860463527439);
            break;
        }
        case 2: {
            if (UNLIKELY(len < sizeof(string_15511907849683514462))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_15511907849683514462, sizeof(string_15511907849683514462));
            buf += sizeof(string_15511907849683514462); len -= sizeof(string_15511907849683514462);
            break;
        }
        case 3: {
            if (UNLIKELY(len < sizeof(string_17675720925268162070))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_17675720925268162070, sizeof(string_17675720925268162070));
            buf += sizeof(string_17675720925268162070); len -= sizeof(string_17675720925268162070);
            break;
        }
        case 4: {
            if (UNLIKELY(len < sizeof(string_6470792744394151105))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_6470792744394151105, sizeof(string_6470792744394151105));
            buf += sizeof(string_6470792744394151105); len -= sizeof(string_6470792744394151105);
            break;
        }
        case 5: {
            if (UNLIKELY(len < sizeof(string_7115538435105230547))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_7115538435105230547, sizeof(string_7115538435105230547));
            buf += sizeof(string_7115538435105230547); len -= sizeof(string_7115538435105230547);
            break;
        }
        case 6: {
            if (UNLIKELY(len < sizeof(string_3407260272413593858))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_3407260272413593858, sizeof(string_3407260272413593858));
            buf += sizeof(string_3407260272413593858); len -= sizeof(string_3407260272413593858);
            break;
        }
        case 7: {
            if (UNLIKELY(len < sizeof(string_17621312972018287284))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_17621312972018287284, sizeof(string_17621312972018287284));
            buf += sizeof(string_17621312972018287284); len -= sizeof(string_17621312972018287284);
            break;
        }
        case 8: {
            if (UNLIKELY(len < sizeof(string_11337447964492719760))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_11337447964492719760, sizeof(string_11337447964492719760));
            buf += sizeof(string_11337447964492719760); len -= sizeof(string_11337447964492719760);
            break;
        }
        case 9: {
            if (UNLIKELY(len < sizeof(string_6810841031584417163))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_6810841031584417163, sizeof(string_6810841031584417163));
            buf += sizeof(string_6810841031584417163); len -= sizeof(string_6810841031584417163);
            break;
        }
        case 10: {
            if (UNLIKELY(len < sizeof(string_3137864909032443780))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_3137864909032443780, sizeof(string_3137864909032443780));
            buf += sizeof(string_3137864909032443780); len -= sizeof(string_3137864909032443780);
            break;
        }
        case 11: {
            if (UNLIKELY(len < sizeof(string_11229452647410007184))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_11229452647410007184, sizeof(string_11229452647410007184));
            buf += sizeof(string_11229452647410007184); len -= sizeof(string_11229452647410007184);
            break;
        }
        case 12: {
            if (UNLIKELY(len < sizeof(string_14490114064326997348))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_14490114064326997348, sizeof(string_14490114064326997348));
            buf += sizeof(string_14490114064326997348); len -= sizeof(string_14490114064326997348);
            break;
        }
        case 13: {
            if (UNLIKELY(len < sizeof(string_4892258942051522414))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_4892258942051522414, sizeof(string_4892258942051522414));
            buf += sizeof(string_4892258942051522414); len -= sizeof(string_4892258942051522414);
            break;
        }
        case 14: {
            if (UNLIKELY(len < sizeof(string_10273663291660329175))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_10273663291660329175, sizeof(string_10273663291660329175));
            buf += sizeof(string_10273663291660329175); len -= sizeof(string_10273663291660329175);
            break;
        }
        case 15: {
            if (UNLIKELY(len < sizeof(string_13522589807571429534))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_13522589807571429534, sizeof(string_13522589807571429534));
            buf += sizeof(string_13522589807571429534); len -= sizeof(string_13522589807571429534);
            break;
        }
        case 16: {
            if (UNLIKELY(len < sizeof(string_6140873944760370179))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_6140873944760370179, sizeof(string_6140873944760370179));
            buf += sizeof(string_6140873944760370179); len -= sizeof(string_6140873944760370179);
            break;
        }
        case 17: {
            if (UNLIKELY(len < sizeof(string_553001306382777075))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_553001306382777075, sizeof(string_553001306382777075));
            buf += sizeof(string_553001306382777075); len -= sizeof(string_553001306382777075);
            break;
        }
        case 18: {
            if (UNLIKELY(len < sizeof(string_9866723023257838052))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_9866723023257838052, sizeof(string_9866723023257838052));
            buf += sizeof(string_9866723023257838052); len -= sizeof(string_9866723023257838052);
            break;
        }
        case 19: {
            if (UNLIKELY(len < sizeof(string_8853320453514720409))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_8853320453514720409, sizeof(string_8853320453514720409));
            buf += sizeof(string_8853320453514720409); len -= sizeof(string_8853320453514720409);
            break;
        }
        case 20: {
            if (UNLIKELY(len < sizeof(string_1787227683893679301))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_1787227683893679301, sizeof(string_1787227683893679301));
            buf += sizeof(string_1787227683893679301); len -= sizeof(string_1787227683893679301);
            break;
        }
        case 21: {
            if (UNLIKELY(len < sizeof(string_14143458678229299797))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_14143458678229299797, sizeof(string_14143458678229299797));
            buf += sizeof(string_14143458678229299797); len -= sizeof(string_14143458678229299797);
            break;
        }
        case 22: {
            if (UNLIKELY(len < sizeof(string_452836965304285521))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_452836965304285521, sizeof(string_452836965304285521));
            buf += sizeof(string_452836965304285521); len -= sizeof(string_452836965304285521);
            break;
        }
        case 23: {
            if (UNLIKELY(len < sizeof(string_3888699756855861392))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_3888699756855861392, sizeof(string_3888699756855861392));
            buf += sizeof(string_3888699756855861392); len -= sizeof(string_3888699756855861392);
            break;
        }
        case 24: {
            if (UNLIKELY(len < sizeof(string_4918681801202486344))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_4918681801202486344, sizeof(string_4918681801202486344));
            buf += sizeof(string_4918681801202486344); len -= sizeof(string_4918681801202486344);
            break;
        }
        case 25: {
            if (UNLIKELY(len < sizeof(string_18269505720688649869))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_18269505720688649869, sizeof(string_18269505720688649869));
            buf += sizeof(string_18269505720688649869); len -= sizeof(string_18269505720688649869);
            break;
        }
        case 26: {
            if (UNLIKELY(len < sizeof(string_357634223873241833))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_357634223873241833, sizeof(string_357634223873241833));
            buf += sizeof(string_357634223873241833); len -= sizeof(string_357634223873241833);
            break;
        }
        case 27: {
            if (UNLIKELY(len < sizeof(string_17397722986415283233))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_17397722986415283233, sizeof(string_17397722986415283233));
            buf += sizeof(string_17397722986415283233); len -= sizeof(string_17397722986415283233);
            break;
        }
        case 28: {
            if (UNLIKELY(len < sizeof(string_14735348259973616835))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_14735348259973616835, sizeof(string_14735348259973616835));
            buf += sizeof(string_14735348259973616835); len -= sizeof(string_14735348259973616835);
            break;
        }
        case 29: {
            if (UNLIKELY(len < sizeof(string_7750727661420411684))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_7750727661420411684, sizeof(string_7750727661420411684));
            buf += sizeof(string_7750727661420411684); len -= sizeof(string_7750727661420411684);
            break;
        }
        case 30: {
            if (UNLIKELY(len < sizeof(string_1322821659644260744))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_1322821659644260744, sizeof(string_1322821659644260744));
            buf += sizeof(string_1322821659644260744); len -= sizeof(string_1322821659644260744);
            break;
        }
        case 31: {
            if (UNLIKELY(len < sizeof(string_6139412818626637881))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_6139412818626637881, sizeof(string_6139412818626637881));
            buf += sizeof(string_6139412818626637881); len -= sizeof(string_6139412818626637881);
            break;
        }
        case 32: {
            if (UNLIKELY(len < sizeof(string_10882850770906693452))) {
                goto container_end;
            }
            __builtin_memcpy_inline(buf, string_10882850770906693452, sizeof(string_10882850770906693452));
            buf += sizeof(string_10882850770906693452); len -= sizeof(string_10882850770906693452);
            break;
        }
        default: {
            __builtin_unreachable();
        }
    }
    container_end:
    return original_len - len;
}

// Entrypoint for the generator
size_t ftp_generator_generate(unsigned char* buf, size_t len) {
    if (UNLIKELY(!buf || !len)) {
        return 0;
    }
    
    return container_0(buf, len);
}
