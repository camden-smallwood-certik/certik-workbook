
#[derive(Clone)]
pub enum DataKind {
    Int,
    Float,
    String
}

#[derive(Clone)]
pub struct Parameter {
    pub name: String,
    pub kind: DataKind
}

impl Parameter {
    pub fn new(name: &str, kind: DataKind) -> Self {
        Self {
            name: name.to_string(),
            kind
        }
    }
}

pub struct Command {
    pub name: String,
    pub parameters: Vec<Parameter>,
}

impl Command {
    pub fn new(name: &str, parameters: &[Parameter]) -> Self {
        Self {
            name: name.to_string(),
            parameters: parameters.to_vec()
        }
    }
}
