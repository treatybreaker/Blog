use tera::Tera;

pub mod markdown;
pub mod page_gen;

pub trait TemplateRenderer {
    fn render_template(&self, tera: &Tera) -> anyhow::Result<String>;
}
