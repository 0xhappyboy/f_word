//! Style definitions for the document.
use super::paragraph::FParagraphProperties;
use super::run::FRunProperties;
use std::collections::HashMap;
/// Style type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FStyleType {
    /// Paragraph style (default).
    #[default]
    Paragraph,
    /// Character style.
    Character,
    /// Table style.
    Table,
    /// Numbering style.
    Numbering,
}
/// A paragraph or character style definition.
#[derive(Debug, Clone, Default)]
pub struct FStyle {
    /// Style identifier as referenced by `w:pStyle` / `w:rStyle`.
    pub style_id: String,
    /// Human readable style name.
    pub name: Option<String>,
    /// Style type.
    pub style_type: FStyleType,
    /// Whether this is the default style of its type.
    pub is_default: bool,
    /// Identifier of the style this one is based on.
    pub based_on: Option<String>,
    /// Identifier of the next style to apply after this one.
    pub next: Option<String>,
    /// Linked style identifier.
    pub link: Option<String>,
    /// UI priority for style ordering.
    pub ui_priority: Option<u32>,
    /// Quick format flag (shows in formatting toolbar).
    pub quick_format: Option<bool>,
    /// Paragraph properties of the style, if it is a paragraph style.
    pub paragraph_properties: Option<FParagraphProperties>,
    /// Run properties of the style.
    pub run_properties: Option<FRunProperties>,
    /// Raw `<w:style>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// The styles part of the document.
#[derive(Debug, Clone, Default)]
pub struct FStyles {
    /// Style definitions, keyed by style identifier.
    pub styles: HashMap<String, FStyle>,
    /// Document default run properties.
    pub default_run_properties: Option<FRunProperties>,
    /// Document default paragraph properties.
    pub default_paragraph_properties: Option<FParagraphProperties>,
    /// Raw `<w:styles>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
