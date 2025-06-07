use crate::Node;
use std::collections::HashMap;
use crate::codegen::special_functions::SpecialFunctionRegistry;

#[derive(Debug, Clone)]
pub struct ComponentDefinition {
    pub params: Vec<String>,
    pub body: Vec<Node>,
}

pub fn generate_html(
    node: &Node,
    indent: usize,
    is_root: bool,
    components: &mut HashMap<String, ComponentDefinition>,
) -> String {
    generate_html_with_registry(node, indent, is_root, components, &mut SpecialFunctionRegistry::new())
}

pub fn generate_html_with_registry(
    node: &Node,
    indent: usize,
    is_root: bool,
    components: &mut HashMap<String, ComponentDefinition>,
    registry: &mut SpecialFunctionRegistry,
) -> String {
    let void_tags = vec![
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
        "track", "wbr",
    ];

    let mut html = String::new();
    let indentation = if is_root {
        "".to_string()
    } else {
        "    ".repeat(indent)
    };

    if node.name == "text" {
        if let Some(attr) = node.atributes.iter().find(|a| a.name == "value") {
            html.push_str(&indentation);
            html.push_str(&attr.value);
            html.push('\n');
        }
        return html;
    }


    if node.is_special_function {
        match registry.execute(node, indent, is_root, components) {
            Ok(result) => return result,
            Err(error) => {
                eprintln!("{}", error);
                return String::new();
            }
        }
    }

    if node.name.starts_with('$') {

        let component_def = ComponentDefinition {
            params: node.arguments.clone(),
            body: node.children.clone(),
        };
        components.insert(node.name.clone(), component_def);
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


        if let Some(component_def) = components.get(&child.name).cloned() {
            let substituted_body =
                substitute_arguments(&component_def.body, &component_def.params, &child.arguments);
            for component_child in substituted_body {
                html.push_str(&generate_html_with_registry(
                    &component_child,
                    child_indent,
                    false,
                    components,
                    registry,
                ));
            }
        } else {
            html.push_str(&generate_html_with_registry(child, child_indent, false, components, registry));
        }
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

fn substitute_arguments(nodes: &[Node], params: &[String], args: &[String]) -> Vec<Node> {
    let mut substituted = Vec::new();

    for node in nodes {
        let mut new_node = node.clone();


        for attr in &mut new_node.atributes {
            attr.value = substitute_in_string(&attr.value, params, args);
        }


        for arg in &mut new_node.arguments {
            *arg = substitute_in_string(arg, params, args);
        }


        new_node.children = substitute_arguments(&node.children, params, args);

        substituted.push(new_node);
    }

    substituted
}



fn substitute_in_string(text: &str, params: &[String], args: &[String]) -> String {
    let mut result = text.to_string();

    for (i, param) in params.iter().enumerate() {
        if let Some(arg) = args.get(i) {
            // Usuń cudzysłowy z argumentu jeśli są
            let clean_arg = arg.trim_matches('"');
            // Zastąp tylko placeholder z klamrami
            result = result.replace(&format!("{{{}}}", param), clean_arg);
        }
    }

    result
}
