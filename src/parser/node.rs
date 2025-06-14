#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub atributes: Vec<Attribute>,
    pub local_styles: Vec<Attribute>,
    pub name: String,
    pub children: Vec<Node>,
    pub arguments: Vec<String>,
    pub is_special_function: bool, 
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}
