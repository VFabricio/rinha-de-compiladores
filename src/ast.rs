use serde::Deserialize;

#[derive(Deserialize)]
pub struct File {
    pub name: String,
    pub expression: Term,
    pub location: Location,
}

#[derive(Deserialize)]
pub struct Location {
    pub start: u32,
    pub end: u32,
    pub filename: String,
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
pub enum Term {
    Int(i32),
}
