
use crate::Node;
use crate::codegen::codegen::ComponentDefinition;
use crate::codegen::special_function_definitions::*;
use std::collections::HashMap;

pub struct SpecialFunctionRegistry {
    functions: HashMap<String, SpecialFunctionHandler>,
}

impl SpecialFunctionRegistry {
    pub fn new() -> Self {
        let mut registry = SpecialFunctionRegistry {
            functions: HashMap::new(),
        };
     
        registry.register("%loop", handle_loop);
        registry.register("%if", handle_if);
        registry.register("%repeat", handle_repeat);
        registry.register("%each", handle_each);
        registry.register("%readFile", handle_readfile);
        
        registry
    }
    
    pub fn register(&mut self, name: &str, handler: SpecialFunctionHandler) {
        self.functions.insert(name.to_string(), handler);
    }
    
    pub fn execute(&self, node: &Node, indent: usize, is_root: bool, components: &mut HashMap<String, ComponentDefinition>) -> Result<String, String> {
        if let Some(handler) = self.functions.get(&node.name) {
            Ok(handler(node, indent, is_root, components))
        } else {
            Err(format!("Unknown special function: {}", node.name))
        }
    }
    
    // pub fn is_registered(&self, name: &str) -> bool {
    //     self.functions.contains_key(name)
    // }
}


