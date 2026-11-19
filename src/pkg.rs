//! Package-level structures for the OOXML archive.
#[derive(Debug, Clone, Default)]
pub struct FPackagePart {
    /// Archive-relative path.
    pub path: String,
    /// Raw bytes of the part.
    pub data: Vec<u8>,
}
/// the entry point relationships and every part that was not parsed into a dedicated structure.
#[derive(Debug, Clone, Default)]
pub struct FPackage {
    pub main_part: String,
    pub parts: Vec<FPackagePart>,
}
impl FPackage {
    /// Returns the bytes of a part by archive path, if present.
    pub fn get_part(&self, path: &str) -> Option<&[u8]> {
        self.parts
            .iter()
            .find(|p| p.path == path)
            .map(|p| p.data.as_slice())
    }
    /// Inserts or replaces a part.
    pub fn set_part(&mut self, path: &str, data: Vec<u8>) {
        if let Some(existing) = self.parts.iter_mut().find(|p| p.path == path) {
            existing.data = data;
        } else {
            self.parts.push(FPackagePart {
                path: path.to_string(),
                data,
            });
        }
    }
    /// Removes a part by archive path.
    pub fn remove_part(&mut self, path: &str) {
        self.parts.retain(|p| p.path != path);
    }
}
