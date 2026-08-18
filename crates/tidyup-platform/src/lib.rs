use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum MoveError {
    AbsolutePathNotAllowed { path: PathBuf },
    DestinationEscapesRoot { path: PathBuf },
    SourceMissing { path: PathBuf },
    DestinationExists { path: PathBuf },
    Io(std::io::Error),
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbsolutePathNotAllowed { path } => {
                write!(
                    f,
                    "absolute path is not allowed for move input: {}",
                    path.display()
                )
            }
            Self::DestinationEscapesRoot { path } => {
                write!(f, "destination escapes selected root: {}", path.display())
            }
            Self::SourceMissing { path } => write!(f, "source file is missing: {}", path.display()),
            Self::DestinationExists { path } => {
                write!(f, "destination already exists: {}", path.display())
            }
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MoveError {}

pub fn move_file_within_root(
    root: &Path,
    source_relative_path: &Path,
    destination_relative_path: &Path,
) -> Result<(), MoveError> {
    if source_relative_path.is_absolute() {
        return Err(MoveError::AbsolutePathNotAllowed {
            path: source_relative_path.to_path_buf(),
        });
    }
    if destination_relative_path.is_absolute() {
        return Err(MoveError::AbsolutePathNotAllowed {
            path: destination_relative_path.to_path_buf(),
        });
    }
    if !is_safe_relative_path(destination_relative_path) {
        return Err(MoveError::DestinationEscapesRoot {
            path: destination_relative_path.to_path_buf(),
        });
    }

    let source_absolute = root.join(source_relative_path);
    let destination_absolute = root.join(destination_relative_path);

    if !source_absolute.exists() {
        return Err(MoveError::SourceMissing {
            path: source_absolute,
        });
    }
    if destination_absolute.exists() {
        return Err(MoveError::DestinationExists {
            path: destination_absolute,
        });
    }
    if let Some(parent) = destination_absolute.parent() {
        fs::create_dir_all(parent).map_err(MoveError::Io)?;
    }
    fs::rename(source_absolute, destination_absolute).map_err(MoveError::Io)
}

fn is_safe_relative_path(path: &Path) -> bool {
    !path
        .components()
        .any(|component| !matches!(component, std::path::Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::{MoveError, move_file_within_root};
    use tidyup_testkit::{FixtureEntry, TestFixture};

    #[test]
    fn move_creates_destination_parent_and_renames_file() {
        let fixture = TestFixture::new(&[FixtureEntry::file("todo.md", b"task")])
            .expect("fixture should exist");

        move_file_within_root(
            fixture.root(),
            "todo.md".as_ref(),
            "Documents/todo.md".as_ref(),
        )
        .expect("move should succeed");

        assert!(!fixture.path("todo.md").exists());
        assert!(fixture.path("Documents/todo.md").exists());
    }

    #[test]
    fn move_rejects_occupied_destination() {
        let fixture = TestFixture::new(&[
            FixtureEntry::file("todo.md", b"task"),
            FixtureEntry::file("Documents/todo.md", b"occupied"),
        ])
        .expect("fixture should exist");

        let error = move_file_within_root(
            fixture.root(),
            "todo.md".as_ref(),
            "Documents/todo.md".as_ref(),
        )
        .expect_err("move should fail");

        assert!(matches!(error, MoveError::DestinationExists { .. }));
    }
}
