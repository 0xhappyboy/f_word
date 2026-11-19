//! Document settings and section properties.
#[derive(Debug, Clone, Default)]
pub struct FSectionProperties {
    /// Page width in twips.
    pub page_width: Option<u32>,
    /// Page height in twips.
    pub page_height: Option<u32>,
    /// Page orientation (portrait, landscape).
    pub orientation: Option<String>,
    /// Top margin in twips.
    pub margin_top: Option<i32>,
    /// Bottom margin in twips.
    pub margin_bottom: Option<i32>,
    /// Left margin in twips.
    pub margin_left: Option<i32>,
    /// Right margin in twips.
    pub margin_right: Option<i32>,
    /// Header margin in twips.
    pub header_margin: Option<i32>,
    /// Footer margin in twips.
    pub footer_margin: Option<i32>,
    /// Gutter margin in twips.
    pub gutter: Option<i32>,
    /// Section type (nextPage, continuous, evenPage, oddPage).
    pub section_type: Option<String>,
    /// Number of columns.
    pub column_count: Option<u32>,
    /// Column spacing in twips.
    pub column_spacing: Option<u32>,
    /// Title page flag (different first page header/footer).
    pub title_page: Option<bool>,
    /// Even and odd headers flag.
    pub even_and_odd_headers: Option<bool>,
    /// Line numbering type.
    pub line_numbering: Option<String>,
    /// Page number type format.
    pub page_number_format: Option<String>,
    /// Page number start value.
    pub page_number_start: Option<u32>,
    /// Raw `<w:sectPr>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Document-level settings.
#[derive(Debug, Clone, Default)]
pub struct FSettings {
    /// Default tab stop in twips.
    pub default_tab_stop: Option<u32>,
    /// Even and odd headers flag.
    pub even_and_odd_headers: Option<bool>,
    /// Mirror margins flag.
    pub mirror_margins: Option<bool>,
    /// Track changes flag.
    pub track_changes: Option<bool>,
    /// Document protection settings.
    pub document_protection: Option<FDocumentProtection>,
    /// Compatibility mode.
    pub compat_mode: Option<String>,
    /// Theme font language.
    pub theme_font_language: Option<String>,
    /// Decimal symbol.
    pub decimal_symbol: Option<String>,
    /// List separator.
    pub list_separator: Option<String>,
    /// Raw `<w:settings>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Document protection settings.
#[derive(Debug, Clone, Default)]
pub struct FDocumentProtection {
    /// Protection type (readOnly, comments, trackedChanges, forms).
    pub protection_type: Option<String>,
    /// Enforcement flag.
    pub enforcement: Option<bool>,
    /// Cryptographic provider type.
    pub cryptographic_provider_type: Option<String>,
    /// Algorithm name.
    pub algorithm_name: Option<String>,
    /// Hash value.
    pub hash_value: Option<String>,
    /// Salt value.
    pub salt_value: Option<String>,
    /// Spin count.
    pub spin_count: Option<u32>,
}
