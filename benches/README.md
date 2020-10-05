Benchmarking is hard... (c)

## Environment

- All test were run on Gentoo Linux with AMD 3700X.
- tiny-skia SSE2 is built with `-Ctarget-cpu=x86-64`
- tiny-skia AVX2 is built with `-Ctarget-cpu=haswell`
- Skia v85.
- Skia SSE2 is built using clang with `-march=x86-64`
- Skia AVX2 is built using clang with `-march=haswell`
- cairo v1.16.0.
- cairo is built using gcc with `-march=native`, because it's the system one
- raqote version can be found at `Cargo.lock`
- raqote is built with `-Ctarget-cpu=x86-64`.
  Testing with `haswell` doesn't change the results much.

## Results

All measurement are in nanoseconds. Lower is better.

### blending modes

`blend.rs`

| Mode/Library         | tiny-skia SSE2 | tiny-skia AVX2 | Skia SSE2  | Skia AVX2  | cairo      | raqote     |
| -------------------- | -------------: | -------------: | ---------: | ---------: | ---------: | ---------: |
| clear                |         49,373 |         42,307 |     45,898 |     50,203 |     62,331 |  1,043,659 |
| source               |         49,410 |         41,941 |     46,195 |     50,949 |     62,548 |  1,114,946 |
| destination          |             53 |             47 |      5,309 |      5,318 |          4 |    980,993 |
| source_over          |        529,245 |        414,233 |    490,423 |    261,943 |    212,838 |  2,645,207 |
| destination_over     |        597,339 |        453,759 |    583,660 |    306,542 |    236,970 |  3,446,445 |
| source_in            |        572,713 |        427,830 |    577,464 |    291,008 |  1,104,817 |  1,163,684 |
| destination_in       |        527,886 |        407,576 |    570,325 |    293,412 |  1,103,758 |  1,193,288 |
| source_out           |        580,979 |        434,952 |    576,125 |    297,537 |  1,122,571 |  1,182,724 |
| destination_out      |        533,645 |        408,273 |    576,521 |    299,481 |    556,088 |  1,193,241 |
| source_atop          |        595,859 |        447,737 |    603,723 |    307,222 |    663,254 |  1,501,419 |
| destination_atop     |        583,768 |        446,935 |    595,771 |    306,610 |  1,220,442 |  1,511,405 |
| xor                  |        616,175 |        453,713 |    601,201 |    310,439 |    671,439 |  1,634,236 |
| plus                 |        515,268 |        395,012 |    559,063 |    286,504 |    122,496 |  4,892,311 |
| modulate             |        534,183 |        407,639 |    570,734 |    290,988 |          - |          - |
| screen               |        541,508 |        414,274 |    598,637 |    312,507 |  3,837,010 |  1,459,500 |
| overlay              |        877,120 |        525,368 |    733,158 |    385,684 |  3,412,192 |  7,016,443 |
| darken               |        656,495 |        457,057 |    610,493 |    317,215 |  3,579,931 |  5,230,384 |
| lighten              |        650,291 |        457,316 |    621,340 |    319,867 |  3,584,409 |  5,255,202 |
| color_dodge          |      1,893,900 |      1,661,131 |    780,556 |    679,996 |  5,151,488 |  9,634,447 |
| color_burn           |      1,983,069 |      1,733,063 |    861,359 |    708,962 |  5,007,181 |  9,617,102 |
| hard_light           |        847,644 |        515,544 |    734,878 |    370,307 |  3,442,584 |  7,034,417 |
| soft_light           |      2,692,075 |      2,438,440 |  1,225,756 |    986,403 |  5,900,415 | 11,941,630 |
| difference           |        684,321 |        464,034 |    632,797 |    326,285 |  3,936,284 |  5,718,776 |
| exclusion            |        545,884 |        425,168 |    618,645 |    316,235 |  3,842,131 |  6,082,393 |
| multiply             |        639,821 |        459,283 |    627,202 |    316,841 |  3,608,817 |  5,986,364 |
| hue                  |      3,526,237 |      3,229,611 |  1,705,745 |  1,413,791 |  7,517,902 | 13,716,827 |
| saturation           |      3,512,181 |      3,106,154 |  1,676,187 |  1,411,117 |  7,443,261 | 13,752,382 |
| color                |      3,015,127 |      2,657,101 |  1,431,804 |  1,115,800 |  6,070,058 | 10,537,391 |
| luminosity           |      2,990,071 |      2,539,697 |  1,351,584 |  1,090,402 |  6,124,294 | 10,488,916 |

*Destination* is faster in `tiny-skia`, because we're exiting immediately,
while Skia uses null blitter, so edges processing is still in place.

### anti-aliased fill

`fill_aa.rs`

| Test/Library         | tiny-skia SSE2 | tiny-skia AVX2 | Skia SSE2  | Skia AVX2  | cairo      | raqote     |
| -------------------- | -------------: | -------------: | ---------: | ---------: | ---------: | ---------: |
| fill                 |        723,757 |        603,640 |    495,359 |    348,326 |    538,842 |  1,520,830 |

### memset fill

`memset_fill.rs`

Shape filling by overwritting original pixels. No blending.

| Test/Library         | tiny-skia SSE2 | tiny-skia AVX2 | Skia SSE2  | Skia AVX2  | cairo      | raqote     |
| -------------------- | -------------: | -------------: | ---------: | ---------: | ---------: | ---------: |
| opaque               |         53,548 |         41,973 |     45,392 |     47,662 |     45,804 |  2,432,875 |
| source               |         54,013 |         47,622 |     50,433 |     50,663 |     48,839 |    678,795 |

### rectangle fill

`fill_rect.rs`

Fills a rectangle with a solid solid color.

| Mode/Library          | tiny-skia SSE2 | tiny-skia AVX2 | Skia SSE2  | Skia AVX2  | cairo      | raqote     |
| --------------------- | -------------: | -------------: | ---------: | ---------: | ---------: | ---------: |
| basic                 |        746,917 |        576,148 |    711,992 |    348,918 |    203,921 |  3,672,295 |
| with AA               |        807,427 |        637,603 |    735,547 |    371,368 |    192,178 |  2,087,047 |
| with AA and transform |        397,436 |        347,655 |    321,105 |    191,670 |    175,061 |    891,996 |

The last test simply fallbacks to path filling in Skia/tiny-skia.

Strangely, Skia is pretty slow in this task. Not sure why.

### canvas fill

`fill_all.rs`

Filling the whole canvas with a color.

| Mode/Library         | tiny-skia SSE2 | tiny-skia AVX2 | Skia SSE2  | Skia AVX2  | cairo      | raqote     |
| -------------------- | -------------: | -------------: | ---------: | ---------: | ---------: | ---------: |
| fill                 |         62,094 |         65,516 |  1,112,417 |    554,113 |    285,316 |     46,560 |

### spiral stroke

`spiral.rs`

A spiral has a lot of short horizontal strides which are not CPU-friendly
when a rendering backend is designed for long horizontal strides.

| Test/Library         | tiny-skia SSE2 | tiny-skia AVX2 | Skia SSE2  | Skia AVX2  | cairo      | raqote     |
| -------------------- | -------------: | -------------: | ---------: | ---------: | ---------: | ---------: |
| stroke               |      2,053,756 |      1,939,394 |  1,205,407 |  1,174,229 |  3,161,763 |  5,707,383 |

### hairline stroking

`hairline.rs`

Draws a large spiral using a subpixel stroke width.

| Test/Library | tiny-skia SSE2 | tiny-skia AVX2 | Skia SSE2  | Skia AVX2  | cairo      | raqote     |
| ------------ | -------------: | -------------: | ---------: | ---------: | ---------: | ---------: |
| no AA        |      1,494,375 |      1,452,121 |  1,022,641 |  1,027,160 |  2,469,764 |          - |
| with AA      |      3,953,877 |      3,670,914 |  1,846,240 |  1,868,304 | 13,457,819 |          - |

- `raqote` doesn't support hairline stroking.
- Not sure why `cairo` is so slow with AA.

### gradients

`gradients.rs`

<!-- this bench contains only the low quality tiny-skia/Skia pipeline results -->
<!-- find a way to force high quality pipeline in Skia -->

| Test/Library                         | tiny-skia SSE2 | tiny-skia AVX2 | Skia SSE2  | Skia AVX2  | cairo      | raqote     |
| ------------------------------------ | -------------: | -------------: | ---------: | ---------: | ---------: | ---------: |
| linear, two stops, pad               |      1,430,807 |      1,265,540 |    993,781 |    571,583 |  2,458,368 |  3,379,723 |
| linear, two stops, reflect           |      1,847,322 |      1,693,896 |  1,248,226 |    684,310 |  2,449,727 |  3,301,529 |
| linear, two stops, repeat            |      1,653,065 |      1,559,043 |  1,043,289 |    599,061 |  2,445,711 |  3,115,416 |
| linear, three stops, evenly spread   |      2,113,378 |      2,154,395 |  1,806,614 |    781,344 |  2,413,454 |  4,021,338 |
| linear, three stops, unevenly spread |      2,160,242 |      2,156,258 |  1,805,379 |    687,479 |  2,423,142 |  3,412,176 |
| simple radial                        |      2,373,564 |      2,355,357 |  2,050,437 |    805,376 |  4,704,141 |  5,531,178 |
| two point radial                     |      4,180,379 |      4,483,205 |  1,943,448 |  1,083,230 |  4,709,760 | 13,454,676 |

### pattern

`pattern.rs`

| Test/Library                | tiny-skia SSE2 | tiny-skia AVX2 | Skia SSE2  | Skia AVX2  | cairo       | raqote     |
| --------------------------- | -------------: | -------------: | ---------: | ---------: | ----------: | ---------: |
| plain (nearest, no ts)      |      3,604,568 |      3,426,429 |  1,315,079 |  1,122,982 |     785,550 |  1,865,327 |
| lq (bilinear, with ts)      |     10,744,786 |      9,396,174 |  4,484,023 |  2,646,523 |  17,612,685 | 24,906,379 |
| hq (bicubic/gauss, with ts) |     32,976,005 |     31,067,973 | 12,386,848 |  9,364,356 | 162,771,632 |          - |

Note that `raqote` doesn't support high quality filtering.

And yes, cairo is really that slow. Not sure why.

### png

`png_io.rs`

We are comparing raw `png` crate performance with `tiny-skia`
premultiplying/demultiplying code to check how much overhead it adds.

| Test/Library         | tiny-skia  | png        |
| -------------------- | ---------: | ---------: |
| Decode RGB           |    128,367 |     67,293 |
| Decode RGBA          |    109,752 |     90,793 |
| Encode RGBA          |    302,154 |    275,042 |

RGB (without alpha) is slower, since we have to decode an image into a RGB buffer,
then transform it into a RGBA buffer.

## Running benchmarks

We support only Linux. The benchmark may work on other OS'es, but it will require a lot of preperation
(building Skia and cairo).

You have to install cairo first and built Skia from sources (see below).

Run:

```sh
export SKIA_DIR="/path/to/skia"
export SKIA_LIB_DIR="/path/to/skia/out/Shared"
export LD_LIBRARY_PATH="/path/to/skia/out/Shared"
cargo bench
```

### Building Skia

You will need `git`, `clang`, `ninja` and Python 2.

On Windows, use `clang-cl` and `clang-cl++` for `cc` and `cxx` instead.

```sh
git clone https://skia.googlesource.com/skia.git
cd skia
git fetch --all
git checkout -b m85 origin/chrome/m85
python2 tools/git-sync-deps # this will download about 3 GiB of code
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
