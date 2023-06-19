#include <intrin.h>
#include <cstdint>

extern "C" void decrypt_fname(char* buf, unsigned __int16 len, char wide, unsigned __int32 key) {
	char* v2; __int64 result; unsigned int v5; __int64 v6; __int64 v7; int v8;
	result = key;

	v2 = buf;
	v5 = len;
	v8 = result;
	if (v5) {
		v6 = 0;
		v7 = v5;
		do {
			++v2;
			result = v6++ & 3;
			*(v2 - 1) ^= v5 ^ *((char*)&v8 + result);
			--v7;
		} while (v7);
	}
};