//! Paragraph-level structures and formatting properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FParagraphAlignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
    Distribute,
}
/// Numbering properties for a paragraph (list membership).
#[derive(Debug, Clone, Default)]
pub struct FNumberingProperties {
    /// Level index (0-based).
    pub level: Option<u32>,
    /// Numbering instance identifier.
    pub num_id: Option<u32>,
}
/// Paragraph-level formatting.
#[derive(Debug, Clone, Default)]
pub struct FParagraphProperties {
    /// Style identifier referenced by the paragraph, e.g. `"Heading1"`.
    pub style_id: Option<String>,
    /// Paragraph alignment.
    pub alignment: FParagraphAlignment,
    /// Left indent in twips.
    pub indent_left: Option<i32>,
    /// Right indent in twips.
    pub indent_right: Option<i32>,
    /// First-line indent in twips.
    pub indent_first_line: Option<i32>,
    /// Hanging indent in twips.
    pub indent_hanging: Option<i32>,
    /// Spacing before the paragraph in twips.
    pub spacing_before: Option<i32>,
    /// Spacing after the paragraph in twips.
    pub spacing_after: Option<i32>,
    /// Line spacing in twips.
    pub line_spacing: Option<i32>,
    /// Line spacing rule (auto, exact, atLeast).
    pub line_spacing_rule: Option<FLineSpacingRule>,
    /// Numbering properties.
    pub numbering: Option<FNumberingProperties>,
    /// Outline level (0-8, for heading hierarchy).
    pub outline_level: Option<u32>,
    /// Contextual spacing (suppress space between same-style paragraphs).
    pub contextual_spacing: Option<bool>,
    /// Raw `<w:pPr>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Line spacing rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FLineSpacingRule {
    /// Automatic line spacing (default).
    #[default]
    Auto,
    /// Exact line spacing.
    Exact,
    /// At least this spacing.
    AtLeast,
}
/// A hyperlink within a paragraph.
#[derive(Debug, Clone, Default)]
pub struct FHyperlink {
    /// Relationship ID for external links.
    pub relationship_id: Option<String>,
    /// Anchor for internal links (bookmarks).
    pub anchor: Option<String>,
    /// Target URL (resolved from relationships).
    pub target: Option<String>,
    /// Display text.
    pub text: String,
    /// Whether this is an external link.
    pub is_external: bool,
}
/// A single paragraph inside a block container.
#[derive(Debug, Clone, Default)]
pub struct FParagraph {
    /// Ordered list of runs that compose this paragraph.
    pub runs: Vec<super::run::FRun>,
    /// Paragraph-level formatting.
    pub properties: FParagraphProperties,
    /// Hyperlinks contained in this paragraph.
    pub hyperlinks: Vec<FHyperlink>,
    /// Raw `<w:p>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
