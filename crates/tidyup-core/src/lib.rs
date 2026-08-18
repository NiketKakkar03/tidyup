use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

static PLAN_COUNTER: AtomicU64 = AtomicU64::new(1);
static OPERATION_COUNTER: AtomicU64 = AtomicU64::new(1);

pub const RULE_PACK_SCHEMA_VERSION: &str = "rule-pack/v1";
pub const PLAN_SCHEMA_VERSION: &str = "plan/v1";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PlanId(String);

impl PlanId {
    #[must_use]
    pub fn new() -> Self {
        let value = PLAN_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(format!("plan-{value:08}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct OperationId(String);

impl OperationId {
    #[must_use]
    pub fn new() -> Self {
        let value = OPERATION_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(format!("op-{value:08}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ActionId(String);

impl ActionId {
    fn new(index: usize) -> Self {
        Self(format!("action-{index:04}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileSnapshot {
    pub relative_path: PathBuf,
    pub file_name: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub modified_unix_seconds: Option<u64>,
}

impl FileSnapshot {
    pub fn from_root(root: &Path, path: &Path) -> Result<Self, ScanError> {
        let relative_path = path
            .strip_prefix(root)
            .map_err(|_| ScanError::RootContainmentViolation {
                root: root.to_path_buf(),
                path: path.to_path_buf(),
            })?
            .to_path_buf();
        let metadata = fs::metadata(path).map_err(|source| ScanError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let file_name = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| ScanError::UnsupportedName {
                path: path.to_path_buf(),
            })?
            .to_owned();
        let extension = path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .map(|value| value.to_ascii_lowercase());
        let modified_unix_seconds = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());

        Ok(Self {
            relative_path,
            file_name,
            extension,
            size_bytes: metadata.len(),
            modified_unix_seconds,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScanSkipReason {
    Directory,
    Symlink,
    NonFile,
    UnsupportedName,
    MetadataReadFailed,
}

impl ScanSkipReason {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Directory => "directory",
            Self::Symlink => "symlink",
            Self::NonFile => "non_file",
            Self::UnsupportedName => "unsupported_name",
            Self::MetadataReadFailed => "metadata_read_failed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkippedEntry {
    pub relative_path: PathBuf,
    pub reason: ScanSkipReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanReport {
    pub root: PathBuf,
    pub scanned_files: Vec<FileSnapshot>,
    pub skipped_entries: Vec<SkippedEntry>,
}

#[derive(Debug)]
pub enum ScanError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    RootContainmentViolation {
        root: PathBuf,
        path: PathBuf,
    },
    UnsupportedName {
        path: PathBuf,
    },
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "failed to read {}: {source}", path.display()),
            Self::RootContainmentViolation { root, path } => write!(
                f,
                "path {} is outside selected root {}",
                path.display(),
                root.display()
            ),
            Self::UnsupportedName { path } => {
                write!(f, "path {} uses a non-unicode file name", path.display())
            }
        }
    }
}

impl std::error::Error for ScanError {}

pub fn scan_root(root: &Path) -> Result<ScanReport, ScanError> {
    let mut scanned_files = Vec::new();
    let mut skipped_entries = Vec::new();

    for entry in fs::read_dir(root).map_err(|source| ScanError::Io {
        path: root.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| ScanError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(root)
            .map_or_else(|_| path.clone(), Path::to_path_buf);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => {
                skipped_entries.push(SkippedEntry {
                    relative_path,
                    reason: ScanSkipReason::MetadataReadFailed,
                });
                continue;
            }
        };
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            skipped_entries.push(SkippedEntry {
                relative_path,
                reason: ScanSkipReason::Symlink,
            });
            continue;
        }
        if file_type.is_dir() {
            skipped_entries.push(SkippedEntry {
                relative_path,
                reason: ScanSkipReason::Directory,
            });
            continue;
        }
        if !file_type.is_file() {
            skipped_entries.push(SkippedEntry {
                relative_path,
                reason: ScanSkipReason::NonFile,
            });
            continue;
        }
        match FileSnapshot::from_root(root, &path) {
            Ok(snapshot) => scanned_files.push(snapshot),
            Err(ScanError::UnsupportedName { .. }) => skipped_entries.push(SkippedEntry {
                relative_path,
                reason: ScanSkipReason::UnsupportedName,
            }),
            Err(error) => return Err(error),
        }
    }

    scanned_files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    skipped_entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    Ok(ScanReport {
        root: root.to_path_buf(),
        scanned_files,
        skipped_entries,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RulePackV1 {
    pub schema_version: String,
    pub pack_id: String,
    pub rules: Vec<ExtensionRule>,
}

impl RulePackV1 {
    #[must_use]
    pub fn built_in() -> Self {
        Self {
            schema_version: RULE_PACK_SCHEMA_VERSION.to_owned(),
            pack_id: "builtin/default".to_owned(),
            rules: vec![
                ExtensionRule::new(
                    "documents",
                    "Documents",
                    &["txt", "md", "pdf", "doc", "docx"],
                ),
                ExtensionRule::new("images", "Images", &["jpg", "jpeg", "png", "gif", "heic"]),
                ExtensionRule::new("spreadsheets", "Spreadsheets", &["csv", "xls", "xlsx"]),
                ExtensionRule::new("archives", "Archives", &["zip", "tar", "gz"]),
            ],
        }
    }

    pub fn validate(&self) -> Result<(), RulePackValidationError> {
        if self.schema_version != RULE_PACK_SCHEMA_VERSION {
            return Err(RulePackValidationError::UnsupportedSchemaVersion(
                self.schema_version.clone(),
            ));
        }
        if self.pack_id.trim().is_empty() {
            return Err(RulePackValidationError::EmptyPackId);
        }
        let mut seen_extensions = BTreeMap::new();
        for rule in &self.rules {
            rule.validate()?;
            for extension in &rule.extensions {
                if let Some(previous) =
                    seen_extensions.insert(extension.clone(), rule.rule_id.clone())
                {
                    return Err(RulePackValidationError::AmbiguousExtension {
                        extension: extension.clone(),
                        first_rule_id: previous,
                        second_rule_id: rule.rule_id.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionRule {
    pub rule_id: String,
    pub destination_dir: String,
    pub extensions: Vec<String>,
}

impl ExtensionRule {
    #[must_use]
    pub fn new(rule_id: &str, destination_dir: &str, extensions: &[&str]) -> Self {
        Self {
            rule_id: rule_id.to_owned(),
            destination_dir: destination_dir.to_owned(),
            extensions: extensions
                .iter()
                .map(|value| value.to_ascii_lowercase())
                .collect(),
        }
    }

    fn validate(&self) -> Result<(), RulePackValidationError> {
        if self.rule_id.trim().is_empty() {
            return Err(RulePackValidationError::EmptyRuleId);
        }
        if !is_valid_destination_component(&self.destination_dir) {
            return Err(RulePackValidationError::InvalidDestinationDir(
                self.destination_dir.clone(),
            ));
        }
        if self.extensions.is_empty() {
            return Err(RulePackValidationError::RuleWithoutExtensions(
                self.rule_id.clone(),
            ));
        }
        let mut local_set = BTreeSet::new();
        for extension in &self.extensions {
            if extension.trim().is_empty() {
                return Err(RulePackValidationError::EmptyExtension(
                    self.rule_id.clone(),
                ));
            }
            if !local_set.insert(extension.clone()) {
                return Err(RulePackValidationError::DuplicateExtensionInRule {
                    rule_id: self.rule_id.clone(),
                    extension: extension.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum RulePackValidationError {
    UnsupportedSchemaVersion(String),
    EmptyPackId,
    EmptyRuleId,
    InvalidDestinationDir(String),
    RuleWithoutExtensions(String),
    EmptyExtension(String),
    DuplicateExtensionInRule {
        rule_id: String,
        extension: String,
    },
    AmbiguousExtension {
        extension: String,
        first_rule_id: String,
        second_rule_id: String,
    },
}

impl fmt::Display for RulePackValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(f, "unsupported rule-pack schema version: {version}")
            }
            Self::EmptyPackId => write!(f, "rule-pack id must not be empty"),
            Self::EmptyRuleId => write!(f, "rule id must not be empty"),
            Self::InvalidDestinationDir(value) => {
                write!(f, "invalid destination directory: {value}")
            }
            Self::RuleWithoutExtensions(rule_id) => {
                write!(f, "rule {rule_id} must declare at least one extension")
            }
            Self::EmptyExtension(rule_id) => {
                write!(f, "rule {rule_id} must not contain empty extensions")
            }
            Self::DuplicateExtensionInRule { rule_id, extension } => {
                write!(f, "rule {rule_id} repeats extension {extension}")
            }
            Self::AmbiguousExtension {
                extension,
                first_rule_id,
                second_rule_id,
            } => write!(
                f,
                "extension {extension} is claimed by both {first_rule_id} and {second_rule_id}"
            ),
        }
    }
}

impl std::error::Error for RulePackValidationError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Classification {
    pub rule_id: String,
    pub destination_dir: String,
    pub reason: String,
}

pub fn classify_snapshot(snapshot: &FileSnapshot, pack: &RulePackV1) -> Option<Classification> {
    let extension = snapshot.extension.as_ref()?;
    for rule in &pack.rules {
        if rule.extensions.iter().any(|value| value == extension) {
            return Some(Classification {
                rule_id: rule.rule_id.clone(),
                destination_dir: rule.destination_dir.clone(),
                reason: format!("extension .{extension} matched rule {}", rule.rule_id),
            });
        }
    }
    None
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedMove {
    pub action_id: ActionId,
    pub source: FileSnapshot,
    pub destination_dir: String,
    pub destination_relative_path: PathBuf,
    pub rule_id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    pub schema_version: String,
    pub plan_id: PlanId,
    pub operation_id: OperationId,
    pub root: PathBuf,
    pub moves: Vec<PlannedMove>,
    pub skipped_files: Vec<PlanSkip>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanSkip {
    pub source_relative_path: PathBuf,
    pub reason_code: PlanSkipReason,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanSkipReason {
    NoMatchingRule,
    InvalidDestinationDir,
    DestinationExists,
    DuplicateDestination,
}

impl PlanSkipReason {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoMatchingRule => "no_matching_rule",
            Self::InvalidDestinationDir => "invalid_destination_dir",
            Self::DestinationExists => "destination_exists",
            Self::DuplicateDestination => "duplicate_destination",
        }
    }
}

pub fn build_plan(scan: &ScanReport, pack: &RulePackV1) -> Result<Plan, RulePackValidationError> {
    pack.validate()?;

    let mut moves = Vec::new();
    let mut skipped_files = Vec::new();
    let mut planned_destinations = BTreeSet::new();

    for snapshot in &scan.scanned_files {
        let Some(classification) = classify_snapshot(snapshot, pack) else {
            skipped_files.push(PlanSkip {
                source_relative_path: snapshot.relative_path.clone(),
                reason_code: PlanSkipReason::NoMatchingRule,
                detail: "no built-in rule matched this file".to_owned(),
            });
            continue;
        };

        if !is_valid_destination_component(&classification.destination_dir) {
            skipped_files.push(PlanSkip {
                source_relative_path: snapshot.relative_path.clone(),
                reason_code: PlanSkipReason::InvalidDestinationDir,
                detail: format!(
                    "destination {:?} is not safe inside the selected root",
                    classification.destination_dir
                ),
            });
            continue;
        }

        let destination_relative_path =
            PathBuf::from(&classification.destination_dir).join(&snapshot.file_name);
        let destination_absolute = scan.root.join(&destination_relative_path);

        if destination_absolute.exists() {
            skipped_files.push(PlanSkip {
                source_relative_path: snapshot.relative_path.clone(),
                reason_code: PlanSkipReason::DestinationExists,
                detail: format!(
                    "destination {} already exists",
                    destination_relative_path.display()
                ),
            });
            continue;
        }

        if !planned_destinations.insert(destination_relative_path.clone()) {
            skipped_files.push(PlanSkip {
                source_relative_path: snapshot.relative_path.clone(),
                reason_code: PlanSkipReason::DuplicateDestination,
                detail: format!(
                    "another action already targets {}",
                    destination_relative_path.display()
                ),
            });
            continue;
        }

        let action_id = ActionId::new(moves.len() + 1);
        moves.push(PlannedMove {
            action_id,
            source: snapshot.clone(),
            destination_dir: classification.destination_dir,
            destination_relative_path,
            rule_id: classification.rule_id,
            reason: classification.reason,
        });
    }

    Ok(Plan {
        schema_version: PLAN_SCHEMA_VERSION.to_owned(),
        plan_id: PlanId::new(),
        operation_id: OperationId::new(),
        root: scan.root.clone(),
        moves,
        skipped_files,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationReport {
    pub valid_actions: Vec<PlannedMove>,
    pub invalid_actions: Vec<InvalidAction>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidAction {
    pub action_id: ActionId,
    pub source_relative_path: PathBuf,
    pub reason_code: ValidationReasonCode,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationReasonCode {
    SourceMissing,
    SourceChanged,
    DestinationExists,
    InvalidDestination,
    DuplicateDestination,
}

impl ValidationReasonCode {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::SourceMissing => "source_missing",
            Self::SourceChanged => "source_changed",
            Self::DestinationExists => "destination_exists",
            Self::InvalidDestination => "invalid_destination",
            Self::DuplicateDestination => "duplicate_destination",
        }
    }
}

pub fn validate_plan(root: &Path, plan: &Plan) -> ValidationReport {
    let mut valid_actions = Vec::new();
    let mut invalid_actions = Vec::new();
    let mut claimed_destinations = BTreeSet::new();

    for planned_move in &plan.moves {
        let source_absolute = root.join(&planned_move.source.relative_path);
        if !source_absolute.exists() {
            invalid_actions.push(InvalidAction {
                action_id: planned_move.action_id.clone(),
                source_relative_path: planned_move.source.relative_path.clone(),
                reason_code: ValidationReasonCode::SourceMissing,
                detail: "source file is no longer present".to_owned(),
            });
            continue;
        }

        match FileSnapshot::from_root(root, &source_absolute) {
            Ok(current) if current != planned_move.source => {
                invalid_actions.push(InvalidAction {
                    action_id: planned_move.action_id.clone(),
                    source_relative_path: planned_move.source.relative_path.clone(),
                    reason_code: ValidationReasonCode::SourceChanged,
                    detail: "source metadata changed after planning".to_owned(),
                });
                continue;
            }
            Err(_) => {
                invalid_actions.push(InvalidAction {
                    action_id: planned_move.action_id.clone(),
                    source_relative_path: planned_move.source.relative_path.clone(),
                    reason_code: ValidationReasonCode::SourceMissing,
                    detail: "source file could not be re-read".to_owned(),
                });
                continue;
            }
            Ok(_) => {}
        }

        if !is_valid_destination_path(&planned_move.destination_relative_path) {
            invalid_actions.push(InvalidAction {
                action_id: planned_move.action_id.clone(),
                source_relative_path: planned_move.source.relative_path.clone(),
                reason_code: ValidationReasonCode::InvalidDestination,
                detail: "destination escapes or violates same-root safety rules".to_owned(),
            });
            continue;
        }

        let destination_absolute = root.join(&planned_move.destination_relative_path);
        if destination_absolute.exists() {
            invalid_actions.push(InvalidAction {
                action_id: planned_move.action_id.clone(),
                source_relative_path: planned_move.source.relative_path.clone(),
                reason_code: ValidationReasonCode::DestinationExists,
                detail: "destination is now occupied".to_owned(),
            });
            continue;
        }

        if !claimed_destinations.insert(planned_move.destination_relative_path.clone()) {
            invalid_actions.push(InvalidAction {
                action_id: planned_move.action_id.clone(),
                source_relative_path: planned_move.source.relative_path.clone(),
                reason_code: ValidationReasonCode::DuplicateDestination,
                detail: "another planned move still targets the same destination".to_owned(),
            });
            continue;
        }

        valid_actions.push(planned_move.clone());
    }

    ValidationReport {
        valid_actions,
        invalid_actions,
    }
}

#[must_use]
pub fn render_scan_json(scan: &ScanReport) -> String {
    let scanned_files = scan
        .scanned_files
        .iter()
        .map(|file| {
            format!(
                "{{\"relative_path\":{},\"file_name\":{},\"extension\":{},\"size_bytes\":{},\"modified_unix_seconds\":{}}}",
                json_string(&path_display(&file.relative_path)),
                json_string(&file.file_name),
                json_option_string(file.extension.as_deref()),
                file.size_bytes,
                json_option_u64(file.modified_unix_seconds)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let skipped_entries = scan
        .skipped_entries
        .iter()
        .map(|entry| {
            format!(
                "{{\"relative_path\":{},\"reason_code\":{}}}",
                json_string(&path_display(&entry.relative_path)),
                json_string(entry.reason.code())
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"root\":{},\"scanned_files\":[{}],\"skipped_entries\":[{}]}}",
        json_string(&path_display(&scan.root)),
        scanned_files,
        skipped_entries
    )
}

#[must_use]
pub fn render_plan_json(plan: &Plan, validation: &ValidationReport) -> String {
    let moves = plan
        .moves
        .iter()
        .map(|move_| {
            format!(
                "{{\"action_id\":{},\"source_relative_path\":{},\"destination_relative_path\":{},\"rule_id\":{},\"reason\":{}}}",
                json_string(move_.action_id.as_str()),
                json_string(&path_display(&move_.source.relative_path)),
                json_string(&path_display(&move_.destination_relative_path)),
                json_string(&move_.rule_id),
                json_string(&move_.reason)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let skipped = plan
        .skipped_files
        .iter()
        .map(|skip| {
            format!(
                "{{\"source_relative_path\":{},\"reason_code\":{},\"detail\":{}}}",
                json_string(&path_display(&skip.source_relative_path)),
                json_string(skip.reason_code.code()),
                json_string(&skip.detail)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let invalid = validation
        .invalid_actions
        .iter()
        .map(|action| {
            format!(
                "{{\"action_id\":{},\"source_relative_path\":{},\"reason_code\":{},\"detail\":{}}}",
                json_string(action.action_id.as_str()),
                json_string(&path_display(&action.source_relative_path)),
                json_string(action.reason_code.code()),
                json_string(&action.detail)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"schema_version\":{},\"plan_id\":{},\"operation_id\":{},\"root\":{},\"moves\":[{}],\"skipped_files\":[{}],\"validation\":{{\"valid_action_count\":{},\"invalid_actions\":[{}]}}}}",
        json_string(&plan.schema_version),
        json_string(plan.plan_id.as_str()),
        json_string(plan.operation_id.as_str()),
        json_string(&path_display(&plan.root)),
        moves,
        skipped,
        validation.valid_actions.len(),
        invalid
    )
}

fn is_valid_destination_component(value: &str) -> bool {
    !value.trim().is_empty()
        && !value.contains(std::path::MAIN_SEPARATOR)
        && value != "."
        && value != ".."
}

fn is_valid_destination_path(path: &Path) -> bool {
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
}

fn path_display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn json_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn json_option_string(value: Option<&str>) -> String {
    value.map_or_else(|| "null".to_owned(), json_string)
}

fn json_option_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "null".to_owned(), |number| number.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        ExtensionRule, FileSnapshot, PlanSkipReason, RulePackV1, RulePackValidationError,
        ScanSkipReason, build_plan, classify_snapshot, render_plan_json, render_scan_json,
        scan_root, validate_plan,
    };
    use std::fs;
    use tidyup_testkit::{FixtureEntry, TestFixture, try_create_symlink_fixture};

    #[test]
    fn file_snapshot_normalizes_extension_and_captures_metadata() {
        let fixture = TestFixture::new(&[FixtureEntry::file("Quarterly Notes.TXT", b"hello")])
            .expect("fixture should exist");

        let snapshot =
            FileSnapshot::from_root(fixture.root(), &fixture.path("Quarterly Notes.TXT"))
                .expect("snapshot should be captured");

        assert_eq!(
            snapshot.relative_path,
            std::path::PathBuf::from("Quarterly Notes.TXT")
        );
        assert_eq!(snapshot.file_name, "Quarterly Notes.TXT");
        assert_eq!(snapshot.extension.as_deref(), Some("txt"));
        assert_eq!(snapshot.size_bytes, 5);
    }

    #[test]
    fn scanner_only_includes_direct_child_files_and_skips_risky_entries() {
        let mut entries = vec![
            FixtureEntry::file("Quarterly Notes.txt", b"hello"),
            FixtureEntry::directory("Photos"),
            FixtureEntry::file("Photos/IMG_0001.JPG", b"nested"),
        ];
        if let Ok(link_entry) = try_create_symlink_fixture("Notes Shortcut", "Quarterly Notes.txt")
        {
            entries.push(link_entry);
        }
        let fixture = TestFixture::new(&entries).expect("fixture should exist");

        let report = scan_root(fixture.root()).expect("scan should succeed");

        assert_eq!(report.scanned_files.len(), 1);
        assert_eq!(report.scanned_files[0].file_name, "Quarterly Notes.txt");
        assert!(
            report
                .skipped_entries
                .iter()
                .any(|entry| entry.reason == ScanSkipReason::Directory)
        );
        if cfg!(unix) {
            assert!(
                report
                    .skipped_entries
                    .iter()
                    .any(|entry| entry.reason == ScanSkipReason::Symlink)
            );
        }
    }

    #[test]
    fn rule_pack_rejects_ambiguous_extensions() {
        let pack = RulePackV1 {
            schema_version: super::RULE_PACK_SCHEMA_VERSION.to_owned(),
            pack_id: "broken".to_owned(),
            rules: vec![
                ExtensionRule::new("one", "Docs", &["txt"]),
                ExtensionRule::new("two", "Text", &["txt"]),
            ],
        };

        assert_eq!(
            pack.validate(),
            Err(RulePackValidationError::AmbiguousExtension {
                extension: "txt".to_owned(),
                first_rule_id: "one".to_owned(),
                second_rule_id: "two".to_owned(),
            })
        );
    }

    #[test]
    fn planner_creates_moves_and_skips_existing_destinations() {
        let fixture = TestFixture::new(&[
            FixtureEntry::file("Quarterly Notes.txt", b"hello"),
            FixtureEntry::file("Documents/Quarterly Notes.txt", b"existing"),
            FixtureEntry::file("todo.md", b"todo"),
            FixtureEntry::file("unknown.bin", b"bin"),
        ])
        .expect("fixture should exist");

        let scan = scan_root(fixture.root()).expect("scan should succeed");
        let plan = build_plan(&scan, &RulePackV1::built_in()).expect("plan should build");

        assert_eq!(plan.moves.len(), 1);
        assert_eq!(
            plan.moves[0].destination_relative_path,
            std::path::PathBuf::from("Documents/todo.md")
        );
        assert!(
            plan.skipped_files
                .iter()
                .any(|skip| skip.reason_code == PlanSkipReason::DestinationExists)
        );
        assert!(
            plan.skipped_files
                .iter()
                .any(|skip| skip.reason_code == PlanSkipReason::NoMatchingRule)
        );
    }

    #[test]
    fn validation_detects_stale_sources_and_new_destination_collisions() {
        let fixture = TestFixture::new(&[
            FixtureEntry::file("todo.md", b"todo"),
            FixtureEntry::file("photo.jpg", b"jpg"),
        ])
        .expect("fixture should exist");

        let scan = scan_root(fixture.root()).expect("scan should succeed");
        let plan = build_plan(&scan, &RulePackV1::built_in()).expect("plan should build");

        fs::write(fixture.path("todo.md"), b"updated").expect("source should change");
        fs::write(fixture.path("Images/photo.jpg"), b"occupied").expect_err("parent missing");
        fs::create_dir_all(fixture.path("Images")).expect("images dir should exist");
        fs::write(fixture.path("Images/photo.jpg"), b"occupied").expect("destination should exist");

        let validation = validate_plan(fixture.root(), &plan);

        assert_eq!(validation.valid_actions.len(), 0);
        assert_eq!(validation.invalid_actions.len(), 2);
        assert!(
            validation
                .invalid_actions
                .iter()
                .any(|action| action.reason_code.code() == "source_changed")
        );
        assert!(
            validation
                .invalid_actions
                .iter()
                .any(|action| action.reason_code.code() == "destination_exists")
        );
    }

    #[test]
    fn json_renderers_include_stable_reason_codes() {
        let fixture = TestFixture::new(&[FixtureEntry::file("todo.md", b"todo")])
            .expect("fixture should exist");
        let scan = scan_root(fixture.root()).expect("scan should succeed");
        let plan = build_plan(&scan, &RulePackV1::built_in()).expect("plan should build");
        let validation = validate_plan(fixture.root(), &plan);

        let scan_json = render_scan_json(&scan);
        let plan_json = render_plan_json(&plan, &validation);

        assert!(scan_json.contains("\"scanned_files\""));
        assert!(plan_json.contains("\"validation\""));
        assert!(plan_json.contains("\"valid_action_count\":1"));
    }

    #[test]
    fn classification_is_deterministic_for_known_extension() {
        let snapshot = FileSnapshot {
            relative_path: "todo.md".into(),
            file_name: "todo.md".to_owned(),
            extension: Some("md".to_owned()),
            size_bytes: 4,
            modified_unix_seconds: Some(1),
        };

        let first = classify_snapshot(&snapshot, &RulePackV1::built_in());
        let second = classify_snapshot(&snapshot, &RulePackV1::built_in());

        assert_eq!(first, second);
    }
}
