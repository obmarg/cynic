#[derive(Debug)]
pub struct Schema {
    pub query: String,
}

#[derive(Debug)]
pub struct Object {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub ty: String,
}
