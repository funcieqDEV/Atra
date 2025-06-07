#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub atributes: Vec<Attribute>,
    pub name: String,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}
