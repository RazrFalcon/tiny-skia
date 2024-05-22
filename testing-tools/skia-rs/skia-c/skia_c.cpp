#include <assert.h>

#include <include/core/SkPathEffect.h>
#include <include/core/SkCanvas.h>
#include <include/core/SkData.h>
#include <include/core/SkGraphics.h>
#include <include/core/SkPaint.h>
#include <include/core/SkSurface.h>
#include <include/effects/SkDashPathEffect.h>
#include <include/effects/SkGradientShader.h>

#include <math.h>

#include "skia_c.hpp"

#define SURFACE_CAST reinterpret_cast<SkSurface*>(c_surface)
#define CANVAS_CAST reinterpret_cast<SkCanvas*>(c_canvas)
#define PAINT_CAST reinterpret_cast<SkPaint*>(c_paint)
#define PATH_CAST reinterpret_cast<SkPath*>(c_path)

extern "C" {

static SkMatrix conv_from_transform(const skiac_transform &c_ts)
{
    return SkMatrix::MakeAll(c_ts.a, c_ts.c, c_ts.e,
                             c_ts.b, c_ts.d, c_ts.f,
                             0, 0, 1);
}

static skiac_transform conv_to_transform(const SkMatrix &matrix)
{
    return skiac_transform {
        matrix.getScaleX(),
        matrix.getSkewY(), // Yes, inverted.
        matrix.getSkewX(), // Skia uses such order for some reasons.
        matrix.getScaleY(),
        matrix.getTranslateX(),
        matrix.getTranslateY(),
    };
}

// Surface

static SkSurface* skiac_surface_create(int width, int height, SkAlphaType alphaType)
{
    // Init() is idempotent, so can be called more than once with no adverse effect.
    SkGraphics::Init();

    auto info = SkImageInfo::Make(width, height, kRGBA_8888_SkColorType, alphaType);
    auto surface = SkSurface::MakeRaster(info);

    if (surface) {
        // The surface ref count will equal one after the pointer is returned.
        return surface.release();
    } else {
        return nullptr;
    }
}

skiac_surface* skiac_surface_create_rgba_premultiplied(int width, int height)
{
    return reinterpret_cast<skiac_surface*>(
        skiac_surface_create(width, height, kPremul_SkAlphaType));
}

skiac_surface* skiac_surface_create_rgba(int width, int height)
{
    return reinterpret_cast<skiac_surface*>(
        skiac_surface_create(width, height, kUnpremul_SkAlphaType));
}

bool skiac_surface_save(skiac_surface* c_surface, const char *path)
{
    sk_sp<SkImage> image = SURFACE_CAST->makeImageSnapshot();
    sk_sp<SkData> data = image->encodeToData(SkEncodedImageFormat::kPNG, 0);
    if (data) {
        SkFILEWStream stream(path);
        if (stream.write(data->data(), data->size())) {
            stream.flush();
            return true;
        }
    }

    return false;
}

void skiac_surface_destroy(skiac_surface* c_surface)
{
    // SkSurface is ref counted.
    SkSafeUnref(SURFACE_CAST);
}

skiac_surface* skiac_surface_copy_rgba(
    skiac_surface *c_surface,
    uint32_t x, uint32_t y, uint32_t width, uint32_t height)
{
    // x, y, width, height are source rectangle coordinates.
    auto copy = skiac_surface_create((int)width, (int)height, kUnpremul_SkAlphaType);
    if (!copy) {
        return nullptr;
    }

    SkPaint paint;
    paint.setFilterQuality(SkFilterQuality::kLow_SkFilterQuality);
    paint.setAlpha(SK_AlphaOPAQUE);

    // The original surface draws itself to the copy's canvas.
    SURFACE_CAST->draw(copy->getCanvas(), -(SkScalar)x, -(SkScalar)y, &paint);

    return reinterpret_cast<skiac_surface*>(copy);
}

int skiac_surface_get_width(skiac_surface* c_surface)
{
    return SURFACE_CAST->width();
}

int skiac_surface_get_height(skiac_surface* c_surface)
{
    return SURFACE_CAST->height();
}

skiac_canvas* skiac_surface_get_canvas(skiac_surface* c_surface)
{
    return reinterpret_cast<skiac_canvas*>(SURFACE_CAST->getCanvas());
}

void skiac_surface_read_pixels(skiac_surface* c_surface, skiac_surface_data* data)
{
    data->ptr = nullptr;
    data->size = 0;

    SkPixmap pixmap;
    if (SURFACE_CAST->peekPixels(&pixmap)) {
        data->ptr = static_cast<uint8_t*>(pixmap.writable_addr());
        data->size = static_cast<uint32_t>(pixmap.computeByteSize());
    }
}

int skiac_surface_get_alpha_type(skiac_surface *c_surface)
{
    return SURFACE_CAST->imageInfo().alphaType();
}

// Canvas

void skiac_canvas_clear(skiac_canvas* c_canvas, uint32_t color)
{
    CANVAS_CAST->clear(static_cast<SkColor>(color));
}

void skiac_canvas_flush(skiac_canvas* c_canvas)
{
    CANVAS_CAST->flush();
}

void skiac_canvas_set_transform(skiac_canvas* c_canvas, skiac_transform c_ts)
{
    CANVAS_CAST->setMatrix(conv_from_transform(c_ts));
}

void skiac_canvas_concat(skiac_canvas* c_canvas, skiac_transform c_ts)
{
    CANVAS_CAST->concat(conv_from_transform(c_ts));
}

void skiac_canvas_scale(skiac_canvas* c_canvas, float sx, float sy)
{
    CANVAS_CAST->scale(sx, sy);
}

void skiac_canvas_translate(skiac_canvas* c_canvas, float dx, float dy)
{
    CANVAS_CAST->translate(dx, dy);
}

skiac_transform skiac_canvas_get_total_transform(skiac_canvas* c_canvas)
{
    return conv_to_transform(CANVAS_CAST->getTotalMatrix());
}

void skiac_canvas_draw_color(skiac_canvas* c_canvas, float r, float g, float b, float a)
{
    CANVAS_CAST->drawColor(SkColor4f { r, g, b, a});
}

void skiac_canvas_draw_path(skiac_canvas* c_canvas, skiac_path* c_path, skiac_paint* c_paint)
{
    CANVAS_CAST->drawPath(*PATH_CAST, *PAINT_CAST);
}

void skiac_canvas_draw_rect(
    skiac_canvas* c_canvas,
    float x, float y, float w, float h,
    skiac_paint* c_paint)
{
    CANVAS_CAST->drawRect(SkRect::MakeXYWH(x, y, w, h), *PAINT_CAST);
}

void skiac_canvas_draw_surface(
    skiac_canvas* c_canvas,
    skiac_surface* c_surface,
    float left,
    float top,
    uint8_t alpha,
    int blend_mode,
    int filter_quality)
{
    auto image = SURFACE_CAST->makeImageSnapshot();
    SkPaint paint;
    paint.setFilterQuality((SkFilterQuality)filter_quality);
    paint.setAlpha(alpha);
    paint.setBlendMode((SkBlendMode)blend_mode);
    const auto sampling = SkSamplingOptions((SkFilterQuality)filter_quality);
    CANVAS_CAST->drawImage(image, left, top, sampling, &paint);
}

void skiac_canvas_draw_surface_rect(
    skiac_canvas* c_canvas,
    skiac_surface* c_surface,
    float x, float y, float w, float h,
    int filter_quality)
{
    auto image = SURFACE_CAST->makeImageSnapshot();
    SkPaint paint;
    paint.setFilterQuality((SkFilterQuality)filter_quality);
    auto src = SkRect::MakeXYWH(0, 0, image->width(), image->height());
    auto dst = SkRect::MakeXYWH(x, y, w, h);
    const auto sampling = SkSamplingOptions((SkFilterQuality)filter_quality);
    CANVAS_CAST->drawImageRect(image, src, dst, sampling, &paint, SkCanvas::kFast_SrcRectConstraint);
}

void skiac_canvas_reset_transform(skiac_canvas* c_canvas)
{
    CANVAS_CAST->resetMatrix();
}

void skiac_canvas_clip_rect(skiac_canvas* c_canvas, float x, float y, float w, float h, bool aa)
{
    auto rect = SkRect::MakeXYWH(x, y, w, h);
    CANVAS_CAST->clipRect(rect, aa);
}

void skiac_canvas_clip_path(skiac_canvas* c_canvas, skiac_path* c_path, bool aa)
{
    CANVAS_CAST->clipPath(*PATH_CAST, aa);
}

void skiac_canvas_save(skiac_canvas* c_canvas)
{
    CANVAS_CAST->save();
}

void skiac_canvas_restore(skiac_canvas* c_canvas)
{
    CANVAS_CAST->restore();
}

// Paint

skiac_paint* skiac_paint_create()
{
    return reinterpret_cast<skiac_paint*>(new SkPaint());
}

void skiac_paint_destroy(skiac_paint* c_paint)
{
    // Will unref() Shader and PathEffect.

    // SkPaint is not ref counted, so explicitly delete.
    delete PAINT_CAST;
}

void skiac_paint_set_color(skiac_paint* c_paint, uint8_t r, uint8_t g, uint8_t b, uint8_t a)
{
    PAINT_CAST->setARGB(a, r, g, b);
}

void skiac_paint_set_alpha(skiac_paint* c_paint, uint8_t a)
{
    PAINT_CAST->setAlpha(a);
}

void skiac_paint_set_anti_alias(skiac_paint* c_paint, bool aa)
{
    PAINT_CAST->setAntiAlias(aa);
}

void skiac_paint_set_blend_mode(skiac_paint* c_paint, int blend_mode)
{
    PAINT_CAST->setBlendMode((SkBlendMode)blend_mode);
}

void skiac_paint_set_shader(skiac_paint* c_paint, skiac_shader* c_shader)
{
    sk_sp<SkShader> shader(reinterpret_cast<SkShader*>(c_shader));

    // setShader accepts a smart pointer which will be destructed on delete.
    // Therefore we have to reference the object once more, to keep it valid in Rust.
    shader->ref();

    PAINT_CAST->setShader(shader);
}

void skiac_paint_set_path_effect(skiac_paint* c_paint, skiac_path_effect* c_path_effect)
{
    sk_sp<SkPathEffect> pathEffect(reinterpret_cast<SkPathEffect*>(c_path_effect));

    // setPathEffect accepts a smart pointer which will be destructed on delete.
    // Therefore we have to reference the object once more, to keep it valid in Rust.
    pathEffect->ref();

    PAINT_CAST->setPathEffect(pathEffect);
}

void skiac_paint_set_style(skiac_paint* c_paint, int style)
{
    PAINT_CAST->setStyle((SkPaint::Style)style);
}

void skiac_paint_set_stroke_width(skiac_paint* c_paint, float width)
{
    PAINT_CAST->setStrokeWidth(width);
}

void skiac_paint_set_stroke_cap(skiac_paint* c_paint, int cap)
{
    PAINT_CAST->setStrokeCap((SkPaint::Cap)cap);
}

void skiac_paint_set_stroke_join(skiac_paint* c_paint, int join)
{
    PAINT_CAST->setStrokeJoin((SkPaint::Join)join);
}

void skiac_paint_set_stroke_miter(skiac_paint* c_paint, float miter)
{
    PAINT_CAST->setStrokeMiter(miter);
}

// Path

skiac_path* skiac_path_create()
{
    return reinterpret_cast<skiac_path*>(new SkPath());
}

void skiac_path_destroy(skiac_path* c_path)
{
    // SkPath is NOT ref counted
    delete PATH_CAST;
}

void skiac_path_set_fill_type(skiac_path* c_path, int type)
{
    PATH_CAST->setFillType((SkPathFillType)type);
}

void skiac_path_move_to(skiac_path* c_path, float x, float y)
{
    PATH_CAST->moveTo(x, y);
}

void skiac_path_line_to(skiac_path* c_path, float x, float y)
{
    PATH_CAST->lineTo(x, y);
}

void skiac_path_cubic_to(
    skiac_path* c_path,
    float x1, float y1, float x2, float y2, float x3, float y3)
{
    PATH_CAST->cubicTo(x1, y1, x2, y2, x3, y3);
}

void skiac_path_close(skiac_path* c_path)
{
    PATH_CAST->close();
}

void skiac_path_add_rect(skiac_path* c_path, float l, float t, float r, float b)
{
    PATH_CAST->addRect(l, t, r, b);
}

void skiac_path_add_circle(skiac_path* c_path, float x, float y, float r)
{
    PATH_CAST->addCircle(x, y, r);
}

// PathEffect

skiac_path_effect* skiac_path_effect_make_dash_path(const float* intervals, int count, float phase)
{
    auto effect = SkDashPathEffect::Make(intervals, count, phase).release();
    if (effect) {
        return reinterpret_cast<skiac_path_effect*>(effect);
    } else {
        return nullptr;
    }
}

void skiac_path_effect_destroy(skiac_path_effect* c_path_effect)
{
    // SkPathEffect is ref counted.
    auto effect = reinterpret_cast<SkPathEffect*>(c_path_effect);
    SkSafeUnref(effect);
}

// Shader

skiac_shader* skiac_shader_make_linear_gradient(
    const skiac_point* c_points,
    const uint32_t* colors,
    const float* positions,
    int count,
    int tile_mode,
    uint32_t flags,
    skiac_transform c_ts)
{
    const auto points = reinterpret_cast<const SkPoint*>(c_points);
    const auto skia_tile_mode = (SkTileMode)tile_mode;
    const auto ts = conv_from_transform(c_ts);
    auto shader = SkGradientShader::MakeLinear(
        points,
        colors,
        positions,
        count,
        skia_tile_mode,
        flags,
        &ts
    ).release();

    if (shader) {
        return reinterpret_cast<skiac_shader*>(shader);
    } else {
        return nullptr;
    }
}

skiac_shader* skiac_shader_make_two_point_conical_gradient(
    skiac_point c_start_point,
    float start_radius,
    skiac_point c_end_point,
    float end_radius,
    const uint32_t* colors,
    const float* positions,
    int count,
    int tile_mode,
    uint32_t flags,
    skiac_transform c_ts)
{
    const SkPoint startPoint = { c_start_point.x, c_start_point.y };
    const SkPoint endPoint = { c_end_point.x, c_end_point.y };
    const auto ts = conv_from_transform(c_ts);
    auto shader = SkGradientShader::MakeTwoPointConical(
        startPoint,
        start_radius,
        endPoint,
        end_radius,
        colors,
        positions,
        count,
        (SkTileMode)tile_mode,
        flags,
        &ts
    ).release();

    if (shader) {
        return reinterpret_cast<skiac_shader*>(shader);
    } else {
        return nullptr;
    }
}

skiac_shader* skiac_shader_make_from_surface_image(
    skiac_surface* c_surface,
    skiac_transform c_ts,
    int filter_quality)
{
    const auto skia_tile_mode = SkTileMode::kRepeat;
    const auto ts = conv_from_transform(c_ts);
    const auto sampling = SkSamplingOptions((SkFilterQuality)filter_quality);
    sk_sp<SkImage> image = SURFACE_CAST->makeImageSnapshot();
    auto shader = image->makeShader(
        skia_tile_mode,
        skia_tile_mode,
        sampling,
        &ts
    ).release();

    if (shader) {
        return reinterpret_cast<skiac_shader*>(shader);
    } else {
        return nullptr;
    }
}

void skiac_shader_destroy(skiac_shader* c_shader)
{
    // SkShader is ref counted.
    auto shader = reinterpret_cast<SkShader*>(c_shader);
    SkSafeUnref(shader);
}

}
