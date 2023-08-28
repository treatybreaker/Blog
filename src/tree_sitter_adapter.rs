/// As a future note to myself. This shit is REALLY cool, but it seems there's some bugs on
/// treesitter's side when it comes to applying highlighting to javascript and that Neovim's
/// treesitter implementation is MUCH better than what you get out of the box from the treesitter
/// project. One on front, this is cool as fuck, on another front, this was a total pain in the
/// ass. Maybe try this later and next time don't go down the rabbit hole of binding rust to Neovim
/// to get their parsers and queries for our own use. Did it work? Yep. Was it fast? Fuck no. Could
/// it have been fast? Probably, but I'm not wasting the time to write the ffi to Neovim only for
/// some fucker over there to change an interface or nuke one; I'm not gonna put up the maintenance
/// effort.
///
/// Full credit for a decent chunk of this to Andrew Biehl. His blog post
/// https://andrewtbiehl.com/blog/jekyll-tree-sitter is what sent me down this horrid path along
/// with some of his code in his kramdown syntax module. I did *significantly* speed this code up
/// though, as in several magnitudes for my purposes.

use std::{
    collections::HashMap,
    convert,
    io::{self, Write},
    path::PathBuf,
};

use anyhow::Context;
use comrak::adapters::SyntaxHighlighterAdapter;
use tree_sitter::Language;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};
use tree_sitter_loader::{Config, LanguageConfiguration, Loader};

pub struct TreesitterSyntaxAdapter {
    parsers_directory: PathBuf,
}

impl TreesitterSyntaxAdapter {
    pub fn new(parsers_directory: PathBuf) -> anyhow::Result<Self> {
        Ok(TreesitterSyntaxAdapter { parsers_directory })
    }

    pub fn get_loader(parsers_directory: &PathBuf) -> anyhow::Result<Loader> {
        Loader::new()
            .and_then(|mut loader| {
                let config = {
                    let parsers_directory = parsers_directory.clone();
                    let parser_directories = vec![parsers_directory];
                    Config { parser_directories }
                };
                loader.find_all_languages(&config)?;
                Ok(loader)
            })
            .with_context(|| {
                let parser_directory_str = parsers_directory.display();
                format!(
                    "Failed to load Treesitter parsers from directory: '{parser_directory_str}'"
                )
            })
    }

    fn get_language_config<'a>(
        loader: &'a Loader,
        code_lang: &'a str,
    ) -> anyhow::Result<(Language, &'a LanguageConfiguration<'a>)> {
        loader
            .language_configuration_for_scope(code_lang)
            .transpose()
            .context("Language configuration not found")
            .and_then(convert::identity)
            .with_context(|| format!("Failed to retrieve Treesitter language configuration for language: '{code_lang}'"))
    }

    fn get_highlight_configuration<'a>(
        language: Language,
        config: &'a LanguageConfiguration<'a>,
        scope: &'a str,
    ) -> anyhow::Result<&'a HighlightConfiguration> {
        config
            .highlight_config(language)
            .transpose()
            .with_context(|| {
                format!("Failed to retrieve Treesitter highlights for language: '{scope}'")
            })
            .and_then(convert::identity)
    }

    fn get_highlights<'a>(
        &'a self,
        loader: &'a Loader,
        highlighter: &'a mut Highlighter,
        code: &'a str,
        code_lang: &'a str,
    ) -> anyhow::Result<
        impl Iterator<Item = Result<HighlightEvent, tree_sitter_highlight::Error>> + 'a,
    > {
        let (language, lang_config) = Self::get_language_config(&loader, code_lang)?;
        let highlight_config = Self::get_highlight_configuration(language, lang_config, code_lang)?;
        let highlights =
            highlighter.highlight(&highlight_config, code.as_bytes(), None, |lang| loader.highlight_config_for_injection_string(lang))?;
        // loader.configure_highlights(&highlight_config.names().to_vec());

        Ok(highlights)
    }

    pub fn determine_ts_scope_name(code_lang: &str) -> String {
        match code_lang {
            _ => format!("source.{}", code_lang),
        }
    }
}

impl SyntaxHighlighterAdapter for TreesitterSyntaxAdapter {

    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> io::Result<()> {
        let scope = if let Some(lang) = lang {
            if lang.is_empty() {
                return Ok(());
            }
            TreesitterSyntaxAdapter::determine_ts_scope_name(lang)
        } else {
            return Ok(());
        };

        let loader = TreesitterSyntaxAdapter::get_loader(&self.parsers_directory).unwrap();
        let mut highlighter = Highlighter::new();
        let highlights = self
            .get_highlights(&loader, &mut highlighter, code, scope.as_str())
            .unwrap();

        let highlight_names = loader.highlight_names();
        println!("AT LANGUAGE -> {}", scope);
        println!("HIGHLIGHTS: {:#?}", highlight_names);

        for event in highlights {
            match event.unwrap() {
                HighlightEvent::Source { start, end } => {
                    write!(output, "{}", String::from(&code[start..end]))?
                }
                HighlightEvent::HighlightStart(s) => write!(
                    output,
                    "<span class=\"ts-{}\">",
                    highlight_names[s.0].replace(".", "-")
                )?,
                HighlightEvent::HighlightEnd => write!(output, "</span>")?,
            }
        }

        Ok(())
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        if attributes.contains_key("lang") {
            write!(output, "<pre lang=\"{}\">", attributes["lang"])
        } else {
            output.write_all(b"<pre>")
        }
    }

    fn write_code_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        if attributes.contains_key("class") {
            write!(output, "<code class=\"{}\">", attributes["class"])
        } else {
            output.write_all(b"<code>")
        }
    }
}
