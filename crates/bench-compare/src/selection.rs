//! What a run of a WebAssembly bench does: every case, a selection of cells, or a list of its cells
//! without running any case. The driver sends it as text, which reaches the module through an
//! import.

/// A selection that names no cell.
#[derive(Debug, thiserror::Error)]
pub enum SelectionError {
    /// A text that is no plan or no pattern.
    #[error("the selection {text:?} holds no op/key/crate")]
    Malformed {
        /// The text.
        text: String,
    },
    /// A pattern that no cell of the bench matches.
    #[error("no cell of the bench matches {pattern}")]
    Unmatched {
        /// The pattern.
        pattern: String,
    },
}

/// Separates the patterns of a selection.
const SEPARATOR: char = ',';

/// The part of a pattern that matches anything.
const ANY: &str = "*";

/// One pattern of a [`Selection`]: `op/key/crate`, where a part may be `*`. A crate's name may hold
/// a `/`.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Pattern {
    op: String,
    key: String,
    library: String,
    /// Whether a cell matched it.
    matched: bool,
}

impl Pattern {
    fn parse(text: &str) -> Result<Self, SelectionError> {
        let mut parts = text.splitn(3, '/');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(op), Some(key), Some(library)) => Ok(Self {
                op: op.trim().to_owned(),
                key: key.trim().to_owned(),
                library: library.trim().to_owned(),
                matched: false,
            }),
            _ => Err(SelectionError::Malformed {
                text: text.to_owned(),
            }),
        }
    }

    fn matches(&self, op: &str, key: &str, library: &str) -> bool {
        let part = |pattern: &str, value: &str| pattern == ANY || pattern == value;
        part(&self.op, op) && part(&self.key, key) && part(&self.library, library)
    }
}

/// Cells by operation, row and crate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    patterns: Vec<Pattern>,
}

impl Selection {
    /// The selection of the patterns in `text`, `op/key/crate` each, separated by commas. A part
    /// `*` matches anything.
    pub fn parse(text: &str) -> Result<Self, SelectionError> {
        Ok(Self {
            patterns: text
                .split(SEPARATOR)
                .map(Pattern::parse)
                .collect::<Result<_, _>>()?,
        })
    }

    /// The selection of `patterns`, each `op/key/crate`.
    pub fn of(patterns: impl IntoIterator<Item = String>) -> Result<Self, SelectionError> {
        Ok(Self {
            patterns: patterns
                .into_iter()
                .map(|pattern| Pattern::parse(&pattern))
                .collect::<Result<_, _>>()?,
        })
    }

    /// Whether the selection holds the cell `op/key/library`.
    #[must_use]
    pub fn holds(&self, op: &str, key: &str, library: &str) -> bool {
        self.patterns
            .iter()
            .any(|pattern| pattern.matches(op, key, library))
    }

    /// The patterns that match some cell of `cells`, each its operation, its row and its crate;
    /// `None` for no such pattern.
    #[must_use]
    pub fn narrowed(&self, cells: &[(String, String, String)]) -> Option<Self> {
        let patterns: Vec<Pattern> = self
            .patterns
            .iter()
            .filter(|pattern| {
                cells
                    .iter()
                    .any(|(op, key, library)| pattern.matches(op, key, library))
            })
            .cloned()
            .collect();
        (!patterns.is_empty()).then_some(Self { patterns })
    }

    /// Takes note of the cell `op/key/library`, which the bench has.
    pub fn note(&mut self, op: &str, key: &str, library: &str) {
        for pattern in &mut self.patterns {
            pattern.matched |= pattern.matches(op, key, library);
        }
    }

    /// Fails on a pattern that no cell matched since the selection was made: it names no cell of
    /// this bench.
    pub fn check_matched(&self) -> Result<(), SelectionError> {
        match self.patterns.iter().find(|pattern| !pattern.matched) {
            Some(pattern) => Err(SelectionError::Unmatched {
                pattern: format!("{}/{}/{}", pattern.op, pattern.key, pattern.library),
            }),
            None => Ok(()),
        }
    }

    /// The selection as [`parse`](Self::parse) reads it.
    #[must_use]
    pub fn text(&self) -> String {
        self.patterns
            .iter()
            .map(|pattern| format!("{}/{}/{}", pattern.op, pattern.key, pattern.library))
            .collect::<Vec<_>>()
            .join(&SEPARATOR.to_string())
    }
}

/// What a run of a bench does.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Plan {
    /// It measures every case.
    All,
    /// It measures the cells of a selection. The module stays whole: the code of a selected case is
    /// the code of a run of every case.
    Only(Selection),
    /// It lists its cells and measures none.
    List,
}

/// The text of [`Plan::All`].
const ALL: &str = "all";

/// The text of [`Plan::List`].
const LIST: &str = "list";

/// The text before the selection of [`Plan::Only`].
const ONLY: &str = "only:";

impl Plan {
    /// The plan as the module reads it.
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Self::All => ALL.to_owned(),
            Self::Only(selection) => format!("{ONLY}{}", selection.text()),
            Self::List => LIST.to_owned(),
        }
    }

    /// The plan of the text `text`.
    pub fn parse(text: &str) -> Result<Self, SelectionError> {
        match text {
            ALL => Ok(Self::All),
            LIST => Ok(Self::List),
            _ => match text.strip_prefix(ONLY) {
                Some(selection) => Ok(Self::Only(Selection::parse(selection)?)),
                None => Err(SelectionError::Malformed {
                    text: text.to_owned(),
                }),
            },
        }
    }
}
