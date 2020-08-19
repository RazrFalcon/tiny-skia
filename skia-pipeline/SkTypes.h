/*
 * Copyright 2006 The Android Open Source Project
 *
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#ifndef SkTypes_DEFINED
#define SkTypes_DEFINED

/** \file SkTypes.h
*/

#ifndef __clang__
#  error "only clang is supported"
#endif

// Allows embedders that want to disable macros that take arguments to just
// define that symbol to be one of these
#define SK_NOTHING_ARG1(arg1)
#define SK_NOTHING_ARG2(arg1, arg2)
#define SK_NOTHING_ARG3(arg1, arg2, arg3)

#if !defined(SK_BUILD_FOR_ANDROID) && !defined(SK_BUILD_FOR_IOS) && !defined(SK_BUILD_FOR_WIN) && \
    !defined(SK_BUILD_FOR_UNIX) && !defined(SK_BUILD_FOR_MAC)

    #if defined(_WIN32) || defined(__SYMBIAN32__)
        #define SK_BUILD_FOR_WIN
    #elif defined(ANDROID) || defined(__ANDROID__)
        #define SK_BUILD_FOR_ANDROID
    #elif defined(linux) || defined(__linux) || defined(__FreeBSD__) || \
          defined(__OpenBSD__) || defined(__sun) || defined(__NetBSD__) || \
          defined(__DragonFly__) || defined(__Fuchsia__) || \
          defined(__GLIBC__) || defined(__GNU__) || defined(__unix__)
        #define SK_BUILD_FOR_UNIX
    #elif TARGET_OS_IPHONE || TARGET_IPHONE_SIMULATOR
        #define SK_BUILD_FOR_IOS
    #else
        #define SK_BUILD_FOR_MAC
    #endif

#endif

#if !defined(SK_CPU_BENDIAN) && !defined(SK_CPU_LENDIAN)
    #if defined(__BYTE_ORDER__) && (__BYTE_ORDER__ == __ORDER_BIG_ENDIAN__)
        #define SK_CPU_BENDIAN
    #elif defined(__BYTE_ORDER__) && (__BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__)
        #define SK_CPU_LENDIAN
    #elif defined(__sparc) || defined(__sparc__) || \
      defined(_POWER) || defined(__powerpc__) || \
      defined(__ppc__) || defined(__hppa) || \
      defined(__PPC__) || defined(__PPC64__) || \
      defined(_MIPSEB) || defined(__ARMEB__) || \
      defined(__s390__) || \
      (defined(__sh__) && defined(__BIG_ENDIAN__)) || \
      (defined(__ia64) && defined(__BIG_ENDIAN__))
         #define SK_CPU_BENDIAN
    #else
        #define SK_CPU_LENDIAN
    #endif
#endif

#if defined(__i386) || defined(_M_IX86) ||  defined(__x86_64__) || defined(_M_X64)
  #define SK_CPU_X86 1
#endif

/**
 *  SK_CPU_SSE_LEVEL
 *
 *  If defined, SK_CPU_SSE_LEVEL should be set to the highest supported level.
 *  On non-intel CPU this should be undefined.
 */
#define SK_CPU_SSE_LEVEL_SSE1     10
#define SK_CPU_SSE_LEVEL_SSE2     20
#define SK_CPU_SSE_LEVEL_SSE3     30
#define SK_CPU_SSE_LEVEL_SSSE3    31
#define SK_CPU_SSE_LEVEL_SSE41    41
#define SK_CPU_SSE_LEVEL_SSE42    42
#define SK_CPU_SSE_LEVEL_AVX      51
#define SK_CPU_SSE_LEVEL_AVX2     52
#define SK_CPU_SSE_LEVEL_SKX      60

// When targetting iOS and using gyp to generate the build files, it is not
// possible to select files to build depending on the architecture (i.e. it
// is not possible to use hand optimized assembly implementation). In that
// configuration SK_BUILD_NO_OPTS is defined. Remove optimisation then.
#ifdef SK_BUILD_NO_OPTS
    #define SK_CPU_SSE_LEVEL 0
#endif

// Are we in GCC/Clang?
#ifndef SK_CPU_SSE_LEVEL
    // These checks must be done in descending order to ensure we set the highest
    // available SSE level.
    #if defined(__AVX512F__) && defined(__AVX512DQ__) && defined(__AVX512CD__) && \
        defined(__AVX512BW__) && defined(__AVX512VL__)
        #define SK_CPU_SSE_LEVEL    SK_CPU_SSE_LEVEL_SKX
    #elif defined(__AVX2__)
        #define SK_CPU_SSE_LEVEL    SK_CPU_SSE_LEVEL_AVX2
    #elif defined(__AVX__)
        #define SK_CPU_SSE_LEVEL    SK_CPU_SSE_LEVEL_AVX
    #elif defined(__SSE4_2__)
        #define SK_CPU_SSE_LEVEL    SK_CPU_SSE_LEVEL_SSE42
    #elif defined(__SSE4_1__)
        #define SK_CPU_SSE_LEVEL    SK_CPU_SSE_LEVEL_SSE41
    #elif defined(__SSSE3__)
        #define SK_CPU_SSE_LEVEL    SK_CPU_SSE_LEVEL_SSSE3
    #elif defined(__SSE3__)
        #define SK_CPU_SSE_LEVEL    SK_CPU_SSE_LEVEL_SSE3
    #elif defined(__SSE2__)
        #define SK_CPU_SSE_LEVEL    SK_CPU_SSE_LEVEL_SSE2
    #endif
#endif

// Are we in VisualStudio?
#ifndef SK_CPU_SSE_LEVEL
    // These checks must be done in descending order to ensure we set the highest
    // available SSE level. 64-bit intel guarantees at least SSE2 support.
    #if defined(__AVX512F__) && defined(__AVX512DQ__) && defined(__AVX512CD__) && \
        defined(__AVX512BW__) && defined(__AVX512VL__)
        #define SK_CPU_SSE_LEVEL        SK_CPU_SSE_LEVEL_SKX
    #elif defined(__AVX2__)
        #define SK_CPU_SSE_LEVEL        SK_CPU_SSE_LEVEL_AVX2
    #elif defined(__AVX__)
        #define SK_CPU_SSE_LEVEL        SK_CPU_SSE_LEVEL_AVX
    #elif defined(_M_X64) || defined(_M_AMD64)
        #define SK_CPU_SSE_LEVEL        SK_CPU_SSE_LEVEL_SSE2
    #elif defined(_M_IX86_FP)
        #if _M_IX86_FP >= 2
            #define SK_CPU_SSE_LEVEL    SK_CPU_SSE_LEVEL_SSE2
        #elif _M_IX86_FP == 1
            #define SK_CPU_SSE_LEVEL    SK_CPU_SSE_LEVEL_SSE1
        #endif
    #endif
#endif

// ARM defines
#if defined(__arm__) && (!defined(__APPLE__) || !TARGET_IPHONE_SIMULATOR)
    #define SK_CPU_ARM32
#elif defined(__aarch64__) && !defined(SK_BUILD_NO_OPTS)
    #define SK_CPU_ARM64
#endif

// All 64-bit ARM chips have NEON.  Many 32-bit ARM chips do too.
#if !defined(SK_ARM_HAS_NEON) && !defined(SK_BUILD_NO_OPTS) && defined(__ARM_NEON)
    #define SK_ARM_HAS_NEON
#endif

// Really this __APPLE__ check shouldn't be necessary, but it seems that Apple's Clang defines
// __ARM_FEATURE_CRC32 for -arch arm64, even though their chips don't support those instructions!
#if defined(__ARM_FEATURE_CRC32) && !defined(__APPLE__)
    #define SK_ARM_HAS_CRC32
#endif


// DLL/.so exports.
#if !defined(SKIA_IMPLEMENTATION)
    #define SKIA_IMPLEMENTATION 0
#endif
#if !defined(SK_API)
    #if defined(SKIA_DLL)
        #if defined(_MSC_VER)
            #if SKIA_IMPLEMENTATION
                #define SK_API __declspec(dllexport)
            #else
                #define SK_API __declspec(dllimport)
            #endif
        #else
            #define SK_API __attribute__((visibility("default")))
        #endif
    #else
        #define SK_API
    #endif
#endif

// SK_SPI is functionally identical to SK_API, but used within src to clarify that it's less stable
#if !defined(SK_SPI)
    #define SK_SPI SK_API
#endif

// IWYU pragma: begin_exports
#include <stddef.h>
#include <stdint.h>
// IWYU pragma: end_exports

#if defined(SK_CPU_LENDIAN) && defined(SK_CPU_BENDIAN)
#  error "cannot define both SK_CPU_LENDIAN and SK_CPU_BENDIAN"
#elif !defined(SK_CPU_LENDIAN) && !defined(SK_CPU_BENDIAN)
#  error "must define either SK_CPU_LENDIAN or SK_CPU_BENDIAN"
#endif

#if defined(SK_CPU_BENDIAN) && !defined(I_ACKNOWLEDGE_SKIA_DOES_NOT_SUPPORT_BIG_ENDIAN)
    #error "The Skia team is not endian-savvy enough to support big-endian CPUs."
    #error "If you still want to use Skia,"
    #error "please define I_ACKNOWLEDGE_SKIA_DOES_NOT_SUPPORT_BIG_ENDIAN."
#endif

#if !defined(SK_ATTRIBUTE)
#  if defined(__clang__) || defined(__GNUC__)
#    define SK_ATTRIBUTE(attr) __attribute__((attr))
#  else
#    define SK_ATTRIBUTE(attr)
#  endif
#endif

#if !defined(SK_SUPPORT_GPU)
#  define SK_SUPPORT_GPU 1
#endif

/**
 * If GPU is enabled but no GPU backends are enabled then enable GL by default.
 * Traditionally clients have relied on Skia always building with the GL backend
 * and opting in to additional backends. TODO: Require explicit opt in for GL.
 */
#if SK_SUPPORT_GPU
#  if !defined(SK_GL) && !defined(SK_VULKAN) && !defined(SK_METAL) && !defined(SK_DAWN) && !defined(SK_DIRECT3D)
#    define SK_GL
#  endif
#else
#  undef SK_GL
#  undef SK_VULKAN
#  undef SK_METAL
#  undef SK_DAWN
#  undef SK_DIRECT3D
#endif

#if !defined(SkUNREACHABLE)
#  if defined(_MSC_VER) && !defined(__clang__)
#    define SkUNREACHABLE __assume(false)
#  else
#    define SkUNREACHABLE __builtin_unreachable()
#  endif
#endif

#if !defined(SK_UNUSED)
#  if !defined(__clang__) && defined(_MSC_VER)
#    define SK_UNUSED __pragma(warning(suppress:4189))
#  else
#    define SK_UNUSED SK_ATTRIBUTE(unused)
#  endif
#endif

/**
 * If your judgment is better than the compiler's (i.e. you've profiled it),
 * you can use SK_ALWAYS_INLINE to force inlining. E.g.
 *     inline void someMethod() { ... }             // may not be inlined
 *     SK_ALWAYS_INLINE void someMethod() { ... }   // should always be inlined
 */
#if !defined(SK_ALWAYS_INLINE)
#  if defined(SK_BUILD_FOR_WIN)
#    define SK_ALWAYS_INLINE __forceinline
#  else
#    define SK_ALWAYS_INLINE SK_ATTRIBUTE(always_inline) inline
#  endif
#endif

/**
 * If your judgment is better than the compiler's (i.e. you've profiled it),
 * you can use SK_NEVER_INLINE to prevent inlining.
 */
#if !defined(SK_NEVER_INLINE)
#  if defined(SK_BUILD_FOR_WIN)
#    define SK_NEVER_INLINE __declspec(noinline)
#  else
#    define SK_NEVER_INLINE SK_ATTRIBUTE(noinline)
#  endif
#endif

#if SK_CPU_SSE_LEVEL >= SK_CPU_SSE_LEVEL_SSE1
    #define SK_PREFETCH(ptr) _mm_prefetch(reinterpret_cast<const char*>(ptr), _MM_HINT_T0)
#elif defined(__GNUC__)
    #define SK_PREFETCH(ptr) __builtin_prefetch(ptr)
#else
    #define SK_PREFETCH(ptr)
#endif

#ifndef SK_PRINTF_LIKE
#  if defined(__clang__) || defined(__GNUC__)
#    define SK_PRINTF_LIKE(A, B) __attribute__((format(printf, (A), (B))))
#  else
#    define SK_PRINTF_LIKE(A, B)
#  endif
#endif

#ifndef SK_ALLOW_STATIC_GLOBAL_INITIALIZERS
    #define SK_ALLOW_STATIC_GLOBAL_INITIALIZERS 0
#endif

#if !defined(SK_GAMMA_EXPONENT)
    #define SK_GAMMA_EXPONENT (0.0f)  // SRGB
#endif

#ifndef GR_TEST_UTILS
#  define GR_TEST_UTILS 0
#endif

#if defined(SK_HISTOGRAM_ENUMERATION) && defined(SK_HISTOGRAM_BOOLEAN)
#  define SK_HISTOGRAMS_ENABLED 1
#else
#  define SK_HISTOGRAMS_ENABLED 0
#endif

#ifndef SK_HISTOGRAM_BOOLEAN
#  define SK_HISTOGRAM_BOOLEAN(name, value)
#endif

#ifndef SK_HISTOGRAM_ENUMERATION
#  define SK_HISTOGRAM_ENUMERATION(name, value, boundary_value)
#endif

#ifndef SK_DISABLE_LEGACY_SHADERCONTEXT
#define SK_ENABLE_LEGACY_SHADERCONTEXT
#endif

#ifdef SK_ENABLE_API_AVAILABLE
#define SK_API_AVAILABLE API_AVAILABLE
#else
#define SK_API_AVAILABLE(...)
#endif

#endif
