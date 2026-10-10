/* Test-only stand-in for <simde/x86/sse2.h>, used by the C oracle so that
 * `cargo test` does not need SIMDe installed. libobs/util/sse-intrin.h
 * includes SIMDe everywhere except MSVC/MinGW on x86.
 *
 * On x86 this is the native SSE header, which is what SIMDe maps to there.
 * Elsewhere it defines, in plain C, only the intrinsics the graphics math
 * headers use. Each one follows the SSE definition lane by lane, which is
 * also what SIMDe guarantees on NEON (including the NaN and signed-zero
 * rules of minps/maxps). The oracle is built with -ffp-contract=off, so no
 * multiply-add is fused. */
#pragma once

#if (defined(__x86_64__) || defined(__i386__)) && !defined(OBS_ORACLE_SCALAR_SSE)
#include <emmintrin.h>
#else

typedef struct {
	_Alignas(16) float f[4];
} __m128;

#define _MM_SHUFFLE(z, y, x, w) (((z) << 6) | ((y) << 4) | ((x) << 2) | (w))

static inline __m128 oracle_sse_make(float a, float b, float c, float d)
{
	__m128 r;
	r.f[0] = a;
	r.f[1] = b;
	r.f[2] = c;
	r.f[3] = d;
	return r;
}

static inline __m128 _mm_setzero_ps(void)
{
	return oracle_sse_make(0.0f, 0.0f, 0.0f, 0.0f);
}

static inline __m128 _mm_set1_ps(float f)
{
	return oracle_sse_make(f, f, f, f);
}

/* Arguments are high lane first. */
static inline __m128 _mm_set_ps(float w, float z, float y, float x)
{
	return oracle_sse_make(x, y, z, w);
}

static inline __m128 _mm_add_ps(__m128 a, __m128 b)
{
	return oracle_sse_make(a.f[0] + b.f[0], a.f[1] + b.f[1], a.f[2] + b.f[2], a.f[3] + b.f[3]);
}

static inline __m128 _mm_sub_ps(__m128 a, __m128 b)
{
	return oracle_sse_make(a.f[0] - b.f[0], a.f[1] - b.f[1], a.f[2] - b.f[2], a.f[3] - b.f[3]);
}

static inline __m128 _mm_mul_ps(__m128 a, __m128 b)
{
	return oracle_sse_make(a.f[0] * b.f[0], a.f[1] * b.f[1], a.f[2] * b.f[2], a.f[3] * b.f[3]);
}

static inline __m128 _mm_div_ps(__m128 a, __m128 b)
{
	return oracle_sse_make(a.f[0] / b.f[0], a.f[1] / b.f[1], a.f[2] / b.f[2], a.f[3] / b.f[3]);
}

/* minps/maxps return the second operand unless the comparison holds, so a
 * NaN in either lane, or equal zeros of either sign, give `b`. */
static inline float oracle_sse_min(float a, float b)
{
	return a < b ? a : b;
}

static inline float oracle_sse_max(float a, float b)
{
	return a > b ? a : b;
}

static inline __m128 _mm_min_ps(__m128 a, __m128 b)
{
	return oracle_sse_make(oracle_sse_min(a.f[0], b.f[0]), oracle_sse_min(a.f[1], b.f[1]),
			       oracle_sse_min(a.f[2], b.f[2]), oracle_sse_min(a.f[3], b.f[3]));
}

static inline __m128 _mm_max_ps(__m128 a, __m128 b)
{
	return oracle_sse_make(oracle_sse_max(a.f[0], b.f[0]), oracle_sse_max(a.f[1], b.f[1]),
			       oracle_sse_max(a.f[2], b.f[2]), oracle_sse_max(a.f[3], b.f[3]));
}

static inline __m128 _mm_movehl_ps(__m128 a, __m128 b)
{
	return oracle_sse_make(b.f[2], b.f[3], a.f[2], a.f[3]);
}

static inline __m128 _mm_movelh_ps(__m128 a, __m128 b)
{
	return oracle_sse_make(a.f[0], a.f[1], b.f[0], b.f[1]);
}

static inline __m128 _mm_unpacklo_ps(__m128 a, __m128 b)
{
	return oracle_sse_make(a.f[0], b.f[0], a.f[1], b.f[1]);
}

static inline __m128 _mm_unpackhi_ps(__m128 a, __m128 b)
{
	return oracle_sse_make(a.f[2], b.f[2], a.f[3], b.f[3]);
}

static inline __m128 _mm_shuffle_ps(__m128 a, __m128 b, int imm)
{
	return oracle_sse_make(a.f[imm & 3], a.f[(imm >> 2) & 3], b.f[(imm >> 4) & 3], b.f[(imm >> 6) & 3]);
}

#endif
