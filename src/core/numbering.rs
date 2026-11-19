//! Numbering (list) definitions for the document.
use std::collections::HashMap;
/// A single numbering level definition.
#[derive(Debug, Clone, Default)]
pub struct FNumberingLevel {
    /// Zero-based level index.
    pub level: u32,
    /// Numbering format, e.g. `"decimal"` or `"bullet"`.
    pub num_format: Option<String>,
    /// Level text template, e.g. `"%1."`.
    pub level_text: Option<String>,
    /// Start value of the level.
    pub start: Option<u32>,
    /// Alignment of the number.
    pub level_justification: Option<String>,
    /// Paragraph properties for this level (indentation).
    pub paragraph_properties: Option<super::paragraph::FParagraphProperties>,
    /// Run properties for the number text.
    pub run_properties: Option<super::run::FRunProperties>,
    /// Suffix after number (tab, space, nothing).
    pub suffix: Option<String>,
}
/// A numbering definition (`w:abstractNum`).
#[derive(Debug, Clone, Default)]
pub struct FAbstractNum {
    /// Identifier of the abstract numbering definition.
    pub abstract_num_id: u32,
    /// Name of the numbering definition.
    pub name: Option<String>,
    /// Multi-level type (singleLevel, multilevel, hybridMultilevel).
    pub multi_level_type: Option<String>,
    /// Style link for this numbering.
    pub style_link: Option<String>,
    /// Levels defined by this abstract numbering.
    pub levels: Vec<FNumberingLevel>,
}
/// A numbering instance (`w:num`) mapping to an abstract definition.
#[derive(Debug, Clone, Default)]
pub struct FNum {
    /// Identifier of the numbering instance.
    pub num_id: u32,
    /// Referenced abstract numbering identifier.
    pub abstract_num_id: u32,
}
/// The numbering part of the document.
#[derive(Debug, Clone, Default)]
pub struct FNumbering {
    /// Abstract numbering definitions, keyed by identifier.
    pub abstract_nums: HashMap<u32, FAbstractNum>,
    /// Numbering instances, keyed by identifier.
    pub nums: HashMap<u32, FNum>,
}
