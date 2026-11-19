#[cfg(test)]
mod tests {
    use f_word::{
        FBlock, FBorder, FCellBorders, FCellVerticalAlignment, FHyperlink, FNumberingProperties,
        FParagraph, FParagraphAlignment, FParagraphProperties, FRun, FRunProperties, FTable,
        FTableCell, FTableCellProperties, FTableProperties, FTableRow, FTableRowProperties,
        FVerticalAlign, FVerticalMerge, FWord,
    };
    /// Returns the output directory (Downloads on Windows, temp dir elsewhere).
    fn output_dir() -> std::path::PathBuf {
        #[cfg(target_os = "windows")]
        {
            std::path::PathBuf::from(r"C:\Users\admin\Downloads")
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::env::temp_dir()
        }
    }
    /// Serializes the document, writes it to `file_name`, and asserts it is non-empty.
    fn save(doc: &FWord, file_name: &str) -> std::path::PathBuf {
        let path = output_dir().join(file_name);
        let bytes = doc.to_bytes().expect("failed to serialize document");
        std::fs::write(&path, &bytes).expect("failed to write document");
        let meta = std::fs::metadata(&path).expect("output file missing");
        assert!(meta.len() > 0, "output file is empty: {file_name}");
        path
    }
    /// Builds a bold run.
    fn bold_run(text: &str) -> FRun {
        FRun {
            text: text.to_string(),
            properties: FRunProperties {
                bold: Some(true),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    /// Builds a run with the given properties.
    fn styled_run(text: &str, props: FRunProperties) -> FRun {
        FRun {
            text: text.to_string(),
            properties: props,
            ..Default::default()
        }
    }
    /// Builds a cell containing a single paragraph with the given text.
    fn make_cell(text: &str) -> FTableCell {
        let mut cell = FTableCell::default();
        let mut p = FParagraph::default();
        p.runs.push(FRun {
            text: text.to_string(),
            ..Default::default()
        });
        cell.paragraphs.push(p);
        cell
    }
    #[test]
    fn create_empty_docs() {
        let doc = FWord::new_empty();
        let path = save(&doc, "empty.docx");
        println!("empty document written to {}", path.display());
    }
    /// Various run styles: bold, italic, underline, strike, size, font,
    /// color, highlight, superscript/subscript, caps, small caps.
    #[test]
    fn create_styled_runs() {
        let mut doc = FWord::new_empty();
        let mut p = FParagraph::default();
        p.runs.push(bold_run("Bold text "));
        p.runs.push(styled_run(
            "Italic text ",
            FRunProperties {
                italic: Some(true),
                ..Default::default()
            },
        ));
        p.runs.push(styled_run(
            "Underlined text ",
            FRunProperties {
                underline: Some(true),
                ..Default::default()
            },
        ));
        p.runs.push(styled_run(
            "Strikethrough text ",
            FRunProperties {
                strike: Some(true),
                ..Default::default()
            },
        ));
        p.runs.push(styled_run(
            "Large text ",
            FRunProperties {
                size: Some(48), // 24pt
                ..Default::default()
            },
        ));
        p.runs.push(styled_run(
            "Monospace font ",
            FRunProperties {
                font_family: Some("Consolas".into()),
                ..Default::default()
            },
        ));
        p.runs.push(styled_run(
            "Red text ",
            FRunProperties {
                color: Some("FF0000".into()),
                ..Default::default()
            },
        ));
        p.runs.push(styled_run(
            "Highlighted text ",
            FRunProperties {
                highlight: Some("yellow".into()),
                ..Default::default()
            },
        ));
        p.runs.push(styled_run(
            "Superscript",
            FRunProperties {
                vert_align: Some(FVerticalAlign::Superscript),
                ..Default::default()
            },
        ));
        p.runs.push(styled_run(
            " subscript ",
            FRunProperties {
                vert_align: Some(FVerticalAlign::Subscript),
                ..Default::default()
            },
        ));
        p.runs.push(styled_run(
            "ALL CAPS ",
            FRunProperties {
                caps: Some(true),
                ..Default::default()
            },
        ));
        p.runs.push(styled_run(
            "Small Caps",
            FRunProperties {
                small_caps: Some(true),
                ..Default::default()
            },
        ));
        doc.body.blocks.push(FBlock::Paragraph(p));
        let path = save(&doc, "styled_runs.docx");
        println!("styled runs written to {}", path.display());
    }
    /// Paragraph properties: alignment, indentation, spacing, line spacing, numbering.
    #[test]
    fn create_paragraph_formatting() {
        let mut doc = FWord::new_empty();
        let mut centered = FParagraph::default();
        centered.runs.push(FRun {
            text: "Centered paragraph".into(),
            ..Default::default()
        });
        centered.properties.alignment = FParagraphAlignment::Center;
        doc.body.blocks.push(FBlock::Paragraph(centered));
        let mut justified = FParagraph::default();
        justified.runs.push(FRun {
            text: "Justified paragraph with indentation and spacing.".into(),
            ..Default::default()
        });
        justified.properties = FParagraphProperties {
            alignment: FParagraphAlignment::Justify,
            indent_left: Some(720),       // 0.5 inch
            indent_first_line: Some(360), // 0.25 inch
            spacing_before: Some(120),
            spacing_after: Some(120),
            line_spacing: Some(360),
            ..Default::default()
        };
        doc.body.blocks.push(FBlock::Paragraph(justified));
        let mut numbered = FParagraph::default();
        numbered.runs.push(FRun {
            text: "Numbered paragraph".into(),
            ..Default::default()
        });
        numbered.properties.numbering = Some(FNumberingProperties {
            level: Some(0),
            num_id: Some(1),
        });
        doc.body.blocks.push(FBlock::Paragraph(numbered));
        let path = save(&doc, "paragraph_formatting.docx");
        println!("paragraph formatting written to {}", path.display());
    }
    /// Hyperlinks: external link and internal anchor.
    #[test]
    fn create_hyperlinks() {
        let mut doc = FWord::new_empty();
        let mut p = FParagraph::default();
        p.runs.push(FRun {
            text: "Visit ".into(),
            ..Default::default()
        });
        p.hyperlinks.push(FHyperlink {
            relationship_id: Some("rId100".into()),
            anchor: None,
            target: Some("https://www.rust-lang.org".into()),
            text: "the Rust website".into(),
            is_external: true,
        });
        p.hyperlinks.push(FHyperlink {
            relationship_id: None,
            anchor: Some("bookmark1".into()),
            target: None,
            text: "internal bookmark".into(),
            is_external: false,
        });
        doc.body.blocks.push(FBlock::Paragraph(p));
        let path = save(&doc, "hyperlinks.docx");
        println!("hyperlinks written to {}", path.display());
    }
    /// Table: header row, row height, merged cells, borders, shading,
    /// vertical alignment, and a nested table.
    #[test]
    fn create_tables() {
        let mut doc = FWord::new_empty();
        // Header row
        let mut header = FTableRow::default();
        header.properties = FTableRowProperties {
            is_header: Some(true),
            height: Some(400),
            height_rule: Some("atLeast".into()),
            ..Default::default()
        };
        header.cells.push(make_cell("Header A"));
        header.cells.push(make_cell("Header B"));
        header.cells.push(make_cell("Header C"));
        // Data row 1: vertical merge start on first cell
        let mut row1 = FTableRow::default();
        let mut merged = make_cell("Merged vertically");
        merged.vertical_merge = Some(FVerticalMerge::Restart);
        row1.cells.push(merged);
        row1.cells.push(make_cell("B1"));
        row1.cells.push(make_cell("C1"));
        // Data row 2: continue vertical merge, horizontal span on second cell
        let mut row2 = FTableRow::default();
        let mut cont = make_cell("");
        cont.vertical_merge = Some(FVerticalMerge::Continue);
        row2.cells.push(cont);
        let mut spanned = make_cell("Spans two columns");
        spanned.grid_span = 2;
        row2.cells.push(spanned);
        // Data row 3: cell with borders, shading, vertical alignment
        let mut row3 = FTableRow::default();
        let border = Some(FBorder {
            style: Some("single".into()),
            size: Some(8),
            color: Some("0000FF".into()),
        });
        let mut styled = make_cell("Styled cell");
        styled.properties = FTableCellProperties {
            borders: Some(FCellBorders {
                top: border.clone(),
                left: border.clone(),
                bottom: border.clone(),
                right: border,
            }),
            shading: Some("D9E2F3".into()),
            vertical_alignment: Some(FCellVerticalAlignment::Center),
            ..Default::default()
        };
        row3.cells.push(styled);
        row3.cells.push(make_cell("B3"));
        row3.cells.push(make_cell("C3"));
        // Data row 4: nested table inside a cell
        let mut row4 = FTableRow::default();
        let mut nested_host = make_cell("Host cell");
        let mut nested = FTable::default();
        let mut nested_row = FTableRow::default();
        nested_row.cells.push(make_cell("Nested 1"));
        nested_row.cells.push(make_cell("Nested 2"));
        nested.rows.push(nested_row);
        nested_host.tables.push(nested);
        row4.cells.push(nested_host);
        row4.cells.push(make_cell("B4"));
        row4.cells.push(make_cell("C4"));
        let mut table = FTable::default();
        table.properties = FTableProperties {
            width: Some(9000),
            width_type: Some("dxa".into()),
            alignment: Some("center".into()),
            ..Default::default()
        };
        table.grid_columns = vec![3000, 3000, 3000];
        table.rows = vec![header, row1, row2, row3, row4];
        doc.body.blocks.push(FBlock::Table(table));
        let path = save(&doc, "tables.docx");
        println!("tables written to {}", path.display());
    }
    /// Round-trip: serialize, re-parse, and verify the content survived.
    #[test]
    fn round_trip_document() {
        let mut doc = FWord::new_empty();
        let mut p = FParagraph::default();
        p.runs.push(bold_run("Round trip bold"));
        p.runs.push(styled_run(
            " and italic",
            FRunProperties {
                italic: Some(true),
                ..Default::default()
            },
        ));
        doc.body.blocks.push(FBlock::Paragraph(p));
        let bytes = doc.to_bytes().expect("serialize failed");
        let parsed = FWord::with_bytes(&bytes).expect("re-parse failed");
        let text = parsed.plain_text();
        assert!(
            text.contains("Round trip bold"),
            "bold text missing: {text}"
        );
        assert!(text.contains("and italic"), "italic text missing: {text}");
        match &parsed.body.blocks[0] {
            FBlock::Paragraph(p) => {
                assert_eq!(p.runs[0].properties.bold, Some(true));
                assert_eq!(p.runs[1].properties.italic, Some(true));
            }
            other => panic!("expected paragraph, got {other:?}"),
        }
        let path = save(&parsed, "round_trip.docx");
        println!("round-trip document written to {}", path.display());
    }
}
