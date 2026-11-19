//! Minimal XML helpers.
pub(crate) fn extract_first_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)?;
    let after = xml[start..].find('>')? + start + 1;
    let end_rel = xml[after..].find(&close)?;
    Some(xml[after..after + end_rel].to_string())
}
/// Returns the value of attribute `attr` on the first element `tag`.
pub(crate) fn extract_first_attr(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let start = xml.find(&open)?;
    let tag_end = xml[start..].find('>')? + start;
    let tag_str = &xml[start..tag_end];
    let needle = format!("{}=\"", attr);
    let a = tag_str.find(&needle)? + needle.len();
    let b = tag_str[a..].find('"')? + a;
    Some(decode_xml_text(&tag_str[a..b]))
}
/// Collects the inner XML of every element with the given `tag`.
pub(crate) fn collect_elements(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let mut result = Vec::new();
    let mut cursor = 0;
    while let Some(rel_start) = xml[cursor..].find(&open) {
        let start = cursor + rel_start;
        let after = match xml[start..].find('>') {
            Some(v) => start + v + 1,
            None => break,
        };
        let end_rel = match xml[after..].find(&close) {
            Some(v) => v,
            None => break,
        };
        let end = after + end_rel;
        result.push(xml[after..end].to_string());
        cursor = end + close.len();
    }
    result
}
/// Decodes the five predefined XML entities.
pub(crate) fn decode_xml_text(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
/// Encodes text for inclusion in XML content.
pub(crate) fn encode_xml_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
/// Returns `true` when a boolean OOXML element is present and not disabled.
pub(crate) fn flag_present(xml: &str, tag: &str) -> Option<bool> {
    let open = format!("<{}", tag);
    let start = xml.find(&open)?;
    let tag_end = xml[start..].find('>')? + start;
    let tag_str = &xml[start..tag_end];
    if tag_str.contains("w:val=\"0\"")
        || tag_str.contains("w:val=\"false\"")
        || tag_str.contains("w:val=\"off\"")
    {
        Some(false)
    } else {
        Some(true)
    }
}
/// Returns the inner XML of the first element `tag`, excluding the tag itself.
pub(crate) fn extract_element_inner(xml: &str, tag: &str) -> Option<String> {
    let (inner, _) = extract_element_with_span(xml, tag)?;
    Some(inner)
}
/// Returns the inner XML and the number of bytes consumed in `xml` for the first element `tag`.
pub(crate) fn extract_element_with_span(xml: &str, tag: &str) -> Option<(String, usize)> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)?;
    let after_open = xml[start..].find('>')? + start + 1;
    let end_rel = xml[after_open..].find(&close)?;
    let inner = xml[after_open..after_open + end_rel].to_string();
    let consumed = after_open + end_rel + close.len() - start;
    Some((inner, consumed))
}
/// Checks if an element exists (including self-closing form).
pub(crate) fn element_exists(xml: &str, tag: &str) -> bool {
    let open = format!("<{}", tag);
    let self_close = format!("{}/>", open);
    xml.contains(&open) || xml.contains(&self_close)
}
