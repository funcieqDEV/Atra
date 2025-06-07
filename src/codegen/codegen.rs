use crate::parser::node::Node;

pub fn generate_html(node: &Node, indent: usize, is_root: bool) -> String {
    let void_tags = vec!["meta", "img", "br", "hr", "input", "link"];
    let mut html = String::new();
    let indentation = if is_root {
        "".to_string()
    } else {
        "    ".repeat(indent)
    };

    if node.name == "text" {
        if let Some(attr) = node.atributes.iter().find(|a| a.name == "value") {
            html.push_str(&indentation);
            html.push_str(&attr.value.trim_matches('"'));
            html.push('\n');
        }
        return html;
    }

    if !node.name.is_empty() {
        html.push_str(&indentation);
        html.push('<');
        html.push_str(&node.name);

        for attr in &node.atributes {
            html.push(' ');
            html.push_str(&attr.name);
            html.push('=');
            html.push('"');
            html.push_str(&attr.value.trim_matches('"'));
            html.push('"');
        }

        html.push('>');
        html.push('\n');
    }

    for child in &node.children {
        let child_indent = if is_root { indent } else { indent + 1 };
        html.push_str(&generate_html(child, child_indent, false));
    }

    if !node.name.is_empty() && !void_tags.contains(&node.name.as_str()) {
        html.push_str(&indentation);
        html.push_str("</");
        html.push_str(&node.name);
        html.push('>');
        html.push('\n');
    }

    html
}
