use std::{
    collections::hash_map::DefaultHasher,
    ffi::OsStr,
    fs,
    hash::{Hash, Hasher},
    io,
    path::{Path, PathBuf},
    process::Command,
    time::UNIX_EPOCH,
};

use walkdir::{DirEntry, WalkDir};

use crate::markdown::{self, MarkdownBlock};

const MAX_PDF_PAGES: usize = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentKind {
    Markdown,
    Pdf,
}

impl DocumentKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Markdown => "MD",
            Self::Pdf => "PDF",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentEntry {
    pub path: PathBuf,
    pub relative_path: String,
    pub name: String,
    pub title: String,
    pub kind: DocumentKind,
}

#[derive(Clone, Debug)]
pub struct PdfPage {
    pub path: PathBuf,
    pub aspect_ratio: f32,
}

#[derive(Clone, Debug)]
pub struct PdfPreview {
    pub pages: Vec<PdfPage>,
    pub total_pages: Option<usize>,
}

#[derive(Clone, Debug)]
pub enum DocumentPreview {
    Empty,
    Markdown(Vec<MarkdownBlock>),
    Pdf(PdfPreview),
    Error(String),
}

pub fn discover(root: &Path) -> Vec<DocumentEntry> {
    let mut documents = WalkDir::new(root)
        .max_depth(8)
        .into_iter()
        .filter_entry(is_visible_entry)
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| entry_from_path(root, entry.into_path()))
        .collect::<Vec<_>>();

    documents.sort_by_key(|entry| entry.relative_path.to_lowercase());
    documents
}

pub fn load(entry: &DocumentEntry, cache_root: &Path) -> DocumentPreview {
    match entry.kind {
        DocumentKind::Markdown => match fs::read_to_string(&entry.path) {
            Ok(source) => DocumentPreview::Markdown(markdown::parse(&source)),
            Err(error) => DocumentPreview::Error(format!(
                "Could not read {}: {error}",
                entry.relative_path
            )),
        },
        DocumentKind::Pdf => match render_pdf(&entry.path, cache_root) {
            Ok(preview) => DocumentPreview::Pdf(preview),
            Err(error) if error.kind() == io::ErrorKind::NotFound => DocumentPreview::Error(
                "PDF rendering requires Poppler. Install it with `brew install poppler`, then restart Zuxi."
                    .to_owned(),
            ),
            Err(error) => DocumentPreview::Error(format!(
                "Could not render {}: {error}",
                entry.relative_path
            )),
        },
    }
}

fn entry_from_path(root: &Path, path: PathBuf) -> Option<DocumentEntry> {
    let extension = path.extension()?.to_string_lossy().to_lowercase();
    let kind = match extension.as_str() {
        "md" | "markdown" => DocumentKind::Markdown,
        "pdf" => DocumentKind::Pdf,
        _ => return None,
    };
    let name = path.file_name()?.to_string_lossy().into_owned();
    let fallback_title = path.file_stem()?.to_string_lossy().replace(['_', '-'], " ");
    let title = if kind == DocumentKind::Markdown {
        fs::read_to_string(&path)
            .ok()
            .and_then(|source| markdown::title(&source))
            .unwrap_or(fallback_title)
    } else {
        fallback_title
    };
    let relative_path = path
        .strip_prefix(root)
        .unwrap_or(&path)
        .to_string_lossy()
        .into_owned();

    Some(DocumentEntry {
        path,
        relative_path,
        name,
        title,
        kind,
    })
}

fn is_visible_entry(entry: &DirEntry) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_dir() {
        return true;
    }

    let name = entry.file_name().to_string_lossy();
    name != "target" && name != ".git" && !name.starts_with('.')
}

fn render_pdf(path: &Path, cache_root: &Path) -> io::Result<PdfPreview> {
    let cache_dir = cache_root.join(pdf_cache_key(path));
    fs::create_dir_all(&cache_dir)?;
    let prefix = cache_dir.join("page");
    let mut pages = rendered_pages(&cache_dir)?;

    if pages.is_empty() {
        let output = Command::new("pdftoppm")
            .arg("-png")
            .arg("-r")
            .arg("120")
            .arg("-f")
            .arg("1")
            .arg("-l")
            .arg(MAX_PDF_PAGES.to_string())
            .arg(path)
            .arg(&prefix)
            .output()?;

        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(io::Error::other(if message.is_empty() {
                "pdftoppm exited without producing an image".to_owned()
            } else {
                message
            }));
        }
        pages = rendered_pages(&cache_dir)?;
    }

    if pages.is_empty() {
        return Err(io::Error::other("the PDF contained no renderable pages"));
    }

    let pages = pages
        .into_iter()
        .map(|path| {
            let aspect_ratio = imagesize::size(&path)
                .map(|size| size.width as f32 / size.height as f32)
                .unwrap_or(1.0);
            PdfPage { path, aspect_ratio }
        })
        .collect();

    Ok(PdfPreview {
        pages,
        total_pages: pdf_page_count(path),
    })
}

fn rendered_pages(cache_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut pages = fs::read_dir(cache_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension() == Some(OsStr::new("png")))
        .collect::<Vec<_>>();
    pages.sort_by_key(|path| {
        path.file_stem()
            .and_then(OsStr::to_str)
            .and_then(|stem| stem.rsplit('-').next())
            .and_then(|page| page.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    });
    Ok(pages)
}

fn pdf_cache_key(path: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    if let Ok(metadata) = fs::metadata(path) {
        metadata.len().hash(&mut hasher);
        if let Ok(modified) = metadata.modified()
            && let Ok(duration) = modified.duration_since(UNIX_EPOCH)
        {
            duration.as_nanos().hash(&mut hasher);
        }
    }
    format!("{:016x}", hasher.finish())
}

fn pdf_page_count(path: &Path) -> Option<usize> {
    let output = Command::new("pdfinfo").arg(path).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("Pages:"))
        .and_then(|pages| pages.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_supported_files_and_ignores_build_folders() {
        let directory = tempfile::tempdir().expect("tempdir");
        fs::write(directory.path().join("readme.md"), "# Read me").expect("markdown");
        fs::write(directory.path().join("paper.PDF"), "%PDF").expect("pdf");
        fs::write(directory.path().join("notes.txt"), "ignored").expect("text");
        fs::create_dir(directory.path().join("target")).expect("target directory");
        fs::write(directory.path().join("target/hidden.md"), "ignored").expect("hidden");

        let documents = discover(directory.path());

        assert_eq!(documents.len(), 2);
        assert!(
            documents
                .iter()
                .any(|entry| entry.kind == DocumentKind::Markdown)
        );
        assert!(
            documents
                .iter()
                .any(|entry| entry.kind == DocumentKind::Pdf)
        );
        let markdown_entry = documents
            .iter()
            .find(|entry| entry.kind == DocumentKind::Markdown)
            .expect("discovered markdown");
        assert_eq!(markdown_entry.title, "Read me");
        match load(markdown_entry, directory.path()) {
            DocumentPreview::Markdown(blocks) => {
                assert_eq!(blocks[0].text, "Read me");
            }
            preview => panic!("expected Markdown preview, got {preview:?}"),
        }
    }
}
