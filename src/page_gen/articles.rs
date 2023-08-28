use super::tags::ArticleLink;
use crate::{markdown::article::FrontMatter, TemplateRenderer};
use anyhow::Context;

pub struct Articles<'a> {
    articles: Vec<ArticleLink<'a>>,
}

impl Articles<'_> {
    pub fn new<'a>(articles: &'a Vec<(FrontMatter, String)>) -> Articles<'a> {
        let mut qual_article_links = Vec::new();
        for (frontmatter, link) in articles {
            qual_article_links.push(ArticleLink { frontmatter, link });
        }

        Articles {
            articles: qual_article_links,
        }
    }
}

impl TemplateRenderer for Articles<'_> {
    fn render_template(&self, tera: &tera::Tera) -> anyhow::Result<String> {
        let mut tera_context = tera::Context::new();
        tera_context.insert("articles", &self.articles);
        tera.render("articles.html", &tera_context)
            .context(format!("Failed to render primary articles page!"))
    }
}
