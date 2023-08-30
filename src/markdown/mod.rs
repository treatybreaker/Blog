use std::io::Cursor;

use comrak::{nodes::AstNode, ComrakOptions, ComrakPlugins, plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder}};
use lazy_static::lazy_static;
use syntect::highlighting::ThemeSet;

pub mod article;

lazy_static! {
    pub static ref SYNTECT_ADAPTER: SyntectAdapter =
        MDComrakSettings::load_theme("kanagawa", &mut Cursor::new(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/Kanagawa.tmTheme"
        ))))
        .expect("Unable to load custom syntax theme!");
}

pub fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &mut F)
where
    F: FnMut(&'a AstNode<'a>),
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

#[derive(Debug)]
pub struct MDComrakSettings<'a> {
    pub options: ComrakOptions,
    pub plugins: ComrakPlugins<'a>,
}

impl MDComrakSettings<'_> {
    pub fn default<'a>() -> anyhow::Result<MDComrakSettings<'a>> {
        let mut options = ComrakOptions::default();
        options.render.unsafe_ = true;
        options.extension.front_matter_delimiter = Some("---".to_owned());
        options.extension.autolink = true;
        options.extension.table = true;
        options.extension.tasklist = true;
        options.extension.superscript = true;
        options.extension.header_ids = Some("header-id-".to_string());
        options.extension.footnotes = true;


        let mut plugins = ComrakPlugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&*SYNTECT_ADAPTER);

        Ok(MDComrakSettings { options, plugins })
    }

    pub fn load_theme<R>(theme_name: &str, theme_cursor: &mut R) -> anyhow::Result<SyntectAdapter>
    where
        R: std::io::BufRead + std::io::Seek,
    {
        let theme = ThemeSet::load_from_reader(theme_cursor)?;
        let mut theme_set = ThemeSet::new();
        theme_set.themes.insert(String::from(theme_name), theme);
        let adapter = SyntectAdapterBuilder::new()
            .theme_set(theme_set)
            .theme(theme_name)
            .build();

        Ok(adapter)
    }
}