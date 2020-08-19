use crate::raster_pipeline::{self, RasterPipelineBuilder};

/// A blending mode.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum BlendMode {
    /// Replaces destination with zero: fully transparent.
    Clear,
    /// Replaces destination.
    Source,
    /// Preserves destination.
    Destination,
    /// Source over destination.
    SourceOver,
    /// Destination over source.
    DestinationOver,
    /// Source trimmed inside destination.
    SourceIn,
    /// Destination trimmed by source.
    DestinationIn,
    /// Source trimmed outside destination.
    SourceOut,
    /// Destination trimmed outside source.
    DestinationOut,
    /// Source inside destination blended with destination.
    SourceATop,
    /// Destination inside source blended with source.
    DestinationATop,
    /// Each of source and destination trimmed outside the other.
    Xor,
    /// Sum of colors.
    Plus,
    /// Product of premultiplied colors; darkens destination.
    Modulate,
    /// Multiply inverse of pixels, inverting result; brightens destination.
    Screen,
    /// Multiply or screen, depending on destination.
    Overlay,
    /// Darker of source and destination.
    Darken,
    /// Lighter of source and destination.
    Lighten,
    /// Brighten destination to reflect source.
    ColorDodge,
    /// Darken destination to reflect source.
    ColorBurn,
    /// Multiply or screen, depending on source.
    HardLight,
    /// Lighten or darken, depending on source.
    SoftLight,
    /// Subtract darker from lighter with higher contrast.
    Difference,
    /// Subtract darker from lighter with lower contrast.
    Exclusion,
    /// Multiply source with destination, darkening image.
    Multiply,
    /// hue of source with saturation and luminosity of destination
    Hue,
    /// saturation of source with hue and luminosity of destination
    Saturation,
    ///. hue and saturation of source with luminosity of destination
    Color,
    /// luminosity of source with hue and saturation of destination
    Luminosity,
}

impl Default for BlendMode {
    #[inline]
    fn default() -> Self {
        BlendMode::SourceOver
    }
}

impl BlendMode {
    #[inline]
    pub(crate) fn push_stages(self, p: &mut RasterPipelineBuilder) {
        p.push(match self {
            BlendMode::Clear            => raster_pipeline::Stage::Clear,
            BlendMode::Source           => return, // This stage is a no-op.
            BlendMode::Destination      => raster_pipeline::Stage::MoveDestinationToSource,
            BlendMode::SourceOver       => raster_pipeline::Stage::SourceOver,
            BlendMode::DestinationOver  => raster_pipeline::Stage::DestinationOver,
            BlendMode::SourceIn         => raster_pipeline::Stage::SourceIn,
            BlendMode::DestinationIn    => raster_pipeline::Stage::DestinationIn,
            BlendMode::SourceOut        => raster_pipeline::Stage::SourceOut,
            BlendMode::DestinationOut   => raster_pipeline::Stage::DestinationOut,
            BlendMode::SourceATop       => raster_pipeline::Stage::SourceATop,
            BlendMode::DestinationATop  => raster_pipeline::Stage::DestinationATop,
            BlendMode::Xor              => raster_pipeline::Stage::Xor,
            BlendMode::Plus             => raster_pipeline::Stage::Plus,
            BlendMode::Modulate         => raster_pipeline::Stage::Modulate,
            BlendMode::Screen           => raster_pipeline::Stage::Screen,
            BlendMode::Overlay          => raster_pipeline::Stage::Overlay,
            BlendMode::Darken           => raster_pipeline::Stage::Darken,
            BlendMode::Lighten          => raster_pipeline::Stage::Lighten,
            BlendMode::ColorDodge       => raster_pipeline::Stage::ColorDodge,
            BlendMode::ColorBurn        => raster_pipeline::Stage::ColorBurn,
            BlendMode::HardLight        => raster_pipeline::Stage::HardLight,
            BlendMode::SoftLight        => raster_pipeline::Stage::SoftLight,
            BlendMode::Difference       => raster_pipeline::Stage::Difference,
            BlendMode::Exclusion        => raster_pipeline::Stage::Exclusion,
            BlendMode::Multiply         => raster_pipeline::Stage::Multiply,
            BlendMode::Hue              => raster_pipeline::Stage::Hue,
            BlendMode::Saturation       => raster_pipeline::Stage::Saturation,
            BlendMode::Color            => raster_pipeline::Stage::Color,
            BlendMode::Luminosity       => raster_pipeline::Stage::Luminosity,
        })
    }
}
