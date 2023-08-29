use crate::TemplateRenderer;

pub struct Home {}

impl TemplateRenderer for Home {
    fn render_template(&self, tera: &tera::Tera) -> anyhow::Result<String> {
        let empty_context = tera::Context::new();
        Ok(tera.render("home.html", &empty_context)?)
    }
}