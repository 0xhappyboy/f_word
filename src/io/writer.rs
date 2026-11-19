//! Document serialization logic.
use crate::{
    FBlock, FBody, FNumbering, FParagraph, FParagraphAlignment, FParagraphProperties, FRun,
    FRunProperties, FSettings, FStyles, FTable, FTableCell, FTableRow, FVerticalMerge, FWord,
    core::{
        paragraph::{FHyperlink, FLineSpacingRule},
        run::FVerticalAlign,
        styles::FStyleType,
        table::{FCellVerticalAlignment, FTableProperties},
    },
    encode_xml_text,
};
use std::io::Write;
use zip::ZipWriter;
use zip::write::FileOptions;
/// XML declaration shared by every serialized part.
const XML_DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";
/// Package content types part.
const CONTENT_TYPES_XML: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
    "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">",
    "<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>",
    "<Default Extension=\"xml\" ContentType=\"application/xml\"/>",
    "<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>",
    "<Override PartName=\"/word/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/>",
    "<Override PartName=\"/word/numbering.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml\"/>",
    "<Override PartName=\"/word/settings.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml\"/>",
    "</Types>"
);
/// Package root relationships part.
const ROOT_RELS_XML: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
    "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>",
    "</Relationships>"
);
/// Main document relationships part.
const DOCUMENT_RELS_XML: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
    "</Relationships>"
);
/// Serializes a complete Word document to bytes.
pub(crate) fn serialize_document(doc: &FWord) -> crate::FWordResult<Vec<u8>> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = FileOptions::<()>::default();
    // Required package parts.
    writer.start_file("[Content_Types].xml", options)?;
    writer.write_all(CONTENT_TYPES_XML.as_bytes())?;
    writer.start_file("_rels/.rels", options)?;
    writer.write_all(ROOT_RELS_XML.as_bytes())?;
    // Main document part.
    let main_part = doc.package.main_part.as_str();
    let body_xml = serialize_body(&doc.body);
    writer.start_file(main_part, options)?;
    writer.write_all(body_xml.as_bytes())?;
    // Main document relationships.
    writer.start_file("word/_rels/document.xml.rels", options)?;
    writer.write_all(DOCUMENT_RELS_XML.as_bytes())?;
    // Styles.
    if !doc.styles.styles.is_empty() || doc.styles.raw_xml.is_some() {
        let xml = serialize_styles(&doc.styles);
        writer.start_file("word/styles.xml", options)?;
        writer.write_all(xml.as_bytes())?;
    }
    // Numbering.
    if !doc.numbering.abstract_nums.is_empty() || !doc.numbering.nums.is_empty() {
        let xml = serialize_numbering(&doc.numbering);
        writer.start_file("word/numbering.xml", options)?;
        writer.write_all(xml.as_bytes())?;
    }
    // Settings.
    if doc.settings.raw_xml.is_some() || doc.settings.default_tab_stop.is_some() {
        let xml = serialize_settings(&doc.settings);
        writer.start_file("word/settings.xml", options)?;
        writer.write_all(xml.as_bytes())?;
    }
    // Preserved parts.
    for part in &doc.package.parts {
        // Skip parts that were already written above.
        if part.path == "[Content_Types].xml"
            || part.path == "_rels/.rels"
            || part.path == main_part
            || part.path == "word/_rels/document.xml.rels"
        {
            continue;
        }
        writer.start_file(part.path.as_str(), options)?;
        writer.write_all(&part.data)?;
    }
    let cursor = writer.finish()?;
    Ok(cursor.into_inner())
}
/// Serializes the document body.
fn serialize_body(body: &FBody) -> String {
    let mut out = String::new();
    out.push_str(XML_DECL);
    out.push_str("<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">");
    out.push_str("<w:body>");
    for block in &body.blocks {
        match block {
            FBlock::Paragraph(p) => out.push_str(&serialize_paragraph(p)),
            FBlock::Table(t) => out.push_str(&serialize_table(t)),
        }
    }
    out.push_str("</w:body></w:document>");
    out
}
/// Serializes a paragraph.
fn serialize_paragraph(p: &FParagraph) -> String {
    let mut out = String::from("<w:p>");
    out.push_str(&serialize_paragraph_properties(&p.properties));
    // Serialize runs.
    for run in &p.runs {
        out.push_str(&serialize_run(run));
    }
    // Serialize hyperlinks.
    for link in &p.hyperlinks {
        out.push_str(&serialize_hyperlink(link));
    }
    out.push_str("</w:p>");
    out
}
/// Serializes paragraph properties.
fn serialize_paragraph_properties(props: &FParagraphProperties) -> String {
    let has_content = props.style_id.is_some()
        || props.alignment != FParagraphAlignment::Left
        || props.indent_left.is_some()
        || props.indent_right.is_some()
        || props.indent_first_line.is_some()
        || props.indent_hanging.is_some()
        || props.spacing_before.is_some()
        || props.spacing_after.is_some()
        || props.line_spacing.is_some()
        || props.numbering.is_some()
        || props.outline_level.is_some()
        || props.contextual_spacing.is_some();
    if !has_content {
        return String::new();
    }
    let mut out = String::from("<w:pPr>");
    if let Some(style) = &props.style_id {
        out.push_str(&format!("<w:pStyle w:val=\"{}\"/>", encode_xml_text(style)));
    }
    // Numbering.
    if let Some(num) = &props.numbering {
        out.push_str("<w:numPr>");
        if let Some(level) = num.level {
            out.push_str(&format!("<w:ilvl w:val=\"{}\"/>", level));
        }
        if let Some(num_id) = num.num_id {
            out.push_str(&format!("<w:numId w:val=\"{}\"/>", num_id));
        }
        out.push_str("</w:numPr>");
    }
    // Alignment.
    let align = match props.alignment {
        FParagraphAlignment::Left => None,
        FParagraphAlignment::Center => Some("center"),
        FParagraphAlignment::Right => Some("right"),
        FParagraphAlignment::Justify => Some("both"),
        FParagraphAlignment::Distribute => Some("distribute"),
    };
    if let Some(a) = align {
        out.push_str(&format!("<w:jc w:val=\"{}\"/>", a));
    }
    // Indentation.
    if props.indent_left.is_some()
        || props.indent_right.is_some()
        || props.indent_first_line.is_some()
        || props.indent_hanging.is_some()
    {
        out.push_str("<w:ind");
        if let Some(v) = props.indent_left {
            out.push_str(&format!(" w:left=\"{}\"", v));
        }
        if let Some(v) = props.indent_right {
            out.push_str(&format!(" w:right=\"{}\"", v));
        }
        if let Some(v) = props.indent_first_line {
            out.push_str(&format!(" w:firstLine=\"{}\"", v));
        }
        if let Some(v) = props.indent_hanging {
            out.push_str(&format!(" w:hanging=\"{}\"", v));
        }
        out.push_str("/>");
    }
    // Spacing.
    if props.spacing_before.is_some()
        || props.spacing_after.is_some()
        || props.line_spacing.is_some()
    {
        out.push_str("<w:spacing");
        if let Some(v) = props.spacing_before {
            out.push_str(&format!(" w:before=\"{}\"", v));
        }
        if let Some(v) = props.spacing_after {
            out.push_str(&format!(" w:after=\"{}\"", v));
        }
        if let Some(v) = props.line_spacing {
            out.push_str(&format!(" w:line=\"{}\"", v));
        }
        if let Some(rule) = props.line_spacing_rule {
            let rule_str = match rule {
                FLineSpacingRule::Auto => "auto",
                FLineSpacingRule::Exact => "exact",
                FLineSpacingRule::AtLeast => "atLeast",
            };
            out.push_str(&format!(" w:lineRule=\"{}\"", rule_str));
        }
        out.push_str("/>");
    }
    // Outline level.
    if let Some(level) = props.outline_level {
        out.push_str(&format!("<w:outlineLvl w:val=\"{}\"/>", level));
    }
    // Contextual spacing.
    if let Some(contextual) = props.contextual_spacing {
        if contextual {
            out.push_str("<w:contextualSpacing/>");
        }
    }
    out.push_str("</w:pPr>");
    out
}
/// Serializes a run.
fn serialize_run(run: &FRun) -> String {
    let mut out = String::from("<w:r>");
    out.push_str(&serialize_run_properties(&run.properties));
    let mut remaining = run.text.as_str();
    // Handle tabs.
    while let Some(pos) = remaining.find('\t') {
        if pos > 0 {
            out.push_str(&format!(
                "<w:t xml:space=\"preserve\">{}</w:t>",
                encode_xml_text(&remaining[..pos])
            ));
        }
        out.push_str("<w:tab/>");
        remaining = &remaining[pos + 1..];
    }
    // Handle line breaks.
    let mut remaining = remaining;
    while let Some(pos) = remaining.find('\n') {
        if pos > 0 {
            out.push_str(&format!(
                "<w:t xml:space=\"preserve\">{}</w:t>",
                encode_xml_text(&remaining[..pos])
            ));
        }
        out.push_str("<w:br/>");
        remaining = &remaining[pos + 1..];
    }
    // Handle page breaks (form feed).
    let mut remaining = remaining;
    while let Some(pos) = remaining.find('\u{000C}') {
        if pos > 0 {
            out.push_str(&format!(
                "<w:t xml:space=\"preserve\">{}</w:t>",
                encode_xml_text(&remaining[..pos])
            ));
        }
        out.push_str("<w:br w:type=\"page\"/>");
        remaining = &remaining[pos + 1..];
    }
    if !remaining.is_empty() {
        out.push_str(&format!(
            "<w:t xml:space=\"preserve\">{}</w:t>",
            encode_xml_text(remaining)
        ));
    }
    out.push_str("</w:r>");
    out
}
/// Serializes run properties.
fn serialize_run_properties(props: &FRunProperties) -> String {
    let has_content = props.bold.is_some()
        || props.italic.is_some()
        || props.underline.is_some()
        || props.strike.is_some()
        || props.size.is_some()
        || props.font_family.is_some()
        || props.color.is_some()
        || props.highlight.is_some()
        || props.caps.is_some()
        || props.small_caps.is_some()
        || props.vanish.is_some()
        || props.vert_align.is_some()
        || props.spacing.is_some()
        || props.scale.is_some()
        || props.position.is_some()
        || props.size_cs.is_some()
        || props.bold_cs.is_some()
        || props.italic_cs.is_some()
        || props.rtl.is_some()
        || props.font_family_east_asia.is_some()
        || props.font_family_cs.is_some();
    if !has_content {
        return String::new();
    }
    let mut out = String::from("<w:rPr>");
    // Fonts.
    if props.font_family.is_some()
        || props.font_family_east_asia.is_some()
        || props.font_family_cs.is_some()
    {
        out.push_str("<w:rFonts");
        if let Some(font) = &props.font_family {
            out.push_str(&format!(" w:ascii=\"{}\"", encode_xml_text(font)));
            out.push_str(&format!(" w:hAnsi=\"{}\"", encode_xml_text(font)));
        }
        if let Some(font_ea) = &props.font_family_east_asia {
            out.push_str(&format!(" w:eastAsia=\"{}\"", encode_xml_text(font_ea)));
        }
        if let Some(font_cs) = &props.font_family_cs {
            out.push_str(&format!(" w:cs=\"{}\"", encode_xml_text(font_cs)));
        }
        out.push_str("/>");
    }
    // Bold.
    if let Some(v) = props.bold {
        out.push_str(if v { "<w:b/>" } else { "<w:b w:val=\"0\"/>" });
    }
    // Bold complex script.
    if let Some(v) = props.bold_cs {
        out.push_str(if v {
            "<w:bCs/>"
        } else {
            "<w:bCs w:val=\"0\"/>"
        });
    }
    // Italic.
    if let Some(v) = props.italic {
        out.push_str(if v { "<w:i/>" } else { "<w:i w:val=\"0\"/>" });
    }
    // Italic complex script.
    if let Some(v) = props.italic_cs {
        out.push_str(if v {
            "<w:iCs/>"
        } else {
            "<w:iCs w:val=\"0\"/>"
        });
    }
    // Underline.
    if let Some(v) = props.underline {
        out.push_str(if v { "<w:u/>" } else { "<w:u w:val=\"0\"/>" });
    }
    // Strike.
    if let Some(v) = props.strike {
        out.push_str(if v {
            "<w:strike/>"
        } else {
            "<w:strike w:val=\"0\"/>"
        });
    }
    // Caps.
    if let Some(v) = props.caps {
        out.push_str(if v {
            "<w:caps/>"
        } else {
            "<w:caps w:val=\"0\"/>"
        });
    }
    // Small caps.
    if let Some(v) = props.small_caps {
        out.push_str(if v {
            "<w:smallCaps/>"
        } else {
            "<w:smallCaps w:val=\"0\"/>"
        });
    }
    // Vanish.
    if let Some(v) = props.vanish {
        out.push_str(if v {
            "<w:vanish/>"
        } else {
            "<w:vanish w:val=\"0\"/>"
        });
    }
    // Color.
    if let Some(color) = &props.color {
        out.push_str(&format!("<w:color w:val=\"{}\"/>", encode_xml_text(color)));
    }
    // Highlight.
    if let Some(highlight) = &props.highlight {
        out.push_str(&format!(
            "<w:highlight w:val=\"{}\"/>",
            encode_xml_text(highlight)
        ));
    }
    // Size.
    if let Some(v) = props.size {
        out.push_str(&format!("<w:sz w:val=\"{}\"/>", v));
    }
    // Size complex script.
    if let Some(v) = props.size_cs {
        out.push_str(&format!("<w:szCs w:val=\"{}\"/>", v));
    }
    // Vertical alignment.
    if let Some(vert) = props.vert_align {
        let val = match vert {
            FVerticalAlign::Baseline => "baseline",
            FVerticalAlign::Superscript => "superscript",
            FVerticalAlign::Subscript => "subscript",
        };
        out.push_str(&format!("<w:vertAlign w:val=\"{}\"/>", val));
    }
    // Spacing.
    if let Some(v) = props.spacing {
        out.push_str(&format!("<w:spacing w:val=\"{}\"/>", v));
    }
    // Scale.
    if let Some(v) = props.scale {
        out.push_str(&format!("<w:w w:val=\"{}\"/>", v));
    }
    // Position.
    if let Some(v) = props.position {
        out.push_str(&format!("<w:position w:val=\"{}\"/>", v));
    }
    // RTL.
    if let Some(v) = props.rtl {
        out.push_str(if v {
            "<w:rtl/>"
        } else {
            "<w:rtl w:val=\"0\"/>"
        });
    }
    out.push_str("</w:rPr>");
    out
}
/// Serializes a hyperlink.
fn serialize_hyperlink(link: &FHyperlink) -> String {
    let mut out = String::from("<w:hyperlink");
    if let Some(rid) = &link.relationship_id {
        out.push_str(&format!(" r:id=\"{}\"", encode_xml_text(rid)));
    }
    if let Some(anchor) = &link.anchor {
        out.push_str(&format!(" w:anchor=\"{}\"", encode_xml_text(anchor)));
    }
    out.push('>');
    // Serialize as a simple run with the link text.
    let run = FRun {
        text: link.text.clone(),
        properties: FRunProperties::default(),
        raw_xml: None,
    };
    out.push_str(&serialize_run(&run));
    out.push_str("</w:hyperlink>");
    out
}
/// Serializes a table.
fn serialize_table(table: &FTable) -> String {
    let mut out = String::from("<w:tbl>");
    // Table properties.
    out.push_str(&serialize_table_properties(&table.properties));
    // Table grid.
    if !table.grid_columns.is_empty() {
        out.push_str("<w:tblGrid>");
        for width in &table.grid_columns {
            out.push_str(&format!("<w:gridCol w:w=\"{}\"/>", width));
        }
        out.push_str("</w:tblGrid>");
    }
    // Rows.
    for row in &table.rows {
        out.push_str(&serialize_table_row(row));
    }
    out.push_str("</w:tbl>");
    out
}
/// Serializes table properties.
fn serialize_table_properties(props: &FTableProperties) -> String {
    let has_content = props.style_id.is_some()
        || props.width.is_some()
        || props.alignment.is_some()
        || props.indent.is_some()
        || props.layout.is_some();
    if !has_content {
        return String::new();
    }
    let mut out = String::from("<w:tblPr>");
    if let Some(style) = &props.style_id {
        out.push_str(&format!(
            "<w:tblStyle w:val=\"{}\"/>",
            encode_xml_text(style)
        ));
    }
    if let Some(width) = props.width {
        let width_type = props.width_type.as_deref().unwrap_or("dxa");
        out.push_str(&format!(
            "<w:tblW w:w=\"{}\" w:type=\"{}\"/>",
            width, width_type
        ));
    }
    if let Some(align) = &props.alignment {
        out.push_str(&format!("<w:jc w:val=\"{}\"/>", encode_xml_text(align)));
    }
    if let Some(indent) = props.indent {
        out.push_str(&format!("<w:tblInd w:w=\"{}\" w:type=\"dxa\"/>", indent));
    }
    if let Some(layout) = &props.layout {
        out.push_str(&format!(
            "<w:tblLayout w:type=\"{}\"/>",
            encode_xml_text(layout)
        ));
    }
    out.push_str("</w:tblPr>");
    out
}
/// Serializes a table row.
fn serialize_table_row(row: &FTableRow) -> String {
    let mut out = String::from("<w:tr>");
    // Row properties.
    let has_row_props = row.properties.height.is_some()
        || row.properties.is_header.is_some()
        || row.properties.cant_split.is_some();
    if has_row_props {
        out.push_str("<w:trPr>");
        if let Some(height) = row.properties.height {
            let rule = row.properties.height_rule.as_deref().unwrap_or("auto");
            out.push_str(&format!(
                "<w:trHeight w:val=\"{}\" w:hRule=\"{}\"/>",
                height, rule
            ));
        }
        if let Some(header) = row.properties.is_header {
            if header {
                out.push_str("<w:tblHeader/>");
            }
        }
        if let Some(cant_split) = row.properties.cant_split {
            if cant_split {
                out.push_str("<w:cantSplit/>");
            }
        }
        out.push_str("</w:trPr>");
    }
    // Cells.
    for cell in &row.cells {
        out.push_str(&serialize_table_cell(cell));
    }
    out.push_str("</w:tr>");
    out
}
/// Serializes a table cell.
fn serialize_table_cell(cell: &FTableCell) -> String {
    let mut out = String::from("<w:tc>");
    // Cell properties.
    let has_cell_props = cell.grid_span > 1
        || cell.vertical_merge.is_some()
        || cell.properties.width.is_some()
        || cell.properties.borders.is_some()
        || cell.properties.shading.is_some()
        || cell.properties.vertical_alignment.is_some()
        || cell.properties.no_wrap.is_some();
    if has_cell_props {
        out.push_str("<w:tcPr>");
        if let Some(width) = cell.properties.width {
            out.push_str(&format!("<w:tcW w:w=\"{}\" w:type=\"dxa\"/>", width));
        }
        if cell.grid_span > 1 {
            out.push_str(&format!("<w:gridSpan w:val=\"{}\"/>", cell.grid_span));
        }
        if let Some(merge) = cell.vertical_merge {
            let val = match merge {
                FVerticalMerge::Restart => "restart",
                FVerticalMerge::Continue => "continue",
            };
            out.push_str(&format!("<w:vMerge w:val=\"{}\"/>", val));
        }
        // Cell borders.
        if let Some(borders) = &cell.properties.borders {
            out.push_str("<w:tcBorders>");
            for (tag, border) in [
                ("w:top", &borders.top),
                ("w:left", &borders.left),
                ("w:bottom", &borders.bottom),
                ("w:right", &borders.right),
            ] {
                if let Some(b) = border {
                    out.push_str(&format!("<{}", tag));
                    if let Some(style) = &b.style {
                        out.push_str(&format!(" w:val=\"{}\"", encode_xml_text(style)));
                    }
                    if let Some(size) = b.size {
                        out.push_str(&format!(" w:sz=\"{}\"", size));
                    }
                    if let Some(color) = &b.color {
                        out.push_str(&format!(" w:color=\"{}\"", encode_xml_text(color)));
                    }
                    out.push_str("/>");
                }
            }
            out.push_str("</w:tcBorders>");
        }
        // Cell shading.
        if let Some(shading) = &cell.properties.shading {
            out.push_str(&format!(
                "<w:shd w:val=\"clear\" w:fill=\"{}\"/>",
                encode_xml_text(shading)
            ));
        }
        // Vertical alignment.
        if let Some(valign) = cell.properties.vertical_alignment {
            let val = match valign {
                FCellVerticalAlignment::Top => "top",
                FCellVerticalAlignment::Center => "center",
                FCellVerticalAlignment::Bottom => "bottom",
            };
            out.push_str(&format!("<w:vAlign w:val=\"{}\"/>", val));
        }
        // No wrap.
        if let Some(no_wrap) = cell.properties.no_wrap {
            if no_wrap {
                out.push_str("<w:noWrap/>");
            }
        }
        out.push_str("</w:tcPr>");
    }
    // Paragraphs.
    for p in &cell.paragraphs {
        out.push_str(&serialize_paragraph(p));
    }
    // Nested tables.
    for tbl in &cell.tables {
        out.push_str(&serialize_table(tbl));
    }
    out.push_str("</w:tc>");
    out
}
/// Serializes styles.
fn serialize_styles(styles: &FStyles) -> String {
    let mut out = String::new();
    out.push_str(XML_DECL);
    out.push_str(
        "<w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
    );
    // Document defaults.
    if styles.default_run_properties.is_some() || styles.default_paragraph_properties.is_some() {
        out.push_str("<w:docDefaults>");
        if let Some(rpr) = &styles.default_run_properties {
            out.push_str("<w:rPrDefault>");
            out.push_str(&serialize_run_properties(rpr));
            out.push_str("</w:rPrDefault>");
        }
        if let Some(ppr) = &styles.default_paragraph_properties {
            out.push_str("<w:pPrDefault>");
            out.push_str(&serialize_paragraph_properties(ppr));
            out.push_str("</w:pPrDefault>");
        }
        out.push_str("</w:docDefaults>");
    }
    // Style definitions.
    for style in styles.styles.values() {
        out.push_str(&format!(
            "<w:style w:type=\"{}\" w:styleId=\"{}\"",
            match style.style_type {
                FStyleType::Paragraph => "paragraph",
                FStyleType::Character => "character",
                FStyleType::Table => "table",
                FStyleType::Numbering => "numbering",
            },
            encode_xml_text(&style.style_id)
        ));
        if style.is_default {
            out.push_str(" w:default=\"1\"");
        }
        out.push('>');
        if let Some(name) = &style.name {
            out.push_str(&format!("<w:name w:val=\"{}\"/>", encode_xml_text(name)));
        }
        if let Some(based) = &style.based_on {
            out.push_str(&format!(
                "<w:basedOn w:val=\"{}\"/>",
                encode_xml_text(based)
            ));
        }
        if let Some(next) = &style.next {
            out.push_str(&format!("<w:next w:val=\"{}\"/>", encode_xml_text(next)));
        }
        if let Some(link) = &style.link {
            out.push_str(&format!("<w:link w:val=\"{}\"/>", encode_xml_text(link)));
        }
        if let Some(ui_priority) = style.ui_priority {
            out.push_str(&format!("<w:uiPriority w:val=\"{}\"/>", ui_priority));
        }
        if let Some(q_format) = style.quick_format {
            if q_format {
                out.push_str("<w:qFormat/>");
            }
        }
        if let Some(ppr) = &style.paragraph_properties {
            out.push_str(&serialize_paragraph_properties(ppr));
        }
        if let Some(rpr) = &style.run_properties {
            out.push_str(&serialize_run_properties(rpr));
        }
        out.push_str("</w:style>");
    }
    out.push_str("</w:styles>");
    out
}
/// Serializes numbering.
fn serialize_numbering(numbering: &FNumbering) -> String {
    let mut out = String::new();
    out.push_str(XML_DECL);
    out.push_str(
        "<w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
    );
    // Abstract numbering definitions.
    for abs in numbering.abstract_nums.values() {
        out.push_str(&format!(
            "<w:abstractNum w:abstractNumId=\"{}\">",
            abs.abstract_num_id
        ));
        if let Some(name) = &abs.name {
            out.push_str(&format!("<w:name w:val=\"{}\"/>", encode_xml_text(name)));
        }
        if let Some(mlt) = &abs.multi_level_type {
            out.push_str(&format!(
                "<w:multiLevelType w:val=\"{}\"/>",
                encode_xml_text(mlt)
            ));
        }
        if let Some(style_link) = &abs.style_link {
            out.push_str(&format!(
                "<w:styleLink w:val=\"{}\"/>",
                encode_xml_text(style_link)
            ));
        }
        for lvl in &abs.levels {
            out.push_str(&format!("<w:lvl w:ilvl=\"{}\">", lvl.level));
            if let Some(start) = lvl.start {
                out.push_str(&format!("<w:start w:val=\"{}\"/>", start));
            }
            if let Some(fmt) = &lvl.num_format {
                out.push_str(&format!("<w:numFmt w:val=\"{}\"/>", encode_xml_text(fmt)));
            }
            if let Some(txt) = &lvl.level_text {
                out.push_str(&format!("<w:lvlText w:val=\"{}\"/>", encode_xml_text(txt)));
            }
            if let Some(jc) = &lvl.level_justification {
                out.push_str(&format!("<w:lvlJc w:val=\"{}\"/>", encode_xml_text(jc)));
            }
            if let Some(suffix) = &lvl.suffix {
                out.push_str(&format!("<w:suff w:val=\"{}\"/>", encode_xml_text(suffix)));
            }
            if let Some(ppr) = &lvl.paragraph_properties {
                out.push_str(&serialize_paragraph_properties(ppr));
            }
            if let Some(rpr) = &lvl.run_properties {
                out.push_str(&serialize_run_properties(rpr));
            }
            out.push_str("</w:lvl>");
        }
        out.push_str("</w:abstractNum>");
    }
    // Numbering instances.
    for num in numbering.nums.values() {
        out.push_str(&format!("<w:num w:numId=\"{}\">", num.num_id));
        out.push_str(&format!(
            "<w:abstractNumId w:val=\"{}\"/>",
            num.abstract_num_id
        ));
        out.push_str("</w:num>");
    }
    out.push_str("</w:numbering>");
    out
}
/// Serializes settings.
fn serialize_settings(settings: &FSettings) -> String {
    let mut out = String::new();
    out.push_str(XML_DECL);
    out.push_str(
        "<w:settings xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
    );
    if let Some(tab) = settings.default_tab_stop {
        out.push_str(&format!("<w:defaultTabStop w:val=\"{}\"/>", tab));
    }
    if let Some(even_odd) = settings.even_and_odd_headers {
        if even_odd {
            out.push_str("<w:evenAndOddHeaders/>");
        }
    }
    if let Some(mirror) = settings.mirror_margins {
        if mirror {
            out.push_str("<w:mirrorMargins/>");
        }
    }
    if let Some(track) = settings.track_changes {
        if track {
            out.push_str("<w:trackChanges/>");
        }
    }
    if let Some(protection) = &settings.document_protection {
        out.push_str("<w:documentProtection");
        if let Some(prot_type) = &protection.protection_type {
            out.push_str(&format!(" w:edit=\"{}\"", encode_xml_text(prot_type)));
        }
        if let Some(enforcement) = protection.enforcement {
            out.push_str(&format!(
                " w:enforcement=\"{}\"",
                if enforcement { "1" } else { "0" }
            ));
        }
        out.push_str("/>");
    }
    if let Some(lang) = &settings.theme_font_language {
        out.push_str(&format!(
            "<w:themeFontLang w:val=\"{}\"/>",
            encode_xml_text(lang)
        ));
    }
    if let Some(decimal) = &settings.decimal_symbol {
        out.push_str(&format!(
            "<w:decimalSymbol w:val=\"{}\"/>",
            encode_xml_text(decimal)
        ));
    }
    if let Some(separator) = &settings.list_separator {
        out.push_str(&format!(
            "<w:listSeparator w:val=\"{}\"/>",
            encode_xml_text(separator)
        ));
    }
    out.push_str("</w:settings>");
    out
}
