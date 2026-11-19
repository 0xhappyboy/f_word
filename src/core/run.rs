//! Run-level structures and formatting properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FVerticalAlign {
    #[default]
    Baseline,
    Superscript,
    Subscript,
}
/// A unit of run-level formatting.
#[derive(Debug, Clone, Default)]
pub struct FRunProperties {
    /// Bold flag; `None` means unspecified (inherited).
    pub bold: Option<bool>,
    /// Italic flag; `None` means unspecified.
    pub italic: Option<bool>,
    /// Underline flag; `None` means unspecified.
    pub underline: Option<bool>,
    /// Strike-through flag; `None` means unspecified.
    pub strike: Option<bool>,
    /// Font size in half-points.
    pub size: Option<u32>,
    /// Font family name (for example `"Calibri"`).
    pub font_family: Option<String>,
    /// East Asian font family name.
    pub font_family_east_asia: Option<String>,
    /// Complex script font family name.
    pub font_family_cs: Option<String>,
    /// Foreground color as a hex string without `#` (for example `"FF0000"`).
    pub color: Option<String>,
    /// Highlight color name as defined by OOXML (for example `"yellow"`).
    pub highlight: Option<String>,
    /// All caps flag.
    pub caps: Option<bool>,
    /// Small caps flag.
    pub small_caps: Option<bool>,
    /// Hidden text flag.
    pub vanish: Option<bool>,
    /// Vertical alignment (superscript/subscript).
    pub vert_align: Option<FVerticalAlign>,
    /// Character spacing in twips.
    pub spacing: Option<i32>,
    /// Character scale percentage.
    pub scale: Option<u32>,
    /// Position offset in half-points (for raised/lowered text).
    pub position: Option<i32>,
    /// Complex script font size in half-points.
    pub size_cs: Option<u32>,
    /// Bold for complex scripts.
    pub bold_cs: Option<bool>,
    /// Italic for complex scripts.
    pub italic_cs: Option<bool>,
    /// Right-to-left flag.
    pub rtl: Option<bool>,
    /// Raw `<w:rPr>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// A run of text with uniform formatting.
#[derive(Debug, Clone, Default)]
pub struct FRun {
    /// Text content of the run.
    pub text: String,
    /// Formatting applied to the run.
    pub properties: FRunProperties,
    /// Raw `<w:r>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
