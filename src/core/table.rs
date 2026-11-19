//! Table-related structures.
use super::paragraph::FParagraph;
/// Vertical merge state of a table cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FVerticalMerge {
    /// The cell starts a merged region.
    Restart,
    /// The cell continues a merged region.
    Continue,
}
/// Horizontal alignment of cell content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FCellVerticalAlignment {
    #[default]
    Top,
    Center,
    Bottom,
}
/// Border style for table cells.
#[derive(Debug, Clone, Default)]
pub struct FBorder {
    /// Border style (single, double, dashed, etc.).
    pub style: Option<String>,
    /// Border size in eighths of a point.
    pub size: Option<u32>,
    /// Border color.
    pub color: Option<String>,
}
/// Cell borders (top, left, bottom, right).
#[derive(Debug, Clone, Default)]
pub struct FCellBorders {
    pub top: Option<FBorder>,
    pub left: Option<FBorder>,
    pub bottom: Option<FBorder>,
    pub right: Option<FBorder>,
}
/// Table cell properties.
#[derive(Debug, Clone, Default)]
pub struct FTableCellProperties {
    /// Cell width in twips.
    pub width: Option<u32>,
    /// Cell borders.
    pub borders: Option<FCellBorders>,
    /// Cell shading (background color).
    pub shading: Option<String>,
    /// Vertical alignment of content.
    pub vertical_alignment: Option<FCellVerticalAlignment>,
    /// No wrap flag.
    pub no_wrap: Option<bool>,
    /// Raw `<w:tcPr>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// A table cell.
#[derive(Debug, Clone, Default)]
pub struct FTableCell {
    /// Column span of the cell.
    pub grid_span: u32,
    /// Vertical merge state: `None` for a normal cell.
    pub vertical_merge: Option<FVerticalMerge>,
    /// Cell properties.
    pub properties: FTableCellProperties,
    /// Block-level content of the cell, normally one or more paragraphs.
    pub paragraphs: Vec<FParagraph>,
    /// Nested tables within this cell.
    pub tables: Vec<FTable>,
}
/// Table row properties.
#[derive(Debug, Clone, Default)]
pub struct FTableRowProperties {
    /// Row height in twips.
    pub height: Option<u32>,
    /// Height rule (auto, exact, atLeast).
    pub height_rule: Option<String>,
    /// Header row flag (repeat on each page).
    pub is_header: Option<bool>,
    /// Cannot split across pages.
    pub cant_split: Option<bool>,
    /// Raw `<w:trPr>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// A table row.
#[derive(Debug, Clone, Default)]
pub struct FTableRow {
    /// Cells contained in the row.
    pub cells: Vec<FTableCell>,
    /// Row properties.
    pub properties: FTableRowProperties,
    /// Raw `<w:tr>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Table properties.
#[derive(Debug, Clone, Default)]
pub struct FTableProperties {
    /// Table style identifier.
    pub style_id: Option<String>,
    /// Table width in twips or percentage.
    pub width: Option<u32>,
    /// Table width type (dxa, pct, auto).
    pub width_type: Option<String>,
    /// Table alignment.
    pub alignment: Option<String>,
    /// Table indentation.
    pub indent: Option<i32>,
    /// Table layout (fixed, autofit).
    pub layout: Option<String>,
    /// Raw `<w:tblPr>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// A table.
#[derive(Debug, Clone, Default)]
pub struct FTable {
    /// Rows of the table.
    pub rows: Vec<FTableRow>,
    /// Table properties.
    pub properties: FTableProperties,
    /// Column widths from tblGrid.
    pub grid_columns: Vec<u32>,
    /// Raw `<w:tbl>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
