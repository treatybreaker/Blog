use crate::{markdown::article::FrontMatter, TemplateRenderer};
use anyhow::Context;
use serde::Serialize;

pub struct Tags<'a> {
    tags: &'a Vec<&'a String>,
}

impl Tags<'_> {
    pub fn new<'a>(tags: &'a Vec<&'a String>) -> Tags<'a> {
        Tags { tags }
    }
}

impl TemplateRenderer for Tags<'_> {
    fn render_template(&self, tera: &tera::Tera) -> anyhow::Result<String> {
        let mut tera_context = tera::Context::new();
        tera_context.insert("tags", &self.tags);
        tera.render("tags.html", &tera_context).context(format!(
            "Failed to render tags page, had tags: {:#?}",
            &self.tags
        ))
    }
}

#[derive(Serialize)]
pub struct ArticleLink<'a> {
    pub frontmatter: &'a FrontMatter,
    pub link: &'a String,
}

pub struct TagArticles<'a> {
    tag: &'a String,
    article_links: Vec<ArticleLink<'a>>,
}

impl TagArticles<'_> {
    pub fn new<'a>(
        tag: &'a String,
        article_links: &'a Vec<(FrontMatter, String)>,
    ) -> TagArticles<'a> {
        let mut qual_article_links = Vec::new();
        for (frontmatter, link) in article_links {
            qual_article_links.push(ArticleLink { frontmatter, link });
        }
        TagArticles {
            tag,
            article_links: qual_article_links,
        }
    }
}

impl TemplateRenderer for TagArticles<'_> {
    fn render_template(&self, tera: &tera::Tera) -> anyhow::Result<String> {
        let mut terra_context = tera::Context::new();
        terra_context.insert("tag", &self.tag);
        terra_context.insert("article_links", &self.article_links);
        tera.render("tag-articles.html", &terra_context)
            .context(format!(
                "Failed to render tag articles page for tag: {}",
                &self.tag
            ))
    }
}
