use std::fmt;

/// A typedef of the result returned by many methods.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Custom error type for database operations. This enum encapsulates
/// various error scenarios that can occur during database interactions,
/// such as connection issues, SQL errors, and schema version mismatches.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
  /// A Rusqlite error
  RusqliteError {
    /// The SQL query that caused the error
    query: String,
    /// The Rusqlite error that occurred
    err: rusqlite::Error,
  },
  /// A connection error, typically occurs when the
  /// database file cannot be accessed or created
  ConnectionError {
    /// The error that occurred during connection
    err: rusqlite::Error,
  },
  /// The connection is in an invalid state, such
  /// as being closed or not initialized
  InvalidConnection,
  /// An error indicating that the database schema version is invalid
  InvalidSchemaVersion {
    /// The expected schema version
    expected: u8,
    /// The actual schema version found in the database
    found: u8,
  }
}

impl Error {
  /// Creates a new `Error` from a Rusqlite error and the associated SQL query.
  /// This is a helper method to simplify error handling when executing SQL queries.
  /// # Arguments
  /// * `e` - The Rusqlite error that occurred.
  /// * `query` - The SQL query that caused the error.
  /// # Returns
  /// A new `Error` instance containing the query and the error.
  pub fn with_sql(e: rusqlite::Error, query: &str) -> Self {
    Self::RusqliteError { query: query.into(), err: e }
  }
}

impl From<rusqlite::Error> for Error {
  fn from(e: rusqlite::Error) -> Self {
    Self::RusqliteError { query: String::new(), err: e }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::RusqliteError { query, err } => write!(f, "Database error: {}. Query: {}", err, query),
      Self::ConnectionError { err } => write!(f, "Connection error: {}", err),
      Self::InvalidSchemaVersion { expected, found } => write!(f, "Invalid schema version: expected {}, found {}", expected, found),
      Self::InvalidConnection => write!(f, "Invalid connection state: connection is not initialized or has been closed"),
    }
  }
}