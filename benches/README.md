Benchmarking is hard... (c)

## Environment

- All test were run on Gentoo Linux.
- cairo and Skia were built with `-march=native` flag on AMD 3700X.
- `tiny-skia` and `raqote` were built with `-Ctarget-cpu=native`.

## Results

### Blending modes

```
test clear_tiny_skia            ... bench:      43,649 ns/iter (+/- 429)
test clear_skia                 ... bench:      50,203 ns/iter (+/- 1,209)
test clear_cairo                ... bench:      62,331 ns/iter (+/- 758)
test clear_raqote               ... bench:   1,043,659 ns/iter (+/- 259,923)

test source_tiny_skia           ... bench:      43,342 ns/iter (+/- 1,661)
test source_skia                ... bench:      50,949 ns/iter (+/- 2,114)
test source_cairo               ... bench:      62,548 ns/iter (+/- 2,803)
test source_raqote              ... bench:   1,114,946 ns/iter (+/- 57,617)

test destination_tiny_skia      ... bench:     266,909 ns/iter (+/- 3,329)
test destination_skia           ... bench:       5,318 ns/iter (+/- 187)
test destination_cairo          ... bench:           4 ns/iter (+/- 0)      (wut?!)
test destination_raqote         ... bench:     980,993 ns/iter (+/- 47,829)

test source_over_tiny_skia      ... bench:     249,321 ns/iter (+/- 620)
test source_over_skia           ... bench:     261,943 ns/iter (+/- 1,591)
test source_over_cairo          ... bench:     212,838 ns/iter (+/- 4,089)
test source_over_raqote         ... bench:   2,645,207 ns/iter (+/- 254,594)

test destination_over_tiny_skia ... bench:     287,740 ns/iter (+/- 2,772)
test destination_over_skia      ... bench:     306,542 ns/iter (+/- 2,728)
test destination_over_cairo     ... bench:     236,970 ns/iter (+/- 5,202)
test destination_over_raqote    ... bench:   3,446,445 ns/iter (+/- 43,492)

test source_in_tiny_skia        ... bench:     273,366 ns/iter (+/- 3,361)
test source_in_skia             ... bench:     291,008 ns/iter (+/- 3,723)
test source_in_cairo            ... bench:   1,104,817 ns/iter (+/- 16,719)
test source_in_raqote           ... bench:   1,163,684 ns/iter (+/- 57,542)

test destination_in_tiny_skia   ... bench:     275,105 ns/iter (+/- 1,871)
test destination_in_skia        ... bench:     293,412 ns/iter (+/- 2,173)
test destination_in_cairo       ... bench:   1,103,758 ns/iter (+/- 16,687)
test destination_in_raqote      ... bench:   1,193,288 ns/iter (+/- 61,003)

test source_out_tiny_skia       ... bench:     278,840 ns/iter (+/- 4,703)
test source_out_skia            ... bench:     297,537 ns/iter (+/- 3,833)
test source_out_cairo           ... bench:   1,122,571 ns/iter (+/- 9,405)
test source_out_raqote          ... bench:   1,182,724 ns/iter (+/- 26,307)

test destination_out_tiny_skia  ... bench:     279,266 ns/iter (+/- 3,715)
test destination_out_skia       ... bench:     299,481 ns/iter (+/- 2,212)
test destination_out_cairo      ... bench:     556,088 ns/iter (+/- 3,815)
test destination_out_raqote     ... bench:   1,193,241 ns/iter (+/- 302,027)

test source_atop_tiny_skia      ... bench:     288,170 ns/iter (+/- 793)
test source_atop_skia           ... bench:     307,222 ns/iter (+/- 1,550)
test source_atop_cairo          ... bench:     663,254 ns/iter (+/- 3,940)
test source_atop_raqote         ... bench:   1,501,419 ns/iter (+/- 31,417)

test destination_atop_tiny_skia ... bench:     288,187 ns/iter (+/- 4,850)
test destination_atop_skia      ... bench:     306,610 ns/iter (+/- 1,755)
test destination_atop_cairo     ... bench:   1,220,442 ns/iter (+/- 10,316)
test destination_atop_raqote    ... bench:   1,511,405 ns/iter (+/- 48,424)

test xor_tiny_skia              ... bench:     292,862 ns/iter (+/- 4,722)
test xor_skia                   ... bench:     310,439 ns/iter (+/- 4,453)
test xor_cairo                  ... bench:     671,439 ns/iter (+/- 4,048)
test xor_raqote                 ... bench:   1,634,236 ns/iter (+/- 168,279)

test plus_tiny_skia             ... bench:     268,093 ns/iter (+/- 1,799)
test plus_skia                  ... bench:     286,504 ns/iter (+/- 1,964)
test plus_cairo                 ... bench:     122,496 ns/iter (+/- 3,177)
test plus_raqote                ... bench:   4,892,311 ns/iter (+/- 561,835)

test modulate_tiny_skia         ... bench:     273,577 ns/iter (+/- 4,543)
test modulate_skia              ... bench:     290,988 ns/iter (+/- 3,277)

test screen_tiny_skia           ... bench:     293,752 ns/iter (+/- 975)
test screen_skia                ... bench:     312,507 ns/iter (+/- 1,955)
test screen_cairo               ... bench:   3,837,010 ns/iter (+/- 22,167)
test screen_raqote              ... bench:   1,459,500 ns/iter (+/- 31,974)

test overlay_tiny_skia          ... bench:     368,250 ns/iter (+/- 6,122)
test overlay_skia               ... bench:     385,684 ns/iter (+/- 3,953)
test overlay_cairo              ... bench:   3,412,192 ns/iter (+/- 38,283)
test overlay_raqote             ... bench:   7,016,443 ns/iter (+/- 69,148)

test darken_tiny_skia           ... bench:     299,405 ns/iter (+/- 5,044)
test darken_skia                ... bench:     317,215 ns/iter (+/- 4,315)
test darken_cairo               ... bench:   3,579,931 ns/iter (+/- 28,375)
test darken_raqote              ... bench:   5,230,384 ns/iter (+/- 79,313)

test lighten_tiny_skia          ... bench:     299,555 ns/iter (+/- 5,932)
test lighten_skia               ... bench:     319,867 ns/iter (+/- 5,679)
test lighten_cairo              ... bench:   3,584,409 ns/iter (+/- 15,092)
test lighten_raqote             ... bench:   5,255,202 ns/iter (+/- 44,182)

test color_dodge_tiny_skia      ... bench:     620,235 ns/iter (+/- 8,234)
test color_dodge_skia           ... bench:     679,996 ns/iter (+/- 8,442)
test color_dodge_cairo          ... bench:   5,151,488 ns/iter (+/- 93,180)
test color_dodge_raqote         ... bench:   9,634,447 ns/iter (+/- 80,584)

test color_burn_tiny_skia       ... bench:     635,705 ns/iter (+/- 8,811)
test color_burn_skia            ... bench:     708,962 ns/iter (+/- 9,257)
test color_burn_cairo           ... bench:   5,007,181 ns/iter (+/- 36,599)
test color_burn_raqote          ... bench:   9,617,102 ns/iter (+/- 177,719)

test hard_light_tiny_skia       ... bench:     353,736 ns/iter (+/- 6,729)
test hard_light_skia            ... bench:     370,307 ns/iter (+/- 6,300)
test hard_light_cairo           ... bench:   3,442,584 ns/iter (+/- 37,358)
test hard_light_raqote          ... bench:   7,034,417 ns/iter (+/- 70,127)

test soft_light_tiny_skia       ... bench:     942,232 ns/iter (+/- 10,979)
test soft_light_skia            ... bench:     986,403 ns/iter (+/- 10,433)
test soft_light_cairo           ... bench:   5,900,415 ns/iter (+/- 26,181)
test soft_light_raqote          ... bench:  11,941,630 ns/iter (+/- 291,833)

test difference_tiny_skia       ... bench:     307,823 ns/iter (+/- 3,577)
test difference_skia            ... bench:     326,285 ns/iter (+/- 766)
test difference_cairo           ... bench:   3,936,284 ns/iter (+/- 38,808)
test difference_raqote          ... bench:   5,718,776 ns/iter (+/- 56,842)

test exclusion_tiny_skia        ... bench:     298,500 ns/iter (+/- 4,695)
test exclusion_skia             ... bench:     316,235 ns/iter (+/- 2,617)
test exclusion_cairo            ... bench:   3,842,131 ns/iter (+/- 15,905)
test exclusion_raqote           ... bench:   6,082,393 ns/iter (+/- 104,945)

test multiply_tiny_skia         ... bench:     299,151 ns/iter (+/- 3,643)
test multiply_skia              ... bench:     316,841 ns/iter (+/- 4,439)
test multiply_cairo             ... bench:   3,608,817 ns/iter (+/- 21,714)
test multiply_raqote            ... bench:   5,986,364 ns/iter (+/- 76,662)

test hue_tiny_skia              ... bench:   1,369,114 ns/iter (+/- 17,099)
test hue_skia                   ... bench:   1,413,791 ns/iter (+/- 19,080)
test hue_cairo                  ... bench:   7,517,902 ns/iter (+/- 33,529)
test hue_raqote                 ... bench:  13,716,827 ns/iter (+/- 217,137)

test saturation_tiny_skia       ... bench:   1,375,247 ns/iter (+/- 13,813)
test saturation_skia            ... bench:   1,411,117 ns/iter (+/- 15,472)
test saturation_cairo           ... bench:   7,443,261 ns/iter (+/- 80,673)
test saturation_raqote          ... bench:  13,752,382 ns/iter (+/- 111,587)

test color_tiny_skia            ... bench:   1,068,994 ns/iter (+/- 5,427)
test color_skia                 ... bench:   1,115,800 ns/iter (+/- 4,762)
test color_cairo                ... bench:   6,070,058 ns/iter (+/- 38,963)
test color_raqote               ... bench:  10,537,391 ns/iter (+/- 75,747)

test luminosity_tiny_skia       ... bench:   1,037,656 ns/iter (+/- 11,953)
test luminosity_skia            ... bench:   1,090,402 ns/iter (+/- 12,929)
test luminosity_cairo           ... bench:   6,124,294 ns/iter (+/- 46,113)
test luminosity_raqote          ... bench:  10,488,916 ns/iter (+/- 129,615)
```

### memset fill

Shape filling by overwritting original pixels. No blending.

```
test opaque_fill_tiny_skia ... bench:      51,620 ns/iter (+/- 3,083)
test opaque_fill_skia      ... bench:      47,662 ns/iter (+/- 1,119)
test opaque_fill_cairo     ... bench:      45,804 ns/iter (+/- 263)
test opaque_fill_raqote    ... bench:   2,432,875 ns/iter (+/- 51,948)

test source_fill_tiny_skia ... bench:      51,448 ns/iter (+/- 2,672)
test source_fill_skia      ... bench:      50,663 ns/iter (+/- 207)
test source_fill_cairo     ... bench:      48,839 ns/iter (+/- 354)
test source_fill_raqote    ... bench:     678,795 ns/iter (+/- 10,330)
```

### PNG

```
test decode_raw_rgb  ... bench:      67,293 ns/iter (+/- 710)
test decode_rgb      ... bench:     128,367 ns/iter (+/- 607)

test decode_raw_rgba ... bench:      90,793 ns/iter (+/- 978)
test decode_rgba     ... bench:     109,752 ns/iter (+/- 1,521)

test encode_raw_rgba ... bench:     275,042 ns/iter (+/- 4,476)
test encode_rgba     ... bench:     327,967 ns/iter (+/- 4,542)
```

## Running benchmarks

We support only Linux. Benchmark may work on other OS'es, but it will require a lot of preperation.

You have to install cairo first and built Skia from sources (see below).

Run:

```sh
export SKIA_DIR="/path/to/skia"
export SKIA_LIB_DIR="/path/to/skia/out/Shared"
export LD_LIBRARY_PATH="/path/to/skia/out/Shared"
cargo bench
```

## Building Skia

(we support only Linux)

You will need `git`, `clang`, `ninja` and Python 2.

```sh
git clone https://skia.googlesource.com/skia.git
cd skia
git fetch --all
git checkout -b m85 origin/chrome/m85
python2 tools/git-sync-deps
bin/gn gen out/Shared --args='
    is_official_build=false
    is_component_build=true
    is_debug=false
    cc="clang"
    cxx="clang++"
    extra_cflags_cc=["-march=native", "-DSK_FORCE_RASTER_PIPELINE_BLITTER"]
    werror=false
    paragraph_gms_enabled=false
    paragraph_tests_enabled=false
    skia_enable_android_utils=false
    skia_enable_discrete_gpu=false
    skia_enable_gpu=false
    skia_enable_nvpr=false
    skia_enable_particles=false
    skia_enable_pdf=false
    skia_enable_skottie=false
    skia_enable_skrive=false
    skia_enable_skshaper=false
    skia_enable_sksl_interpreter=false
    skia_enable_skvm_jit=false
    skia_enable_tools=false
    skia_use_expat=false
    skia_use_gl=false
    skia_use_harfbuzz=false
    skia_use_icu=false
    skia_use_libgifcodec=false
    skia_use_libheif=false
    skia_use_libjpeg_turbo_decode=false
    skia_use_libjpeg_turbo_encode=false
    skia_use_libwebp_decode=false
    skia_use_libwebp_encode=false
    skia_use_lua=false
    skia_use_piex=false'
ninja -C out/Shared
```
