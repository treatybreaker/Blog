use tera::Tera;

pub mod markdown;
pub mod page_gen;

pub trait TemplateRenderer {
    fn render_template(&self, tera: &Tera) -> anyhow::Result<String>;
}


pub fn iter_dir<F>(path: &std::path::PathBuf, phandler: &F) -> anyhow::Result<()>
where
    F: Fn(&std::fs::DirEntry) -> anyhow::Result<()>,
{
    for item in std::fs::read_dir(path)? {
        let entry = item?;
        if entry.file_type()?.is_dir() {
            iter_dir(&entry.path(), phandler)?;
        } else {
            phandler(&entry)?;
        }
    }
    Ok(())
}
