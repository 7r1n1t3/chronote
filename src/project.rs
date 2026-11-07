//! User-made study timelines (stored as "projects"): a custom time unit,
//! events with study notes and attachments, plus on-disk storage.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub type ProjectId = u64;
pub type EventId = u64;

/// Name of the built-in demo shown in the menu.
pub const DEMO_NAME: &str = "Embryo";
/// File extensions accepted as attachments.
pub const ATTACHMENT_EXTENSIONS: [&str; 2] = ["pdf", "docx"];

const LIBRARY_FILE: &str = "library.json";
const ATTACHMENT_DIR: &str = "attachments";

/// How the menu orders its entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortMode {
    /// Oldest first; the demo always counts as the oldest.
    #[default]
    Created,
    Alphabetical,
    /// The user's own arrangement.
    Manual,
}

/// One row of the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Entry {
    Demo,
    Project(ProjectId),
}

/// Every project plus menu preferences; this is what gets saved.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Library {
    next_id: u64,
    pub sort: SortMode,
    /// Manual order of the menu entries.
    order: Vec<Entry>,
    pub projects: Vec<Project>,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            next_id: 1,
            sort: SortMode::default(),
            order: vec![Entry::Demo],
            projects: Vec::new(),
        }
    }
}

impl Library {
    /// Returns a fresh id, unique across projects and events.
    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id.max(1);
        self.next_id = id + 1;
        id
    }

    /// Creates a project and returns its id.
    pub fn create_project(&mut self, name: impl Into<String>, created: u64) -> ProjectId {
        let id = self.next_id();
        self.projects.push(Project {
            id,
            name: name.into(),
            unit: "week".to_owned(),
            length: 40.0,
            created,
            events: Vec::new(),
        });
        self.order.push(Entry::Project(id));
        id
    }

    #[must_use]
    pub fn project(&self, id: ProjectId) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == id)
    }

    pub fn project_mut(&mut self, id: ProjectId) -> Option<&mut Project> {
        self.projects.iter_mut().find(|p| p.id == id)
    }

    pub fn remove_project(&mut self, id: ProjectId) -> Option<Project> {
        let index = self.projects.iter().position(|p| p.id == id)?;
        self.order.retain(|e| *e != Entry::Project(id));
        Some(self.projects.remove(index))
    }

    /// Menu entries in the current sort order.
    #[must_use]
    pub fn entries(&self) -> Vec<Entry> {
        let mut entries = self.manual_order();
        match self.sort {
            SortMode::Manual => {}
            SortMode::Created => entries.sort_by_key(|e| match e {
                Entry::Demo => (0, 0),
                Entry::Project(id) => (self.project(*id).map_or(0, |p| p.created), *id),
            }),
            SortMode::Alphabetical => {
                entries.sort_by_cached_key(|e| self.name(*e).to_lowercase());
            }
        }
        entries
    }

    /// Display name of a menu entry.
    #[must_use]
    pub fn name(&self, entry: Entry) -> &str {
        match entry {
            Entry::Demo => DEMO_NAME,
            Entry::Project(id) => self.project(id).map_or("", |p| p.name.as_str()),
        }
    }

    /// Moves `entry` up (negative) or down (positive) in the manual order.
    pub fn move_entry(&mut self, entry: Entry, delta: isize) {
        let mut order = self.manual_order();
        if let Some(from) = order.iter().position(|e| *e == entry) {
            let to = from.saturating_add_signed(delta).min(order.len() - 1);
            let item = order.remove(from);
            order.insert(to, item);
        }
        self.order = order;
    }

    /// The saved manual order, repaired so it lists every entry exactly once.
    fn manual_order(&self) -> Vec<Entry> {
        let mut order: Vec<Entry> = Vec::with_capacity(self.projects.len() + 1);
        for entry in &self.order {
            let valid = match entry {
                Entry::Demo => true,
                Entry::Project(id) => self.project(*id).is_some(),
            };
            if valid && !order.contains(entry) {
                order.push(*entry);
            }
        }
        if !order.contains(&Entry::Demo) {
            order.insert(0, Entry::Demo);
        }
        for project in &self.projects {
            if !order.contains(&Entry::Project(project.id)) {
                order.push(Entry::Project(project.id));
            }
        }
        order
    }
}

/// A timeline measured in a custom unit (days, sprints, chapters...).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    /// Singular name of one time unit, e.g. "day".
    pub unit: String,
    /// Total length of the timeline in units.
    pub length: f64,
    /// Creation time, seconds since the Unix epoch.
    pub created: u64,
    #[serde(default)]
    pub events: Vec<Event>,
}

impl Project {
    #[must_use]
    pub fn event(&self, id: EventId) -> Option<&Event> {
        self.events.iter().find(|e| e.id == id)
    }

    pub fn event_mut(&mut self, id: EventId) -> Option<&mut Event> {
        self.events.iter_mut().find(|e| e.id == id)
    }

    pub fn remove_event(&mut self, id: EventId) -> Option<Event> {
        let index = self.events.iter().position(|e| e.id == id)?;
        Some(self.events.remove(index))
    }

    /// Assigns each event a row so that overlapping events never collide.
    /// Returns `(event id, lane)` pairs and the number of lanes used.
    #[must_use]
    pub fn lanes(&self) -> (Vec<(EventId, usize)>, usize) {
        // Instants get a little breathing room so dots don't touch.
        let gap = self.length.max(1.0) * 0.015;
        let mut sorted: Vec<&Event> = self.events.iter().collect();
        sorted.sort_by(|a, b| {
            a.kind
                .start()
                .total_cmp(&b.kind.start())
                .then(a.id.cmp(&b.id))
        });

        let mut lane_ends: Vec<f64> = Vec::new();
        let mut result = Vec::with_capacity(sorted.len());
        for event in sorted {
            let (start, end) = (event.kind.start(), event.kind.end());
            let lane = lane_ends
                .iter()
                .position(|&last| last + gap < start)
                .unwrap_or_else(|| {
                    lane_ends.push(f64::NEG_INFINITY);
                    lane_ends.len() - 1
                });
            lane_ends[lane] = end;
            result.push((event.id, lane));
        }
        (result, lane_ends.len())
    }
}

/// Something that happens on a project timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub kind: EventKind,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

/// Instantaneous events are points; lasting events span a range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    Instant { at: f64 },
    Lasting { start: f64, end: f64 },
}

impl EventKind {
    #[must_use]
    pub fn start(self) -> f64 {
        match self {
            Self::Instant { at } => at,
            Self::Lasting { start, end } => start.min(end),
        }
    }

    #[must_use]
    pub fn end(self) -> f64 {
        match self {
            Self::Instant { at } => at,
            Self::Lasting { start, end } => start.max(end),
        }
    }

    #[must_use]
    pub fn is_lasting(self) -> bool {
        matches!(self, Self::Lasting { .. })
    }

    /// Converts between kinds, keeping the event roughly where it was.
    #[must_use]
    pub fn toggled(self, default_span: f64) -> Self {
        match self {
            Self::Instant { at } => Self::Lasting {
                start: at,
                end: at + default_span,
            },
            Self::Lasting { start, .. } => Self::Instant { at: start },
        }
    }
}

/// A file copied into the app's data directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    /// Original file name, shown to the user.
    pub name: String,
    /// Path relative to the data directory.
    pub file: String,
}

/// Whether `path` has an accepted attachment extension.
#[must_use]
pub fn is_supported_attachment(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        ATTACHMENT_EXTENSIONS
            .iter()
            .any(|ok| e.eq_ignore_ascii_case(ok))
    })
}

/// Seconds since the Unix epoch.
#[must_use]
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Formats Unix seconds as an ISO date (`YYYY-MM-DD`, UTC).
#[must_use]
pub fn format_date(secs: u64) -> String {
    // Civil-from-days, Howard Hinnant's algorithm.
    let days = (secs / 86_400).cast_signed() + 719_468;
    let era = days.div_euclid(146_097);
    let doe = days.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Formats a position on a timeline, dropping needless decimals.
#[must_use]
pub fn format_units(value: f64) -> String {
    if (value - value.round()).abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        let s = format!("{value:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_owned()
    }
}

/// Failure to load the library file.
#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    Parse(serde_json::Error),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read timelines: {e}"),
            Self::Parse(e) => write!(f, "timelines file is damaged: {e}"),
        }
    }
}

impl std::error::Error for StoreError {}

/// Reads and writes the library and attachments under one directory.
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// The platform data directory, or `$CHRONOTE_DATA_DIR` when set.
    #[must_use]
    pub fn default_location() -> Option<Self> {
        if let Some(dir) = std::env::var_os("CHRONOTE_DATA_DIR") {
            return Some(Self::at(dir));
        }
        directories::ProjectDirs::from("dev", "chronote", "chronote")
            .map(|dirs| Self::at(dirs.data_dir()))
    }

    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn library_path(&self) -> PathBuf {
        self.root.join(LIBRARY_FILE)
    }

    /// Loads the library; a missing file yields an empty library.
    ///
    /// # Errors
    /// Fails when the file exists but cannot be read or parsed.
    pub fn load(&self) -> Result<Library, StoreError> {
        match fs::read(self.library_path()) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(StoreError::Parse),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Library::default()),
            Err(e) => Err(StoreError::Io(e)),
        }
    }

    /// Moves a damaged library file aside so it is not overwritten.
    ///
    /// # Errors
    /// Fails if the file cannot be renamed.
    pub fn quarantine(&self) -> io::Result<PathBuf> {
        let target = self.root.join(format!("{LIBRARY_FILE}.damaged-{}", now()));
        fs::rename(self.library_path(), &target)?;
        Ok(target)
    }

    /// Writes the library atomically (temp file, then rename).
    ///
    /// # Errors
    /// Fails if the directory or file cannot be written.
    pub fn save(&self, library: &Library) -> io::Result<()> {
        fs::create_dir_all(&self.root)?;
        let json = serde_json::to_vec_pretty(library).map_err(io::Error::other)?;
        let tmp = self.root.join(format!("{LIBRARY_FILE}.tmp"));
        fs::write(&tmp, json)?;
        fs::rename(tmp, self.library_path())
    }

    /// Absolute path of a stored attachment.
    #[must_use]
    pub fn attachment_path(&self, attachment: &Attachment) -> PathBuf {
        self.root.join(&attachment.file)
    }

    /// Copies `source` into the data directory for the given event.
    ///
    /// # Errors
    /// Fails for unsupported file types or when copying fails.
    pub fn import_attachment(
        &self,
        project: ProjectId,
        event: EventId,
        source: &Path,
    ) -> io::Result<Attachment> {
        if !is_supported_attachment(source) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "only PDF and DOCX files can be attached",
            ));
        }
        let name = source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("attachment")
            .to_owned();
        let dir = Path::new(ATTACHMENT_DIR)
            .join(project.to_string())
            .join(event.to_string());
        fs::create_dir_all(self.root.join(&dir))?;

        // Avoid clobbering an existing attachment with the same name.
        let mut relative = dir.join(&name);
        let mut n = 1;
        while self.root.join(&relative).exists() {
            relative = dir.join(format!("{n}-{name}"));
            n += 1;
        }
        fs::copy(source, self.root.join(&relative))?;
        Ok(Attachment {
            name,
            file: relative.to_string_lossy().replace('\\', "/"),
        })
    }

    /// Deletes a stored attachment file; missing files are ignored.
    ///
    /// # Errors
    /// Fails if an existing file cannot be removed.
    pub fn remove_attachment(&self, attachment: &Attachment) -> io::Result<()> {
        ignore_missing(fs::remove_file(self.attachment_path(attachment)))
    }

    /// Deletes all attachment files of a project.
    ///
    /// # Errors
    /// Fails if an existing directory cannot be removed.
    pub fn remove_project_files(&self, project: ProjectId) -> io::Result<()> {
        let dir = self.root.join(ATTACHMENT_DIR).join(project.to_string());
        ignore_missing(fs::remove_dir_all(dir))
    }

    /// Deletes all attachment files of one event.
    ///
    /// # Errors
    /// Fails if an existing directory cannot be removed.
    pub fn remove_event_files(&self, project: ProjectId, event: EventId) -> io::Result<()> {
        let dir = self
            .root
            .join(ATTACHMENT_DIR)
            .join(project.to_string())
            .join(event.to_string());
        ignore_missing(fs::remove_dir_all(dir))
    }
}

fn ignore_missing(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library() -> Library {
        let mut lib = Library::default();
        lib.create_project("Zebra", 300);
        lib.create_project("apple", 100);
        lib.create_project("Mango", 200);
        lib
    }

    #[test]
    fn sorts_by_creation_with_demo_first() {
        let lib = library();
        let names: Vec<_> = lib.entries().iter().map(|e| lib.name(*e)).collect();
        assert_eq!(names, ["Embryo", "apple", "Mango", "Zebra"]);
    }

    #[test]
    fn sorts_alphabetically_ignoring_case() {
        let mut lib = library();
        lib.sort = SortMode::Alphabetical;
        let names: Vec<_> = lib.entries().iter().map(|e| lib.name(*e)).collect();
        assert_eq!(names, ["apple", "Embryo", "Mango", "Zebra"]);
    }

    #[test]
    fn manual_order_moves_entries() {
        let mut lib = library();
        lib.sort = SortMode::Manual;
        let mango = lib.entries()[3];
        lib.move_entry(mango, -10);
        lib.move_entry(Entry::Demo, 1);
        let names: Vec<_> = lib.entries().iter().map(|e| lib.name(*e)).collect();
        assert_eq!(names, ["Mango", "Zebra", "Embryo", "apple"]);
        lib.move_entry(Entry::Demo, 99);
        assert_eq!(lib.entries().last(), Some(&Entry::Demo));
    }

    #[test]
    fn removing_a_project_drops_it_from_the_menu() {
        let mut lib = library();
        let id = lib.projects[0].id;
        assert!(lib.remove_project(id).is_some());
        assert!(!lib.entries().contains(&Entry::Project(id)));
        assert_eq!(lib.entries().len(), 3);
    }

    #[test]
    fn ids_are_unique() {
        let mut lib = library();
        let a = lib.next_id();
        let b = lib.next_id();
        assert_ne!(a, b);
        assert!(lib.projects.iter().all(|p| p.id != a && p.id != b));
    }

    fn event(id: EventId, kind: EventKind) -> Event {
        Event {
            id,
            title: format!("e{id}"),
            description: String::new(),
            kind,
            attachments: Vec::new(),
        }
    }

    #[test]
    fn overlapping_events_get_separate_lanes() {
        let mut lib = Library::default();
        let id = lib.create_project("p", 0);
        let project = lib.project_mut(id).unwrap();
        project.length = 100.0;
        project.events = vec![
            event(
                1,
                EventKind::Lasting {
                    start: 0.0,
                    end: 50.0,
                },
            ),
            event(
                2,
                EventKind::Lasting {
                    start: 10.0,
                    end: 20.0,
                },
            ),
            event(3, EventKind::Instant { at: 60.0 }),
            event(4, EventKind::Instant { at: 60.5 }),
        ];
        let (lanes, count) = project.lanes();
        let lane = |id| lanes.iter().find(|(e, _)| *e == id).unwrap().1;
        assert_ne!(lane(1), lane(2));
        assert_eq!(lane(3), 0);
        assert_ne!(lane(3), lane(4));
        assert_eq!(count, 2);
    }

    #[test]
    fn toggling_kind_keeps_position() {
        let lasting = EventKind::Instant { at: 4.0 }.toggled(2.0);
        assert_eq!(
            lasting,
            EventKind::Lasting {
                start: 4.0,
                end: 6.0
            }
        );
        assert_eq!(lasting.toggled(2.0), EventKind::Instant { at: 4.0 });
        let reversed = EventKind::Lasting {
            start: 9.0,
            end: 3.0,
        };
        assert!((reversed.start() - 3.0).abs() < f64::EPSILON);
        assert!((reversed.end() - 9.0).abs() < f64::EPSILON);
    }

    #[test]
    fn formats_dates_and_units() {
        assert_eq!(format_date(0), "1970-01-01");
        assert_eq!(format_date(1_794_009_600), "2026-11-07");
        assert_eq!(format_units(3.0), "3");
        assert_eq!(format_units(2.5), "2.5");
        assert_eq!(format_units(1.234), "1.23");
    }

    #[test]
    fn store_round_trips_and_tolerates_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::at(dir.path().join("data"));
        assert_eq!(store.load().unwrap().projects.len(), 0);

        let mut lib = library();
        lib.sort = SortMode::Manual;
        let pid = lib.projects[0].id;
        lib.project_mut(pid).unwrap().events.push(event(
            42,
            EventKind::Lasting {
                start: 1.0,
                end: 2.0,
            },
        ));
        store.save(&lib).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.projects, lib.projects);
        assert_eq!(loaded.sort, SortMode::Manual);
        assert_eq!(loaded.entries(), lib.entries());
    }

    #[test]
    fn damaged_file_is_reported_and_quarantined() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::at(dir.path());
        fs::write(dir.path().join(LIBRARY_FILE), b"{ not json").unwrap();
        assert!(matches!(store.load(), Err(StoreError::Parse(_))));
        let moved = store.quarantine().unwrap();
        assert!(moved.exists());
        assert_eq!(store.load().unwrap().projects.len(), 0);
    }

    #[test]
    fn attachments_are_copied_and_filtered() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::at(dir.path().join("data"));
        let pdf = dir.path().join("Report.PDF");
        fs::write(&pdf, b"%PDF-1.7").unwrap();
        let txt = dir.path().join("notes.txt");
        fs::write(&txt, b"hi").unwrap();

        let first = store.import_attachment(1, 2, &pdf).unwrap();
        let second = store.import_attachment(1, 2, &pdf).unwrap();
        assert_eq!(first.name, "Report.PDF");
        assert_ne!(first.file, second.file);
        assert_eq!(
            fs::read(store.attachment_path(&first)).unwrap(),
            b"%PDF-1.7"
        );
        assert!(store.import_attachment(1, 2, &txt).is_err());
        assert!(is_supported_attachment(Path::new("a.docx")));

        store.remove_attachment(&first).unwrap();
        assert!(!store.attachment_path(&first).exists());
        store.remove_project_files(1).unwrap();
        assert!(!store.attachment_path(&second).exists());
        store.remove_project_files(1).unwrap();
    }
}
