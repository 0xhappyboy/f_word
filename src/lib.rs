//! Word document parsing engine.
mod core;
mod error;
mod io;
mod pkg;
mod xml;
pub use core::docs::{FBlock, FBody, FWord};
pub use core::numbering::{FAbstractNum, FNum, FNumbering, FNumberingLevel};
pub use core::paragraph::{
    FHyperlink, FLineSpacingRule, FNumberingProperties, FParagraph, FParagraphAlignment,
    FParagraphProperties,
};
pub use core::run::{FRun, FRunProperties, FVerticalAlign};
pub use core::settings::{FSectionProperties, FSettings};
pub use core::styles::{FStyle, FStyles};
pub use core::table::{
    FBorder, FCellBorders, FCellVerticalAlignment, FTable, FTableCell, FTableCellProperties,
    FTableProperties, FTableRow, FTableRowProperties, FVerticalMerge,
};
pub use error::{FWordError, FWordResult};
pub use io::*;
pub use pkg::{FPackage, FPackagePart};
pub use xml::*;
