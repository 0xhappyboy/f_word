//! Document parsing logic.
use crate::core::paragraph::{FHyperlink, FLineSpacingRule, FNumberingProperties};
use crate::core::run::FVerticalAlign;
use crate::core::settings::FDocumentProtection;
use crate::core::styles::FStyleType;
use crate::core::table::{
    FBorder, FCellBorders, FCellVerticalAlignment, FTableCellProperties, FTableProperties,
    FTableRowProperties,
};
use crate::{
    FAbstractNum, FBlock, FBody, FNum, FNumbering, FNumberingLevel, FPackagePart, FParagraph,
    FParagraphAlignment, FParagraphProperties, FRun, FRunProperties, FSettings, FStyle, FStyles,
    FTable, FTableCell, FTableRow, FVerticalMerge, FWord, FWordError, collect_elements,
    decode_xml_text, extract_element_inner, extract_element_with_span, extract_first_attr,
    flag_present,
};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use zip::read::ZipArchive;
/// Parses a complete Word document from raw bytes.
pub(crate) fn parse_document(data: &[u8]) -> crate::FWordResult<FWord> {
    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)?;
    let mut doc = FWord::default();
    let mut main_part = String::from("word/document.xml");
    let mut parts: Vec<FPackagePart> = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().unwrap_or_default().to_string();
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        match name.as_str() {
            "word/document.xml" => {
                main_part = name.clone();
                doc.body = parse_body(&buf)?;
            }
            "word/styles.xml" => {
                doc.styles = parse_styles(&buf)?;
            }
            "word/numbering.xml" => {
                doc.numbering = parse_numbering(&buf)?;
            }
            "word/settings.xml" => {
                doc.settings = parse_settings(&buf)?;
            }
            _ => {
                parts.push(FPackagePart {
                    path: name,
                    data: buf,
                });
            }
        }
    }
    doc.package.main_part = main_part;
    doc.package.parts = parts;
    doc.dirty = false;
    Ok(doc)
}
/// Parses the document body from XML bytes.
fn parse_body(xml: &[u8]) -> crate::FWordResult<FBody> {
    let text = std::str::from_utf8(xml)
        .map_err(|e| FWordError::Xml(format!("document.xml is not valid utf-8: {}", e)))?;
    let mut body = FBody::default();
    let body_inner = extract_element_inner(text, "w:body").unwrap_or_default();
    let mut cursor = 0usize;
    while cursor < body_inner.len() {
        let rest = &body_inner[cursor..];
        let p_pos = rest.find("<w:p").map(|v| (v, "w:p"));
        let t_pos = rest.find("<w:tbl").map(|v| (v, "w:tbl"));
        let next = match (p_pos, t_pos) {
            (Some(a), Some(b)) => Some(if a.0 <= b.0 { a } else { b }),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        let (offset, kind) = match next {
            Some(v) => v,
            None => break,
        };
        cursor += offset;
        let (inner, consumed) = match extract_element_with_span(&body_inner[cursor..], kind) {
            Some(v) => v,
            None => break,
        };
        cursor += consumed;
        match kind {
            "w:p" => body.blocks.push(FBlock::Paragraph(parse_paragraph(&inner))),
            "w:tbl" => body.blocks.push(FBlock::Table(parse_table(&inner))),
            _ => {}
        }
    }
    Ok(body)
}
/// Parses a paragraph element.
fn parse_paragraph(xml: &str) -> FParagraph {
    let mut paragraph = FParagraph::default();
    if let Some(ppr) = extract_element_inner(xml, "w:pPr") {
        paragraph.properties = parse_paragraph_properties(&ppr);
    }
    // Parse hyperlinks
    for link_xml in collect_elements(xml, "w:hyperlink") {
        if let Some(link) = parse_hyperlink(&link_xml) {
            paragraph.hyperlinks.push(link);
        }
    }
    // Parse runs
    for run_xml in collect_elements(xml, "w:r") {
        paragraph.runs.push(parse_run(&run_xml));
    }
    paragraph.raw_xml = Some(xml.to_string());
    paragraph
}
/// Parses a hyperlink element.
fn parse_hyperlink(xml: &str) -> Option<FHyperlink> {
    let relationship_id = extract_first_attr(xml, "w:hyperlink", "r:id");
    let anchor = extract_first_attr(xml, "w:hyperlink", "w:anchor");
    let mut text = String::new();
    for run_xml in collect_elements(xml, "w:r") {
        let run = parse_run(&run_xml);
        text.push_str(&run.text);
    }
    Some(FHyperlink {
        relationship_id: relationship_id.clone(),
        anchor,
        target: None,
        text,
        is_external: relationship_id.is_some(),
    })
}
/// Parses paragraph properties.
fn parse_paragraph_properties(xml: &str) -> FParagraphProperties {
    let mut props = FParagraphProperties::default();
    if let Some(style) = extract_first_attr(xml, "w:pStyle", "w:val") {
        props.style_id = Some(style);
    }
    if let Some(align) = extract_first_attr(xml, "w:jc", "w:val") {
        props.alignment = match align.as_str() {
            "center" => FParagraphAlignment::Center,
            "right" => FParagraphAlignment::Right,
            "both" => FParagraphAlignment::Justify,
            "distribute" => FParagraphAlignment::Distribute,
            _ => FParagraphAlignment::Left,
        };
    }
    // Parse indentation
    if let Some(ind) = extract_first_attr(xml, "w:ind", "w:left") {
        props.indent_left = ind.parse().ok();
    }
    if let Some(ind) = extract_first_attr(xml, "w:ind", "w:right") {
        props.indent_right = ind.parse().ok();
    }
    if let Some(ind) = extract_first_attr(xml, "w:ind", "w:firstLine") {
        props.indent_first_line = ind.parse().ok();
    }
    if let Some(ind) = extract_first_attr(xml, "w:ind", "w:hanging") {
        props.indent_hanging = ind.parse().ok();
    }
    // Parse spacing
    if let Some(spacing) = extract_first_attr(xml, "w:spacing", "w:before") {
        props.spacing_before = spacing.parse().ok();
    }
    if let Some(spacing) = extract_first_attr(xml, "w:spacing", "w:after") {
        props.spacing_after = spacing.parse().ok();
    }
    if let Some(spacing) = extract_first_attr(xml, "w:spacing", "w:line") {
        props.line_spacing = spacing.parse().ok();
    }
    // Parse line spacing rule
    if let Some(rule) = extract_first_attr(xml, "w:spacing", "w:lineRule") {
        props.line_spacing_rule = match rule.as_str() {
            "exact" => Some(FLineSpacingRule::Exact),
            "atLeast" => Some(FLineSpacingRule::AtLeast),
            _ => Some(FLineSpacingRule::Auto),
        };
    }
    // Parse numbering properties
    if let Some(numpr) = extract_element_inner(xml, "w:numPr") {
        let mut numbering = FNumberingProperties::default();
        if let Some(ilvl) = extract_first_attr(&numpr, "w:ilvl", "w:val") {
            numbering.level = ilvl.parse().ok();
        }
        if let Some(numid) = extract_first_attr(&numpr, "w:numId", "w:val") {
            numbering.num_id = numid.parse().ok();
        }
        props.numbering = Some(numbering);
    }
    // Parse outline level
    if let Some(outline) = extract_first_attr(xml, "w:outlineLvl", "w:val") {
        props.outline_level = outline.parse().ok();
    }
    // Parse contextual spacing
    if let Some(contextual) = flag_present(xml, "w:contextualSpacing") {
        props.contextual_spacing = Some(contextual);
    }
    props.raw_xml = Some(xml.to_string());
    props
}
/// Parses a run element.
fn parse_run(xml: &str) -> FRun {
    let mut run = FRun::default();
    if let Some(rpr) = extract_element_inner(xml, "w:rPr") {
        run.properties = parse_run_properties(&rpr);
    }
    // Parse text content
    for text_xml in collect_elements(xml, "w:t") {
        run.text.push_str(&decode_xml_text(&text_xml));
    }
    // Parse tabs
    for _ in collect_elements(xml, "w:tab") {
        run.text.push('\t');
    }
    // Parse line breaks
    for _ in collect_elements(xml, "w:br") {
        run.text.push('\n');
    }
    // Parse page breaks
    for br_xml in collect_elements(xml, "w:br") {
        if let Some(br_type) = extract_first_attr(&br_xml, "w:br", "w:type") {
            if br_type == "page" {
                run.text.push('\u{000C}'); // Form feed for page break
            }
        }
    }
    run.raw_xml = Some(xml.to_string());
    run
}
/// Parses run properties.
fn parse_run_properties(xml: &str) -> FRunProperties {
    let mut props = FRunProperties::default();
    props.bold = flag_present(xml, "w:b");
    props.italic = flag_present(xml, "w:i");
    props.underline = flag_present(xml, "w:u");
    props.strike = flag_present(xml, "w:strike");
    props.caps = flag_present(xml, "w:caps");
    props.small_caps = flag_present(xml, "w:smallCaps");
    props.vanish = flag_present(xml, "w:vanish");
    props.bold_cs = flag_present(xml, "w:bCs");
    props.italic_cs = flag_present(xml, "w:iCs");
    props.rtl = flag_present(xml, "w:rtl");
    if let Some(sz) = extract_first_attr(xml, "w:sz", "w:val") {
        props.size = sz.parse().ok();
    }
    if let Some(sz_cs) = extract_first_attr(xml, "w:szCs", "w:val") {
        props.size_cs = sz_cs.parse().ok();
    }
    if let Some(font) = extract_first_attr(xml, "w:rFonts", "w:ascii") {
        props.font_family = Some(font);
    }
    if let Some(font_ea) = extract_first_attr(xml, "w:rFonts", "w:eastAsia") {
        props.font_family_east_asia = Some(font_ea);
    }
    if let Some(font_cs) = extract_first_attr(xml, "w:rFonts", "w:hAnsi") {
        props.font_family_cs = Some(font_cs);
    }
    if let Some(color) = extract_first_attr(xml, "w:color", "w:val") {
        props.color = Some(color);
    }
    if let Some(highlight) = extract_first_attr(xml, "w:highlight", "w:val") {
        props.highlight = Some(highlight);
    }
    // Parse vertical alignment
    if let Some(vert) = extract_first_attr(xml, "w:vertAlign", "w:val") {
        props.vert_align = match vert.as_str() {
            "superscript" => Some(FVerticalAlign::Superscript),
            "subscript" => Some(FVerticalAlign::Subscript),
            _ => Some(FVerticalAlign::Baseline),
        };
    }
    // Parse character spacing
    if let Some(spacing) = extract_first_attr(xml, "w:spacing", "w:val") {
        props.spacing = spacing.parse().ok();
    }
    // Parse character scale
    if let Some(scale) = extract_first_attr(xml, "w:w", "w:val") {
        props.scale = scale.parse().ok();
    }
    // Parse position offset
    if let Some(pos) = extract_first_attr(xml, "w:position", "w:val") {
        props.position = pos.parse().ok();
    }
    props.raw_xml = Some(xml.to_string());
    props
}
/// Parses a table element.
fn parse_table(xml: &str) -> FTable {
    let mut table = FTable::default();
    // Parse table properties
    if let Some(tblpr) = extract_element_inner(xml, "w:tblPr") {
        table.properties = parse_table_properties(&tblpr);
    }
    // Parse table grid (column widths)
    if let Some(tblgrid) = extract_element_inner(xml, "w:tblGrid") {
        for grid_col in collect_elements(&tblgrid, "w:gridCol") {
            if let Some(w) = extract_first_attr(&grid_col, "w:gridCol", "w:w") {
                if let Ok(width) = w.parse::<u32>() {
                    table.grid_columns.push(width);
                }
            }
        }
    }
    // Parse rows
    for row_xml in collect_elements(xml, "w:tr") {
        let mut row = FTableRow::default();
        // Parse row properties
        if let Some(trpr) = extract_element_inner(&row_xml, "w:trPr") {
            row.properties = parse_table_row_properties(&trpr);
        }
        // Parse cells
        for cell_xml in collect_elements(&row_xml, "w:tc") {
            let mut cell = FTableCell::default();
            // Parse cell properties
            if let Some(tcpr) = extract_element_inner(&cell_xml, "w:tcPr") {
                cell.properties = parse_table_cell_properties(&tcpr);
            }
            cell.grid_span = extract_first_attr(&cell_xml, "w:gridSpan", "w:val")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            cell.vertical_merge = extract_first_attr(&cell_xml, "w:vMerge", "w:val")
                .map(|v| {
                    if v == "restart" {
                        FVerticalMerge::Restart
                    } else {
                        FVerticalMerge::Continue
                    }
                })
                .or_else(|| {
                    if cell_xml.contains("<w:vMerge") {
                        Some(FVerticalMerge::Continue)
                    } else {
                        None
                    }
                });
            // Parse paragraphs in cell
            for p_xml in collect_elements(&cell_xml, "w:p") {
                cell.paragraphs.push(parse_paragraph(&p_xml));
            }
            // Parse nested tables
            for tbl_xml in collect_elements(&cell_xml, "w:tbl") {
                cell.tables.push(parse_table(&tbl_xml));
            }
            row.cells.push(cell);
        }
        row.raw_xml = Some(row_xml.clone());
        table.rows.push(row);
    }
    table.raw_xml = Some(xml.to_string());
    table
}
/// Parses table properties.
fn parse_table_properties(xml: &str) -> FTableProperties {
    let mut props = FTableProperties::default();
    if let Some(style) = extract_first_attr(xml, "w:tblStyle", "w:val") {
        props.style_id = Some(style);
    }
    if let Some(width) = extract_first_attr(xml, "w:tblW", "w:w") {
        props.width = width.parse().ok();
    }
    if let Some(width_type) = extract_first_attr(xml, "w:tblW", "w:type") {
        props.width_type = Some(width_type);
    }
    if let Some(align) = extract_first_attr(xml, "w:jc", "w:val") {
        props.alignment = Some(align);
    }
    if let Some(indent) = extract_first_attr(xml, "w:tblInd", "w:w") {
        props.indent = indent.parse().ok();
    }
    if let Some(layout) = extract_first_attr(xml, "w:tblLayout", "w:type") {
        props.layout = Some(layout);
    }
    props.raw_xml = Some(xml.to_string());
    props
}
/// Parses table row properties.
fn parse_table_row_properties(xml: &str) -> FTableRowProperties {
    let mut props = FTableRowProperties::default();
    if let Some(height) = extract_first_attr(xml, "w:trHeight", "w:val") {
        props.height = height.parse().ok();
    }
    if let Some(rule) = extract_first_attr(xml, "w:trHeight", "w:hRule") {
        props.height_rule = Some(rule);
    }
    if let Some(header) = flag_present(xml, "w:tblHeader") {
        props.is_header = Some(header);
    }
    if let Some(cant_split) = flag_present(xml, "w:cantSplit") {
        props.cant_split = Some(cant_split);
    }
    props.raw_xml = Some(xml.to_string());
    props
}
/// Parses table cell properties.
fn parse_table_cell_properties(xml: &str) -> FTableCellProperties {
    let mut props = FTableCellProperties::default();
    if let Some(width) = extract_first_attr(xml, "w:tcW", "w:w") {
        props.width = width.parse().ok();
    }
    // Parse cell borders
    if let Some(borders_xml) = extract_element_inner(xml, "w:tcBorders") {
        let mut borders = FCellBorders::default();
        for (tag, border) in [
            ("w:top", &mut borders.top),
            ("w:left", &mut borders.left),
            ("w:bottom", &mut borders.bottom),
            ("w:right", &mut borders.right),
        ] {
            if let Some(border_xml) = extract_element_inner(&borders_xml, tag) {
                let mut b = FBorder::default();
                if let Some(style) = extract_first_attr(&border_xml, tag, "w:val") {
                    b.style = Some(style);
                }
                if let Some(size) = extract_first_attr(&border_xml, tag, "w:sz") {
                    b.size = size.parse().ok();
                }
                if let Some(color) = extract_first_attr(&border_xml, tag, "w:color") {
                    b.color = Some(color);
                }
                *border = Some(b);
            }
        }
        props.borders = Some(borders);
    }
    // Parse cell shading
    if let Some(shd) = extract_first_attr(xml, "w:shd", "w:fill") {
        props.shading = Some(shd);
    }
    // Parse vertical alignment
    if let Some(valign) = extract_first_attr(xml, "w:vAlign", "w:val") {
        props.vertical_alignment = match valign.as_str() {
            "center" => Some(FCellVerticalAlignment::Center),
            "bottom" => Some(FCellVerticalAlignment::Bottom),
            _ => Some(FCellVerticalAlignment::Top),
        };
    }
    // Parse no wrap
    if let Some(no_wrap) = flag_present(xml, "w:noWrap") {
        props.no_wrap = Some(no_wrap);
    }
    props.raw_xml = Some(xml.to_string());
    props
}
/// Parses styles from XML bytes.
fn parse_styles(xml: &[u8]) -> crate::FWordResult<FStyles> {
    let text = std::str::from_utf8(xml)
        .map_err(|e| FWordError::Xml(format!("styles.xml is not valid utf-8: {}", e)))?;
    let mut styles = FStyles::default();
    // Parse document defaults
    if let Some(doc_defaults) = extract_element_inner(text, "w:docDefaults") {
        if let Some(rpr_default) = extract_element_inner(&doc_defaults, "w:rPrDefault") {
            if let Some(rpr) = extract_element_inner(&rpr_default, "w:rPr") {
                styles.default_run_properties = Some(parse_run_properties(&rpr));
            }
        }
        if let Some(ppr_default) = extract_element_inner(&doc_defaults, "w:pPrDefault") {
            if let Some(ppr) = extract_element_inner(&ppr_default, "w:pPr") {
                styles.default_paragraph_properties = Some(parse_paragraph_properties(&ppr));
            }
        }
    }
    // Parse style definitions
    for style_xml in collect_elements(text, "w:style") {
        let style_id = extract_first_attr(&style_xml, "w:style", "w:styleId").unwrap_or_default();
        let mut style = FStyle {
            style_id: style_id.clone(),
            ..Default::default()
        };
        // Parse style type
        if let Some(style_type) = extract_first_attr(&style_xml, "w:style", "w:type") {
            style.style_type = match style_type.as_str() {
                "character" => FStyleType::Character,
                "table" => FStyleType::Table,
                "numbering" => FStyleType::Numbering,
                _ => FStyleType::Paragraph,
            };
        }
        // Parse default flag
        if let Some(default) = extract_first_attr(&style_xml, "w:style", "w:default") {
            style.is_default = default == "1" || default == "true";
        }
        if let Some(name) = extract_first_attr(&style_xml, "w:name", "w:val") {
            style.name = Some(name);
        }
        if let Some(based) = extract_first_attr(&style_xml, "w:basedOn", "w:val") {
            style.based_on = Some(based);
        }
        if let Some(next) = extract_first_attr(&style_xml, "w:next", "w:val") {
            style.next = Some(next);
        }
        if let Some(link) = extract_first_attr(&style_xml, "w:link", "w:val") {
            style.link = Some(link);
        }
        if let Some(ui_priority) = extract_first_attr(&style_xml, "w:uiPriority", "w:val") {
            style.ui_priority = ui_priority.parse().ok();
        }
        if let Some(q_format) = flag_present(&style_xml, "w:qFormat") {
            style.quick_format = Some(q_format);
        }
        if let Some(ppr) = extract_element_inner(&style_xml, "w:pPr") {
            style.paragraph_properties = Some(parse_paragraph_properties(&ppr));
        }
        if let Some(rpr) = extract_element_inner(&style_xml, "w:rPr") {
            style.run_properties = Some(parse_run_properties(&rpr));
        }
        style.raw_xml = Some(style_xml.clone());
        styles.styles.insert(style_id, style);
    }
    styles.raw_xml = Some(text.to_string());
    Ok(styles)
}
/// Parses numbering from XML bytes.
fn parse_numbering(xml: &[u8]) -> crate::FWordResult<FNumbering> {
    let text = std::str::from_utf8(xml)
        .map_err(|e| FWordError::Xml(format!("numbering.xml is not valid utf-8: {}", e)))?;
    let mut numbering = FNumbering::default();
    // Parse abstract numbering definitions
    for abs_xml in collect_elements(text, "w:abstractNum") {
        let id: u32 = extract_first_attr(&abs_xml, "w:abstractNum", "w:abstractNumId")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let mut abs = FAbstractNum {
            abstract_num_id: id,
            ..Default::default()
        };
        if let Some(name) = extract_first_attr(&abs_xml, "w:name", "w:val") {
            abs.name = Some(name);
        }
        if let Some(mlt) = extract_first_attr(&abs_xml, "w:multiLevelType", "w:val") {
            abs.multi_level_type = Some(mlt);
        }
        if let Some(style_link) = extract_first_attr(&abs_xml, "w:styleLink", "w:val") {
            abs.style_link = Some(style_link);
        }
        // Parse levels
        for lvl_xml in collect_elements(&abs_xml, "w:lvl") {
            let level: u32 = extract_first_attr(&lvl_xml, "w:lvl", "w:ilvl")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            let mut lvl = FNumberingLevel {
                level,
                ..Default::default()
            };
            if let Some(fmt) = extract_first_attr(&lvl_xml, "w:numFmt", "w:val") {
                lvl.num_format = Some(fmt);
            }
            if let Some(txt) = extract_first_attr(&lvl_xml, "w:lvlText", "w:val") {
                lvl.level_text = Some(txt);
            }
            if let Some(start) = extract_first_attr(&lvl_xml, "w:start", "w:val") {
                lvl.start = start.parse().ok();
            }
            if let Some(jc) = extract_first_attr(&lvl_xml, "w:lvlJc", "w:val") {
                lvl.level_justification = Some(jc);
            }
            if let Some(suffix) = extract_first_attr(&lvl_xml, "w:suff", "w:val") {
                lvl.suffix = Some(suffix);
            }
            // Parse paragraph properties for this level
            if let Some(ppr) = extract_element_inner(&lvl_xml, "w:pPr") {
                lvl.paragraph_properties = Some(parse_paragraph_properties(&ppr));
            }
            // Parse run properties for this level
            if let Some(rpr) = extract_element_inner(&lvl_xml, "w:rPr") {
                lvl.run_properties = Some(parse_run_properties(&rpr));
            }
            abs.levels.push(lvl);
        }
        numbering.abstract_nums.insert(id, abs);
    }
    // Parse numbering instances
    for num_xml in collect_elements(text, "w:num") {
        let num_id: u32 = extract_first_attr(&num_xml, "w:num", "w:numId")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let abstract_id: u32 = extract_first_attr(&num_xml, "w:abstractNumId", "w:val")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        numbering.nums.insert(
            num_id,
            FNum {
                num_id,
                abstract_num_id: abstract_id,
            },
        );
    }
    Ok(numbering)
}
/// Parses settings from XML bytes.
fn parse_settings(xml: &[u8]) -> crate::FWordResult<FSettings> {
    let text = std::str::from_utf8(xml)
        .map_err(|e| FWordError::Xml(format!("settings.xml is not valid utf-8: {}", e)))?;
    let mut settings = FSettings::default();
    if let Some(tab) = extract_first_attr(text, "w:defaultTabStop", "w:val") {
        settings.default_tab_stop = tab.parse().ok();
    }
    if let Some(even_odd) = flag_present(text, "w:evenAndOddHeaders") {
        settings.even_and_odd_headers = Some(even_odd);
    }
    if let Some(mirror) = flag_present(text, "w:mirrorMargins") {
        settings.mirror_margins = Some(mirror);
    }
    if let Some(track) = flag_present(text, "w:trackChanges") {
        settings.track_changes = Some(track);
    }
    // Parse document protection
    if let Some(protect_xml) = extract_element_inner(text, "w:documentProtection") {
        let mut protection = FDocumentProtection::default();
        if let Some(prot_type) = extract_first_attr(&protect_xml, "w:documentProtection", "w:edit")
        {
            protection.protection_type = Some(prot_type);
        }
        if let Some(enforcement) =
            extract_first_attr(&protect_xml, "w:documentProtection", "w:enforcement")
        {
            protection.enforcement = Some(enforcement == "1" || enforcement == "true");
        }
        if let Some(alg) =
            extract_first_attr(&protect_xml, "w:documentProtection", "w:cryptAlgorithmSid")
        {
            protection.algorithm_name = Some(alg);
        }
        if let Some(hash) = extract_first_attr(&protect_xml, "w:documentProtection", "w:hash") {
            protection.hash_value = Some(hash);
        }
        if let Some(salt) = extract_first_attr(&protect_xml, "w:documentProtection", "w:salt") {
            protection.salt_value = Some(salt);
        }
        if let Some(spin) = extract_first_attr(&protect_xml, "w:documentProtection", "w:spinCount")
        {
            protection.spin_count = spin.parse().ok();
        }
        settings.document_protection = Some(protection);
    }
    // Parse compatibility mode
    if let Some(compat) = extract_element_inner(text, "w:compat") {
        if let Some(mode) = extract_first_attr(&compat, "w:compatSetting", "w:name") {
            settings.compat_mode = Some(mode);
        }
    }
    if let Some(lang) = extract_first_attr(text, "w:themeFontLang", "w:val") {
        settings.theme_font_language = Some(lang);
    }
    if let Some(decimal) = extract_first_attr(text, "w:decimalSymbol", "w:val") {
        settings.decimal_symbol = Some(decimal);
    }
    if let Some(separator) = extract_first_attr(text, "w:listSeparator", "w:val") {
        settings.list_separator = Some(separator);
    }
    settings.raw_xml = Some(text.to_string());
    Ok(settings)
}
