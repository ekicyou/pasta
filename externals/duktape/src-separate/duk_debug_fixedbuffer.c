/*
 *  Fixed buffer helper useful for debugging, requires no allocation
 *  which is critical for debugging.
 */

#include "duk_internal.h"

#ifdef DUK_USE_DEBUG

void duk_fb_put_bytes(duk_fixedbuffer *fb, duk_uint8_t *buffer, duk_size_t length) {
	duk_size_t avail;
	duk_size_t copylen;

	avail = (fb->offset >= fb->length ? (duk_size_t) 0 : (duk_size_t) (fb->length - fb->offset));
	if (length > avail) {
		copylen = avail;
		fb->truncated = 1;
	} else {
		copylen = length;
	}
	DUK_MEMCPY(fb->buffer + fb->offset, buffer, copylen);
	fb->offset += copylen;
}

void duk_fb_put_byte(duk_fixedbuffer *fb, duk_uint8_t x) {
	duk_fb_put_bytes(fb, &x, 1);
}

void duk_fb_put_cstring(duk_fixedbuffer *fb, const char *x) {
	duk_fb_put_bytes(fb, (duk_uint8_t *) x, (duk_size_t) DUK_STRLEN(x));
}

void duk_fb_sprintf(duk_fixedbuffer *fb, const char *fmt, ...) {
	duk_size_t avail;
	va_list ap;

	va_start(ap, fmt);
	avail = (fb->offset >= fb->length ? (duk_size_t) 0 : (duk_size_t) (fb->length - fb->offset));
	if (avail > 0) {
		duk_int_t res = (duk_int_t) DUK_VSNPRINTF((char *) (fb->buffer + fb->offset), avail, fmt, ap);
		if (res < 0) {
			/* error */
		} else if ((duk_size_t) res >= avail) {
			/* (maybe) truncated */
			fb->offset += avail;
			if ((duk_size_t) res > avail) {
				/* actual chars dropped (not just NUL term) */
				fb->truncated = 1;
			}
		} else {
			/* normal */
			fb->offset += res;
		}
	}
	va_end(ap);
}

duk_bool_t duk_fb_is_full(duk_fixedbuffer *fb) {
	return (fb->offset >= fb->length);
}

#endif  /* DUK_USE_DEBUG */
