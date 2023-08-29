use crate::markdown::article::FrontMatter;
use serde::Serialize;

pub mod articles;
pub mod tags;

#[derive(Serialize)]
pub struct ArticleLink<'a> {
    frontmatter: &'a FrontMatter,
    link: &'a String,
}