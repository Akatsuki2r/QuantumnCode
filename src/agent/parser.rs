//! Parse tool calls from AI response

use super::tools::ToolCall;

/// Parse tool calls from AI response text
pub fn parse_tool_calls(text: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("<tool>") {
        let after_start = &remaining[start + 6..];

        if let Some(end) = after_start.find("</tool>") {
            let tool_xml = &after_start[..end];

            let name = extract_tag(tool_xml, "name");
            let arg = extract_tag(tool_xml, "arg");
            let content = extract_tag_optional(tool_xml, "content");

            if let Some(name) = name {
                calls.push(ToolCall {
                    name,
                    arg: arg.unwrap_or_default(),
                    content,
                });
            }

            remaining = &after_start[end + 7..];
        } else {
            break;
        }
    }

    calls
}

/// Extract required tag value
fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);

    if let Some(start) = xml.find(&open_tag) {
        let after_open = &xml[start + open_tag.len()..];
        if let Some(end) = after_open.find(&close_tag) {
            return Some(after_open[..end].trim().to_string());
        }
    }
    None
}

/// Extract optional tag value
fn extract_tag_optional(xml: &str, tag: &str) -> Option<String> {
    extract_tag(xml, tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_tool() {
        let text = r#"Hello world
<tool>
<name>Read</name>
<arg>src/main.rs</arg>
</tool>
Goodbye"#;

        let calls = parse_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "Read");
        assert_eq!(calls[0].arg, "src/main.rs");
    }

    #[test]
    fn test_parse_multiple_tools() {
        let text = r#"<tool>
<name>Read</name>
<arg>file1.txt</arg>
</tool>
<tool>
<name>Write</name>
<arg>file2.txt</arg>
<content>hello</content>
</tool>"#;

        let calls = parse_tool_calls(text);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "Read");
        assert_eq!(calls[1].name, "Write");
        assert_eq!(calls[1].content, Some("hello".to_string()));
    }

    #[test]
    fn test_parse_no_tools() {
        let text = "Just a normal response without any tools.";
        let calls = parse_tool_calls(text);
        assert!(calls.is_empty());
    }
}
