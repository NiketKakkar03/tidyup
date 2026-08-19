use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct TestFixture {
    root: PathBuf,
}

impl TestFixture {
    pub fn new(entries: &[FixtureEntry]) -> io::Result<Self> {
        let root = unique_fixture_root()?;

        for entry in entries {
            entry.create(&root)?;
        }

        Ok(Self { root })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root.join(relative)
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FixtureEntry {
    Directory { path: PathBuf },
    File { path: PathBuf, contents: Vec<u8> },
    Symlink { path: PathBuf, target: PathBuf },
}

impl FixtureEntry {
    #[must_use]
    pub fn directory(path: impl Into<PathBuf>) -> Self {
        Self::Directory { path: path.into() }
    }

    #[must_use]
    pub fn file(path: impl Into<PathBuf>, contents: impl Into<Vec<u8>>) -> Self {
        Self::File {
            path: path.into(),
            contents: contents.into(),
        }
    }

    #[must_use]
    pub fn symlink(path: impl Into<PathBuf>, target: impl Into<PathBuf>) -> Self {
        Self::Symlink {
            path: path.into(),
            target: target.into(),
        }
    }

    fn create(&self, root: &Path) -> io::Result<()> {
        match self {
            Self::Directory { path } => {
                fs::create_dir_all(root.join(path))?;
                Ok(())
            }
            Self::File { path, contents } => {
                let full_path = root.join(path);
                create_parent_dir(&full_path)?;
                fs::write(full_path, contents)
            }
            Self::Symlink { path, target } => {
                let full_path = root.join(path);
                create_parent_dir(&full_path)?;
                create_symlink(target, &full_path)
            }
        }
    }
}

#[derive(Debug)]
pub struct UnsupportedFeatureError {
    feature: &'static str,
}

impl UnsupportedFeatureError {
    #[must_use]
    pub fn symlinks() -> Self {
        Self {
            feature: "symlinks",
        }
    }
}

impl fmt::Display for UnsupportedFeatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the current platform does not support fixture {} in this testkit",
            self.feature
        )
    }
}

impl std::error::Error for UnsupportedFeatureError {}

pub fn try_create_symlink_fixture(
    path: impl Into<PathBuf>,
    target: impl Into<PathBuf>,
) -> Result<FixtureEntry, UnsupportedFeatureError> {
    if cfg!(unix) {
        Ok(FixtureEntry::symlink(path, target))
    } else {
        Err(UnsupportedFeatureError::symlinks())
    }
}

fn unique_fixture_root() -> io::Result<PathBuf> {
    let nonce = FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let candidate = std::env::temp_dir().join(format!(
        "tidyup-fixture-{}-{}-{}",
        std::process::id(),
        nanos,
        nonce
    ));

    fs::create_dir_all(&candidate)?;
    Ok(candidate)
}

fn create_parent_dir(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}

#[cfg(unix)]
fn create_symlink(target: &Path, path: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, path)
}

#[cfg(not(unix))]
fn create_symlink(_target: &Path, _path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "symlink fixtures are not supported on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::{FixtureEntry, TestFixture, try_create_symlink_fixture};

    #[test]
    fn creates_files_with_spaces_and_unicode_names() {
        let fixture = TestFixture::new(&[
            FixtureEntry::file("report final.txt", b"alpha"),
            FixtureEntry::file("receipts/résumé 2026.txt", b"beta"),
        ])
        .expect("fixture should be created");

        assert!(fixture.path("report final.txt").is_file());
        assert!(fixture.path("receipts/résumé 2026.txt").is_file());
    }

    #[test]
    fn supports_collision_shaped_layouts() {
        let fixture = TestFixture::new(&[
            FixtureEntry::file("incoming/report.txt", b"source"),
            FixtureEntry::file("Documents/report.txt", b"existing"),
        ])
        .expect("fixture should be created");

        assert!(fixture.path("incoming/report.txt").exists());
        assert!(fixture.path("Documents/report.txt").exists());
    }

    #[test]
    fn drops_fixture_directory_on_cleanup() {
        let root = {
            let fixture = TestFixture::new(&[FixtureEntry::file("sample.txt", b"hello")])
                .expect("fixture should be created");
            let root = fixture.root().to_path_buf();
            assert!(root.exists());
            root
        };

        assert!(!root.exists());
    }

    #[test]
    fn creates_symlink_fixtures_when_supported() {
        let symlink = match try_create_symlink_fixture("aliases/report.txt", "../report.txt") {
            Ok(entry) => entry,
            Err(_) => return,
        };
        let fixture = TestFixture::new(&[FixtureEntry::file("report.txt", b"hello"), symlink])
            .expect("fixture should be created");

        let metadata = std::fs::symlink_metadata(fixture.path("aliases/report.txt"))
            .expect("symlink metadata should exist");
        assert!(metadata.file_type().is_symlink());
    }

    #[test]
    fn supports_case_collision_layouts_where_filesystem_allows_them() {
        let fixture = TestFixture::new(&[
            FixtureEntry::file("Report.txt", b"upper"),
            FixtureEntry::file("report.txt", b"lower"),
        ])
        .expect("fixture should be created");

        let entries = std::fs::read_dir(fixture.root())
            .expect("fixture root should be readable")
            .count();

        if entries < 2 {
            return;
        }

        assert_eq!(
            std::fs::read(fixture.path("Report.txt")).expect("upper file should exist"),
            b"upper"
        );
        assert_eq!(
            std::fs::read(fixture.path("report.txt")).expect("lower file should exist"),
            b"lower"
        );
    }
}
