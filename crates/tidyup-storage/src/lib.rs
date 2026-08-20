use std::fmt::{self, Write as _};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tidyup_core::{ActionExecutionStatus, ExecutionReport, Plan};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationRecord {
    pub operation_id: String,
    pub plan_id: String,
    pub root_path: PathBuf,
    pub applied_at_unix_seconds: u64,
    pub completed_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionResultRecord {
    pub operation_id: String,
    pub action_id: String,
    pub source_relative_path: PathBuf,
    pub destination_relative_path: PathBuf,
    pub status_code: String,
    pub reason_code: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
    Utf8(std::string::FromUtf8Error),
    SqliteCommandFailed(String),
    Parse(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Utf8(error) => write!(f, "{error}"),
            Self::SqliteCommandFailed(message) | Self::Parse(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<std::io::Error> for StorageError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<std::string::FromUtf8Error> for StorageError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::Utf8(value)
    }
}

#[must_use]
pub fn default_history_db_path(root: &Path) -> PathBuf {
    root.join(".tidyup").join("history.sqlite3")
}

/// Ensures the history database exists and has the expected schema.
///
/// # Errors
///
/// Returns [`StorageError`] when the database directory cannot be created or
/// the schema initialization SQL fails.
pub fn initialize_history_db(db_path: &Path) -> Result<(), StorageError> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    run_sql(
        db_path,
        r"
        PRAGMA journal_mode=WAL;
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at_unix_seconds INTEGER NOT NULL
        );
        INSERT OR IGNORE INTO schema_migrations(version, applied_at_unix_seconds)
        VALUES (1, unixepoch());

        CREATE TABLE IF NOT EXISTS operations (
            operation_id TEXT PRIMARY KEY,
            plan_id TEXT NOT NULL,
            root_path TEXT NOT NULL,
            applied_at_unix_seconds INTEGER NOT NULL,
            completed_count INTEGER NOT NULL,
            skipped_count INTEGER NOT NULL,
            failed_count INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS action_results (
            operation_id TEXT NOT NULL,
            action_id TEXT NOT NULL,
            source_relative_path TEXT NOT NULL,
            destination_relative_path TEXT NOT NULL,
            status_code TEXT NOT NULL,
            reason_code TEXT,
            detail TEXT,
            PRIMARY KEY (operation_id, action_id),
            FOREIGN KEY (operation_id) REFERENCES operations(operation_id)
        );
        ",
    )?;

    Ok(())
}

/// Persists an executed plan and all action results into the history database.
///
/// # Errors
///
/// Returns [`StorageError`] when the database cannot be initialized or the
/// execution records cannot be written successfully.
pub fn persist_execution(
    db_path: &Path,
    plan: &Plan,
    execution: &ExecutionReport,
) -> Result<(), StorageError> {
    initialize_history_db(db_path)?;

    let operation_sql = format!(
        r"
        BEGIN IMMEDIATE;
        INSERT INTO operations(
            operation_id,
            plan_id,
            root_path,
            applied_at_unix_seconds,
            completed_count,
            skipped_count,
            failed_count
        ) VALUES (
            '{operation_id}',
            '{plan_id}',
            '{root_path}',
            unixepoch(),
            {completed_count},
            {skipped_count},
            {failed_count}
        );
        ",
        operation_id = sql_escape(execution.operation_id.as_str()),
        plan_id = sql_escape(plan.plan_id.as_str()),
        root_path = sql_escape(&plan.root.to_string_lossy()),
        completed_count = execution.completed_count(),
        skipped_count = execution.skipped_count(),
        failed_count = execution.failed_count(),
    );

    let mut sql = operation_sql;
    for result in &execution.results {
        let (status_code, reason_code, detail) = match &result.status {
            ActionExecutionStatus::Completed => ("completed", None, None),
            ActionExecutionStatus::Skipped {
                reason_code,
                detail,
            } => (
                "skipped",
                Some(reason_code.code().to_owned()),
                Some(detail.clone()),
            ),
            ActionExecutionStatus::Failed { detail } => ("failed", None, Some(detail.clone())),
        };

        let _ = write!(
            sql,
            r"
            INSERT INTO action_results(
                operation_id,
                action_id,
                source_relative_path,
                destination_relative_path,
                status_code,
                reason_code,
                detail
            ) VALUES (
                '{operation_id}',
                '{action_id}',
                '{source_path}',
                '{destination_path}',
                '{status_code}',
                {reason_code},
                {detail}
            );
            ",
            operation_id = sql_escape(execution.operation_id.as_str()),
            action_id = sql_escape(result.action_id.as_str()),
            source_path = sql_escape(&result.source_relative_path.to_string_lossy()),
            destination_path = sql_escape(&result.destination_relative_path.to_string_lossy()),
            status_code = sql_escape(status_code),
            reason_code = sql_option_literal(reason_code.as_deref()),
            detail = sql_option_literal(detail.as_deref()),
        );
    }
    sql.push_str("COMMIT;");

    run_sql(db_path, &sql)
}

/// Loads recorded operations for the selected history database.
///
/// # Errors
///
/// Returns [`StorageError`] when the database cannot be initialized, queried,
/// or parsed into operation records.
pub fn load_operations(db_path: &Path) -> Result<Vec<OperationRecord>, StorageError> {
    initialize_history_db(db_path)?;
    let output = query_sql(
        db_path,
        r"
        SELECT
            operation_id,
            plan_id,
            root_path,
            applied_at_unix_seconds,
            completed_count,
            skipped_count,
            failed_count
        FROM operations
        ORDER BY applied_at_unix_seconds DESC, operation_id DESC;
        ",
    )?;

    parse_operations(&output)
}

/// Loads action-level results for a single recorded operation.
///
/// # Errors
///
/// Returns [`StorageError`] when the database cannot be initialized, queried,
/// or parsed into action result records.
pub fn load_action_results(
    db_path: &Path,
    operation_id: &str,
) -> Result<Vec<ActionResultRecord>, StorageError> {
    initialize_history_db(db_path)?;
    let sql = format!(
        r"
        SELECT
            operation_id,
            action_id,
            source_relative_path,
            destination_relative_path,
            status_code,
            COALESCE(reason_code, ''),
            COALESCE(detail, '')
        FROM action_results
        WHERE operation_id = '{operation_id}'
        ORDER BY action_id ASC;
        ",
        operation_id = sql_escape(operation_id)
    );
    let output = query_sql(db_path, &sql)?;
    parse_action_results(&output)
}

fn parse_operations(output: &str) -> Result<Vec<OperationRecord>, StorageError> {
    let mut records = Vec::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let columns: Vec<_> = line.split('\t').collect();
        if columns.len() != 7 {
            return Err(StorageError::Parse(format!(
                "expected 7 columns for operation row, found {}",
                columns.len()
            )));
        }
        records.push(OperationRecord {
            operation_id: columns[0].to_owned(),
            plan_id: columns[1].to_owned(),
            root_path: PathBuf::from(columns[2]),
            applied_at_unix_seconds: columns[3].parse().map_err(|_| {
                StorageError::Parse(format!(
                    "invalid applied_at_unix_seconds value: {}",
                    columns[3]
                ))
            })?,
            completed_count: columns[4].parse().map_err(|_| {
                StorageError::Parse(format!("invalid completed_count value: {}", columns[4]))
            })?,
            skipped_count: columns[5].parse().map_err(|_| {
                StorageError::Parse(format!("invalid skipped_count value: {}", columns[5]))
            })?,
            failed_count: columns[6].parse().map_err(|_| {
                StorageError::Parse(format!("invalid failed_count value: {}", columns[6]))
            })?,
        });
    }
    Ok(records)
}

fn parse_action_results(output: &str) -> Result<Vec<ActionResultRecord>, StorageError> {
    let mut records = Vec::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let columns: Vec<_> = line.split('\t').collect();
        if columns.len() != 7 {
            return Err(StorageError::Parse(format!(
                "expected 7 columns for action row, found {}",
                columns.len()
            )));
        }
        records.push(ActionResultRecord {
            operation_id: columns[0].to_owned(),
            action_id: columns[1].to_owned(),
            source_relative_path: PathBuf::from(columns[2]),
            destination_relative_path: PathBuf::from(columns[3]),
            status_code: columns[4].to_owned(),
            reason_code: empty_string_to_none(columns[5]),
            detail: empty_string_to_none(columns[6]),
        });
    }
    Ok(records)
}

fn empty_string_to_none(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_owned())
}

fn sql_escape(value: &str) -> String {
    value.replace('\'', "''")
}

fn sql_option_literal(value: Option<&str>) -> String {
    value.map_or_else(
        || "NULL".to_owned(),
        |inner| format!("'{}'", sql_escape(inner)),
    )
}

fn run_sql(db_path: &Path, sql: &str) -> Result<(), StorageError> {
    let output = Command::new("sqlite3").arg(db_path).arg(sql).output()?;

    if output.status.success() {
        return Ok(());
    }

    Err(StorageError::SqliteCommandFailed(String::from_utf8(
        output.stderr,
    )?))
}

fn query_sql(db_path: &Path, sql: &str) -> Result<String, StorageError> {
    let output = Command::new("sqlite3")
        .arg("-tabs")
        .arg(db_path)
        .arg(sql)
        .output()?;

    if output.status.success() {
        return Ok(String::from_utf8(output.stdout)?);
    }

    Err(StorageError::SqliteCommandFailed(String::from_utf8(
        output.stderr,
    )?))
}

#[cfg(test)]
mod tests {
    use super::{
        default_history_db_path, initialize_history_db, load_action_results, load_operations,
        persist_execution,
    };
    use tidyup_core::{RulePackV1, build_plan, execute_plan, scan_root};
    use tidyup_testkit::{FixtureEntry, TestFixture};

    #[test]
    fn creates_history_db_and_persists_operation_results() {
        let fixture = TestFixture::new(&[
            FixtureEntry::file("todo.md", b"todo"),
            FixtureEntry::file("photo.jpg", b"jpg"),
        ])
        .expect("fixture should exist");
        let db_path = default_history_db_path(fixture.root());
        initialize_history_db(&db_path).expect("db should initialize");

        let scan = scan_root(fixture.root()).expect("scan should succeed");
        let plan = build_plan(&scan, &RulePackV1::built_in()).expect("plan should build");
        let execution = execute_plan(fixture.root(), &plan);
        persist_execution(&db_path, &plan, &execution).expect("execution should persist");

        let operations = load_operations(&db_path).expect("operations should load");
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].completed_count, 2);

        let action_results = load_action_results(&db_path, &operations[0].operation_id)
            .expect("action results should load");
        assert_eq!(action_results.len(), 2);
        assert!(
            action_results
                .iter()
                .all(|record| record.status_code == "completed")
        );
    }
}
