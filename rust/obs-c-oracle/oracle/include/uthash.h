/* Test-only stand-in for the system <uthash.h>, which libobs/util/uthash.h
 * wraps. The oracle crate cannot rely on uthash being installed (the Rust
 * CI jobs do not install it), so this implements just the macros
 * util/text-lookup.c uses, as a singly linked list. Lookups give the same
 * results whatever the hash function, so the oracle's observable behavior
 * is unchanged. */
#pragma once

#include <stddef.h>
#include <string.h>

typedef struct UT_hash_handle {
	void *next;
	const void *key;
	size_t keylen;
} UT_hash_handle;

/* The handle of `el`, found at the same offset as in `head`. */
#define UT_HH_(head, el) ((UT_hash_handle *)((char *)(el) + ((char *)&(head)->hh - (char *)(head))))

#define HASH_FIND_STR(head, findstr, out)                                                       \
	do {                                                                                    \
		const char *ut_k_ = (findstr);                                                  \
		size_t ut_l_ = strlen(ut_k_);                                                   \
		(out) = (head);                                                                 \
		while ((out) && !((out)->hh.keylen == ut_l_ && memcmp((out)->hh.key, ut_k_, ut_l_) == 0)) \
			(out) = (out)->hh.next;                                                 \
	} while (0)

#define HASH_DELETE(hh, head, delptr)                                         \
	do {                                                                  \
		void *ut_d_ = (delptr);                                       \
		if ((void *)(head) == ut_d_) {                                \
			(head) = (head)->hh.next;                             \
		} else {                                                      \
			void *ut_e_ = (head);                                 \
			while (UT_HH_(head, ut_e_)->next != ut_d_)            \
				ut_e_ = UT_HH_(head, ut_e_)->next;            \
			UT_HH_(head, ut_e_)->next = UT_HH_(head, ut_d_)->next; \
		}                                                             \
	} while (0)

#define HASH_REPLACE_STR(head, strfield, add, replaced)              \
	do {                                                         \
		(add)->hh.key = (add)->strfield;                     \
		(add)->hh.keylen = strlen((add)->strfield);          \
		HASH_FIND_STR(head, (add)->strfield, replaced);      \
		if (replaced)                                        \
			HASH_DELETE(hh, head, replaced);             \
		(add)->hh.next = (head);                             \
		(head) = (add);                                      \
	} while (0)

#define HASH_ITER(hh, head, el, tmp)                                         \
	for ((el) = (head), (tmp) = (el) ? (el)->hh.next : NULL; (el); \
	     (el) = (tmp), (tmp) = (el) ? (el)->hh.next : NULL)
