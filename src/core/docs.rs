//! Root document structure.
use super::numbering::FNumbering;
use super::paragraph::FParagraph;
use super::settings::{FSectionProperties, FSettings};
use super::styles::FStyles;
use super::table::FTable;
use crate::FPackage;
/// Block-level content of the document body.
#[derive(Debug, Clone)]
pub enum FBlock {
    /// paragraph.
    Paragraph(FParagraph),
    /// table.
    Table(FTable),
}
/// The document body, preserving the order of blocks.
#[derive(Debug, Clone, Default)]
pub struct FBody {
    /// Ordered block-level content.
    pub blocks: Vec<FBlock>,
}
/// Root of the Word document data model.
#[derive(Debug, Clone, Default)]
pub struct FWord {
    /// Package-level parts that are not modeled explicitly.
    pub package: FPackage,
    /// Document body content.
    pub body: FBody,
    /// Style definitions.
    pub styles: FStyles,
    /// Numbering definitions.
    pub numbering: FNumbering,
    /// Document settings.
    pub settings: FSettings,
    /// Section properties of the document.
    pub section: FSectionProperties,
    /// Whether the document has been modified since it was loaded.
    pub dirty: bool,
}
impl FWord {
    /// Creates an empty in-memory Word document.
    pub fn new_empty() -> Self {
        FWord::default()
    }
    /// Loads a Word document from a file on disk.
    pub fn with_path<P: AsRef<std::path::Path>>(path: P) -> crate::FWordResult<Self> {
        let data = std::fs::read(path)?;
        Self::with_bytes(&data)
    }
    /// Parses a Word document from its raw `.docx` bytes.
    pub fn with_bytes(data: &[u8]) -> crate::FWordResult<Self> {
        crate::io::reader::parse_document(data)
    }
    /// Serializes the document back into `.docx` bytes.
    pub fn to_bytes(&self) -> crate::FWordResult<Vec<u8>> {
        crate::io::writer::serialize_document(self)
    }
    /// Marks the document as modified.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    /// Clears the modified flag.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
    /// Returns the plain text content of the document body.
    pub fn plain_text(&self) -> String {
        let mut text = String::new();
        for block in &self.body.blocks {
            match block {
                FBlock::Paragraph(p) => {
                    for run in &p.runs {
                        text.push_str(&run.text);
                    }
                    text.push('\n');
                }
                FBlock::Table(t) => {
                    for row in &t.rows {
                        for cell in &row.cells {
                            for p in &cell.paragraphs {
                                for run in &p.runs {
                                    text.push_str(&run.text);
                                }
                                text.push(' ');
                            }
                            text.push('|');
                        }
                        text.push('\n');
                    }
                }
            }
        }
        text
    }
    /// Returns the number of paragraphs in the document.
    pub fn paragraph_count(&self) -> usize {
        self.body
            .blocks
            .iter()
            .filter(|b| matches!(b, FBlock::Paragraph(_)))
            .count()
    }
    /// Returns the number of tables in the document.
    pub fn table_count(&self) -> usize {
        self.body
            .blocks
            .iter()
            .filter(|b| matches!(b, FBlock::Table(_)))
            .count()
    }
}
