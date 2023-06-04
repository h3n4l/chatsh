pub mod openai;

#[derive(Clone, Debug)]
pub struct Detail {
    pub description: String,
    pub command: String,
}

pub trait Converter {
    fn convert(&self, question: &str) -> anyhow::Result<Detail>;
}
