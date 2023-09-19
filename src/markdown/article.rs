use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::NaiveDate;
use comrak::{nodes::NodeValue, parse_document, Arena};
use serde::{Deserialize, Serialize};

use crate::TemplateRenderer;

use super::{iter_nodes, MDComrakSettings};

#[derive(Debug, Serialize)]
pub struct Article {
    pub html_content: String,
    pub frontmatter: FrontMatter,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrontMatter {
    pub name: String,
    pub summary: String,
    pub published: NaiveDate,
    pub updated: NaiveDate,
    pub tags: Vec<String>,
}

impl Article {
    pub fn parse(
        article_md_path: &PathBuf,
        comrak_settings: &MDComrakSettings,
    ) -> anyhow::Result<Article> {
        let article_content = Self::load_doc(article_md_path).with_context(|| {
            format!("Could not read article markdown document at {article_md_path:?}")
        })?;
        let arena = Arena::new();
        let root = parse_document(&arena, &article_content, &comrak_settings.options);

        let mut front_matter_raw: String = String::from("");
        iter_nodes(root, &mut |node| match node.data.borrow().value {
            // NOTE: This is kinda hacky, we just assume the frontmatter delimiter is ALWAYS "---";
            // furthermore, we assume there is even a frontmatter input period and this will have a
            // nuclear incident if there doesn't happen to be a frontmatter delimiter. We should
            // better handle this, but as of the time of this writing I don't fucken care enough.
            // Will come back to later or this will be a temporary permanent fix.
            NodeValue::FrontMatter(ref front_data) => {
                front_matter_raw = front_data
                    .lines()
                    .filter(|line| match *line {
                        "---" => false,
                        _ => true,
                    })
                    .map(|s| s.to_string() + "\n")
                    .collect::<String>();
            }
            _ => (),
        });
        let frontmatter: FrontMatter =
            serde_yaml::from_str(&front_matter_raw).with_context(|| {
                format!("Failed to parse frontmatter for document: {article_md_path:?}")
            })?;

        let mut html_out = vec![];
        comrak::format_html_with_plugins(
            root,
            &comrak_settings.options,
            &mut html_out,
            &comrak_settings.plugins,
        )?;
        let html = String::from_utf8(html_out)?;

        Ok(Article {
            html_content: html,
            frontmatter,
        })
    }

    fn load_doc<P: AsRef<Path>>(file_path: P) -> anyhow::Result<String> {
        let mut f = File::options().read(true).write(false).open(file_path)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        Ok(s)
    }
}

impl TemplateRenderer for Article {
    fn render_template(&self, tera: &tera::Tera) -> anyhow::Result<String> {
        let mut tera_context = tera::Context::new();
        tera_context.insert("article_title", &self.frontmatter.name);
        tera_context.insert("article_summary", &self.frontmatter.summary);
        tera_context.insert("article_published", &self.frontmatter.published);
        tera_context.insert("article_last_updated", &self.frontmatter.updated);
        tera_context.insert("article_tags", &self.frontmatter.tags);
        tera_context.insert("article_content", &self.html_content);
        tera.render("article.html", &tera_context).context(format!(
            "Failed to render Article: '{}'",
            &self.frontmatter.name
        ))
    }
}
