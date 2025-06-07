
use crate::Node;
use crate::codegen::codegen::{generate_html, ComponentDefinition};
use std::collections::HashMap;

pub type SpecialFunctionHandler = fn(&Node, usize, bool, &mut HashMap<String, ComponentDefinition>) -> String;



pub fn handle_loop(
    node: &Node,
    indent: usize,
    is_root: bool,
    components: &mut HashMap<String, ComponentDefinition>,
) -> String {
    let mut html = String::new();
    
    if let Some(count_str) = node.arguments.get(0) {
        if let Ok(count) = count_str.parse::<usize>() {
            for _ in 0..count {
                for child in &node.children {
                    html.push_str(&generate_html(child, indent, is_root, components));
                }
            }
        } else {
            eprintln!("Invalid loop count: {}", count_str);
        }
    } else {
        eprintln!("Loop function requires a count argument");
    }
    
    html
}

pub fn handle_if(
    node: &Node,
    indent: usize,
    is_root: bool,
    components: &mut HashMap<String, ComponentDefinition>,
) -> String {
    let mut html = String::new();
    
    if let Some(condition) = node.arguments.get(0) {
     
        let should_render = !matches!(condition.as_str(), "false" | "0" | "");
        
        if should_render {
            for child in &node.children {
                html.push_str(&generate_html(child, indent, is_root, components));
            }
        }
    } else {
        eprintln!("If function requires a condition argument");
    }
    
    html
}

pub fn handle_repeat(
    node: &Node,
    indent: usize,
    is_root: bool,
    components: &mut HashMap<String, ComponentDefinition>,
) -> String {
    let mut html = String::new();
    
    if let Some(text) = node.arguments.get(0) {
        if let Some(count_str) = node.arguments.get(1) {
            if let Ok(count) = count_str.parse::<usize>() {
                let indentation = if is_root { "".to_string() } else { "    ".repeat(indent) };
                for _ in 0..count {
                    html.push_str(&indentation);
                    html.push_str(&text.trim_matches('"'));
                    html.push('\n');
                }
            } else {
                eprintln!("Invalid repeat count: {}", count_str);
            }
        } else {
            eprintln!("Repeat function requires text and count arguments");
        }
    } else {
        eprintln!("Repeat function requires text and count arguments");
    }
    
    html
}

pub fn handle_each(
    node: &Node,
    indent: usize,
    is_root: bool,
    components: &mut HashMap<String, ComponentDefinition>,
) -> String {
    let mut html = String::new();
    
    if let Some(items_str) = node.arguments.get(0) {

        let items: Vec<&str> = items_str.trim_matches('"').split(',').map(|s| s.trim()).collect();
        
        for item in items {
          
            for child in &node.children {
                let mut child_copy = child.clone();
                substitute_item_in_node(&mut child_copy, item);
                html.push_str(&generate_html(&child_copy, indent, is_root, components));
            }
        }
    } else {
        eprintln!("Each function requires an items argument");
    }
    
    html
}


fn substitute_item_in_node(node: &mut Node, item: &str) {

    for attr in &mut node.atributes {
        attr.value = attr.value.replace("{item}", item);
    }
    

    for arg in &mut node.arguments {
        *arg = arg.replace("{item}", item);
    }
    

    for child in &mut node.children {
        substitute_item_in_node(child, item);
    }
}
