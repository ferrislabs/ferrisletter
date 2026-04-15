//! Generic user state storage — preferences, subscriptions, read tracking.
//!
//! The [`UserStore`] trait defines the interface. Two implementations:
//!
//! - [`InMemoryUserStore`] — for stdio/local mode, with optional JSON file
//!   persistence at `~/.config/ferrisletter/users.json`.
//! - External crates (e.g. Lattice) can provide a database-backed implementation
//!   via [`BoxedUserStore`].

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

// ── Types ──────────────────────────────────────────────────────────────────

/// A user identity record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Full user state — identity plus subscriptions and preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user: User,
    pub topics: Vec<String>,
    pub tags: Vec<String>,
    pub preferences: HashMap<String, String>,
}

/// Internal row: all of a user's state stored together.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct UserRow {
    email: Option<String>,
    name: Option<String>,
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    topics: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    preferences: HashMap<String, String>,
    #[serde(default)]
    read_items: HashSet<String>,
}

// ── Trait ──────────────────────────────────────────────────────────────────

/// Trait for storing user state (preferences, subscriptions, read tracking).
///
/// Ferrisletter provides [`InMemoryUserStore`] (file-backed for stdio mode).
/// Lattice (and other downstreams) can provide a database-backed implementation.
pub trait UserStore: Send + Sync {
    // ── User identity ────────────────────────────────────────────────────

    fn upsert_user(
        &self,
        id: &str,
        email: Option<&str>,
        name: Option<&str>,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

    fn get_user(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<Option<User>>> + Send;

    // ── Topic subscriptions ──────────────────────────────────────────────

    fn get_topics(
        &self,
        user_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<String>>> + Send;

    fn set_topics(
        &self,
        user_id: &str,
        topics: &[String],
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

    // ── Tag subscriptions ────────────────────────────────────────────────

    fn get_tags(
        &self,
        user_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<String>>> + Send;

    fn set_tags(
        &self,
        user_id: &str,
        tags: &[String],
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

    // ── Key-value preferences ────────────────────────────────────────────

    fn get_preference(
        &self,
        user_id: &str,
        key: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<Option<String>>> + Send;

    fn set_preference(
        &self,
        user_id: &str,
        key: &str,
        value: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

    fn get_all_preferences(
        &self,
        user_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<HashMap<String, String>>> + Send;

    // ── Read tracking ────────────────────────────────────────────────────

    fn mark_read(
        &self,
        user_id: &str,
        item_ids: &[String],
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

    fn is_read(
        &self,
        user_id: &str,
        item_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<bool>> + Send;

    fn unread_count(
        &self,
        user_id: &str,
        candidate_item_ids: &[String],
    ) -> impl std::future::Future<Output = anyhow::Result<usize>> + Send;

    // ── Profile convenience ──────────────────────────────────────────────

    fn get_profile(
        &self,
        user_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<Option<UserProfile>>> + Send;
}

// ── In-memory implementation ───────────────────────────────────────────────

/// In-memory user store with optional JSON file persistence.
///
/// When `file_path` is `Some`, changes are flushed to disk after every
/// mutation so that user state survives server restarts.
pub struct InMemoryUserStore {
    /// user_id → UserRow.
    data: Arc<RwLock<HashMap<String, UserRow>>>,
    file_path: Option<PathBuf>,
}

impl InMemoryUserStore {
    /// Create a new store. If `file_path` is provided and the file exists,
    /// previously saved user state is loaded from it.
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let data = if let Some(ref path) = file_path {
            Self::load_from_file(path).unwrap_or_default()
        } else {
            HashMap::new()
        };
        Self {
            data: Arc::new(RwLock::new(data)),
            file_path,
        }
    }

    /// Default path: `~/.config/ferrisletter/users.json`.
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("ferrisletter").join("users.json"))
    }

    fn load_from_file(path: &std::path::Path) -> Option<HashMap<String, UserRow>> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    async fn flush(&self) {
        let Some(ref path) = self.file_path else {
            return;
        };
        let snapshot = self.data.read().await.clone();
        let path = path.clone();
        tokio::task::spawn_blocking(move || {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match serde_json::to_string_pretty(&snapshot) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&path, json) {
                        tracing::warn!("failed to write users file: {e}");
                    }
                }
                Err(e) => tracing::warn!("failed to serialize users: {e}"),
            }
        });
    }

    fn row_to_user(id: &str, row: &UserRow) -> User {
        User {
            id: id.to_string(),
            email: row.email.clone(),
            name: row.name.clone(),
            created_at: row.created_at.unwrap_or_else(Utc::now),
        }
    }
}

impl UserStore for InMemoryUserStore {
    async fn upsert_user(
        &self,
        id: &str,
        email: Option<&str>,
        name: Option<&str>,
    ) -> anyhow::Result<()> {
        let mut data = self.data.write().await;
        let row = data.entry(id.to_string()).or_default();
        if row.created_at.is_none() {
            row.created_at = Some(Utc::now());
        }
        if let Some(e) = email {
            row.email = Some(e.to_string());
        }
        if let Some(n) = name {
            row.name = Some(n.to_string());
        }
        drop(data);
        self.flush().await;
        Ok(())
    }

    async fn get_user(&self, id: &str) -> anyhow::Result<Option<User>> {
        let data = self.data.read().await;
        Ok(data.get(id).map(|row| Self::row_to_user(id, row)))
    }

    async fn get_topics(&self, user_id: &str) -> anyhow::Result<Vec<String>> {
        let data = self.data.read().await;
        Ok(data
            .get(user_id)
            .map(|r| r.topics.clone())
            .unwrap_or_default())
    }

    async fn set_topics(&self, user_id: &str, topics: &[String]) -> anyhow::Result<()> {
        let mut data = self.data.write().await;
        let row = data.entry(user_id.to_string()).or_default();
        if row.created_at.is_none() {
            row.created_at = Some(Utc::now());
        }
        row.topics = topics.to_vec();
        drop(data);
        self.flush().await;
        Ok(())
    }

    async fn get_tags(&self, user_id: &str) -> anyhow::Result<Vec<String>> {
        let data = self.data.read().await;
        Ok(data
            .get(user_id)
            .map(|r| r.tags.clone())
            .unwrap_or_default())
    }

    async fn set_tags(&self, user_id: &str, tags: &[String]) -> anyhow::Result<()> {
        let mut data = self.data.write().await;
        let row = data.entry(user_id.to_string()).or_default();
        if row.created_at.is_none() {
            row.created_at = Some(Utc::now());
        }
        row.tags = tags.to_vec();
        drop(data);
        self.flush().await;
        Ok(())
    }

    async fn get_preference(&self, user_id: &str, key: &str) -> anyhow::Result<Option<String>> {
        let data = self.data.read().await;
        Ok(data
            .get(user_id)
            .and_then(|r| r.preferences.get(key).cloned()))
    }

    async fn set_preference(&self, user_id: &str, key: &str, value: &str) -> anyhow::Result<()> {
        let mut data = self.data.write().await;
        let row = data.entry(user_id.to_string()).or_default();
        if row.created_at.is_none() {
            row.created_at = Some(Utc::now());
        }
        row.preferences.insert(key.to_string(), value.to_string());
        drop(data);
        self.flush().await;
        Ok(())
    }

    async fn get_all_preferences(&self, user_id: &str) -> anyhow::Result<HashMap<String, String>> {
        let data = self.data.read().await;
        Ok(data
            .get(user_id)
            .map(|r| r.preferences.clone())
            .unwrap_or_default())
    }

    async fn mark_read(&self, user_id: &str, item_ids: &[String]) -> anyhow::Result<()> {
        let mut data = self.data.write().await;
        let row = data.entry(user_id.to_string()).or_default();
        if row.created_at.is_none() {
            row.created_at = Some(Utc::now());
        }
        for id in item_ids {
            row.read_items.insert(id.clone());
        }
        drop(data);
        self.flush().await;
        Ok(())
    }

    async fn is_read(&self, user_id: &str, item_id: &str) -> anyhow::Result<bool> {
        let data = self.data.read().await;
        Ok(data
            .get(user_id)
            .is_some_and(|r| r.read_items.contains(item_id)))
    }

    async fn unread_count(
        &self,
        user_id: &str,
        candidate_item_ids: &[String],
    ) -> anyhow::Result<usize> {
        let data = self.data.read().await;
        let Some(row) = data.get(user_id) else {
            return Ok(candidate_item_ids.len());
        };
        Ok(candidate_item_ids
            .iter()
            .filter(|id| !row.read_items.contains(id.as_str()))
            .count())
    }

    async fn get_profile(&self, user_id: &str) -> anyhow::Result<Option<UserProfile>> {
        let data = self.data.read().await;
        let Some(row) = data.get(user_id) else {
            return Ok(None);
        };
        Ok(Some(UserProfile {
            user: Self::row_to_user(user_id, row),
            topics: row.topics.clone(),
            tags: row.tags.clone(),
            preferences: row.preferences.clone(),
        }))
    }
}

// ── Type-erased wrapper ────────────────────────────────────────────────────

/// Erased trait with boxed futures. Same pattern as `BoxedFavoriteStore`.
#[allow(clippy::type_complexity)]
trait ErasedUserStore: Send + Sync {
    fn upsert_user<'a>(
        &'a self,
        id: &'a str,
        email: Option<&'a str>,
        name: Option<&'a str>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;

    fn get_user<'a>(
        &'a self,
        id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<Option<User>>> + Send + 'a>,
    >;

    fn get_topics<'a>(
        &'a self,
        user_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<String>>> + Send + 'a>>;

    fn set_topics<'a>(
        &'a self,
        user_id: &'a str,
        topics: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;

    fn get_tags<'a>(
        &'a self,
        user_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<String>>> + Send + 'a>>;

    fn set_tags<'a>(
        &'a self,
        user_id: &'a str,
        tags: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;

    fn get_preference<'a>(
        &'a self,
        user_id: &'a str,
        key: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<Option<String>>> + Send + 'a>,
    >;

    fn set_preference<'a>(
        &'a self,
        user_id: &'a str,
        key: &'a str,
        value: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;

    fn get_all_preferences<'a>(
        &'a self,
        user_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<HashMap<String, String>>> + Send + 'a>,
    >;

    fn mark_read<'a>(
        &'a self,
        user_id: &'a str,
        item_ids: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;

    fn is_read<'a>(
        &'a self,
        user_id: &'a str,
        item_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<bool>> + Send + 'a>>;

    fn unread_count<'a>(
        &'a self,
        user_id: &'a str,
        candidate_item_ids: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<usize>> + Send + 'a>>;

    fn get_profile<'a>(
        &'a self,
        user_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<Option<UserProfile>>> + Send + 'a>,
    >;
}

#[allow(clippy::type_complexity)]
impl<T: UserStore> ErasedUserStore for T {
    fn upsert_user<'a>(
        &'a self,
        id: &'a str,
        email: Option<&'a str>,
        name: Option<&'a str>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(UserStore::upsert_user(self, id, email, name))
    }

    fn get_user<'a>(
        &'a self,
        id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<Option<User>>> + Send + 'a>,
    > {
        Box::pin(UserStore::get_user(self, id))
    }

    fn get_topics<'a>(
        &'a self,
        user_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<String>>> + Send + 'a>>
    {
        Box::pin(UserStore::get_topics(self, user_id))
    }

    fn set_topics<'a>(
        &'a self,
        user_id: &'a str,
        topics: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(UserStore::set_topics(self, user_id, topics))
    }

    fn get_tags<'a>(
        &'a self,
        user_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<String>>> + Send + 'a>>
    {
        Box::pin(UserStore::get_tags(self, user_id))
    }

    fn set_tags<'a>(
        &'a self,
        user_id: &'a str,
        tags: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(UserStore::set_tags(self, user_id, tags))
    }

    fn get_preference<'a>(
        &'a self,
        user_id: &'a str,
        key: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<Option<String>>> + Send + 'a>,
    > {
        Box::pin(UserStore::get_preference(self, user_id, key))
    }

    fn set_preference<'a>(
        &'a self,
        user_id: &'a str,
        key: &'a str,
        value: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(UserStore::set_preference(self, user_id, key, value))
    }

    fn get_all_preferences<'a>(
        &'a self,
        user_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<HashMap<String, String>>> + Send + 'a>,
    > {
        Box::pin(UserStore::get_all_preferences(self, user_id))
    }

    fn mark_read<'a>(
        &'a self,
        user_id: &'a str,
        item_ids: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(UserStore::mark_read(self, user_id, item_ids))
    }

    fn is_read<'a>(
        &'a self,
        user_id: &'a str,
        item_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<bool>> + Send + 'a>>
    {
        Box::pin(UserStore::is_read(self, user_id, item_id))
    }

    fn unread_count<'a>(
        &'a self,
        user_id: &'a str,
        candidate_item_ids: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<usize>> + Send + 'a>>
    {
        Box::pin(UserStore::unread_count(self, user_id, candidate_item_ids))
    }

    fn get_profile<'a>(
        &'a self,
        user_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<Option<UserProfile>>> + Send + 'a>,
    > {
        Box::pin(UserStore::get_profile(self, user_id))
    }
}

/// Type-erased user store that can be shared across async tasks.
pub struct BoxedUserStore(Box<dyn ErasedUserStore>);

impl BoxedUserStore {
    pub fn new<T: UserStore + 'static>(store: T) -> Self {
        Self(Box::new(store))
    }

    pub async fn upsert_user(
        &self,
        id: &str,
        email: Option<&str>,
        name: Option<&str>,
    ) -> anyhow::Result<()> {
        self.0.upsert_user(id, email, name).await
    }

    pub async fn get_user(&self, id: &str) -> anyhow::Result<Option<User>> {
        self.0.get_user(id).await
    }

    pub async fn get_topics(&self, user_id: &str) -> anyhow::Result<Vec<String>> {
        self.0.get_topics(user_id).await
    }

    pub async fn set_topics(&self, user_id: &str, topics: &[String]) -> anyhow::Result<()> {
        self.0.set_topics(user_id, topics).await
    }

    pub async fn get_tags(&self, user_id: &str) -> anyhow::Result<Vec<String>> {
        self.0.get_tags(user_id).await
    }

    pub async fn set_tags(&self, user_id: &str, tags: &[String]) -> anyhow::Result<()> {
        self.0.set_tags(user_id, tags).await
    }

    pub async fn get_preference(&self, user_id: &str, key: &str) -> anyhow::Result<Option<String>> {
        self.0.get_preference(user_id, key).await
    }

    pub async fn set_preference(
        &self,
        user_id: &str,
        key: &str,
        value: &str,
    ) -> anyhow::Result<()> {
        self.0.set_preference(user_id, key, value).await
    }

    pub async fn get_all_preferences(
        &self,
        user_id: &str,
    ) -> anyhow::Result<HashMap<String, String>> {
        self.0.get_all_preferences(user_id).await
    }

    pub async fn mark_read(&self, user_id: &str, item_ids: &[String]) -> anyhow::Result<()> {
        self.0.mark_read(user_id, item_ids).await
    }

    pub async fn is_read(&self, user_id: &str, item_id: &str) -> anyhow::Result<bool> {
        self.0.is_read(user_id, item_id).await
    }

    pub async fn unread_count(
        &self,
        user_id: &str,
        candidate_item_ids: &[String],
    ) -> anyhow::Result<usize> {
        self.0.unread_count(user_id, candidate_item_ids).await
    }

    pub async fn get_profile(&self, user_id: &str) -> anyhow::Result<Option<UserProfile>> {
        self.0.get_profile(user_id).await
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Disambiguation aliases (same pattern as favorites tests).
    async fn upsert(s: &InMemoryUserStore, id: &str, email: Option<&str>, name: Option<&str>) {
        UserStore::upsert_user(s, id, email, name).await.unwrap();
    }
    async fn set_topics(s: &InMemoryUserStore, id: &str, topics: &[String]) {
        UserStore::set_topics(s, id, topics).await.unwrap();
    }
    async fn get_topics(s: &InMemoryUserStore, id: &str) -> Vec<String> {
        UserStore::get_topics(s, id).await.unwrap()
    }
    async fn set_pref(s: &InMemoryUserStore, id: &str, k: &str, v: &str) {
        UserStore::set_preference(s, id, k, v).await.unwrap();
    }
    async fn get_pref(s: &InMemoryUserStore, id: &str, k: &str) -> Option<String> {
        UserStore::get_preference(s, id, k).await.unwrap()
    }
    async fn mark_read(s: &InMemoryUserStore, id: &str, items: &[String]) {
        UserStore::mark_read(s, id, items).await.unwrap();
    }
    async fn is_read(s: &InMemoryUserStore, id: &str, item: &str) -> bool {
        UserStore::is_read(s, id, item).await.unwrap()
    }
    async fn unread(s: &InMemoryUserStore, id: &str, items: &[String]) -> usize {
        UserStore::unread_count(s, id, items).await.unwrap()
    }

    #[tokio::test]
    async fn upsert_and_get_user() {
        let store = InMemoryUserStore::new(None);
        upsert(&store, "alice", Some("a@x.com"), Some("Alice")).await;

        let user = UserStore::get_user(&store, "alice").await.unwrap().unwrap();
        assert_eq!(user.id, "alice");
        assert_eq!(user.email.as_deref(), Some("a@x.com"));
        assert_eq!(user.name.as_deref(), Some("Alice"));
    }

    #[tokio::test]
    async fn set_and_get_topics() {
        let store = InMemoryUserStore::new(None);
        let topics = vec!["rust".to_string(), "ai".to_string()];
        set_topics(&store, "alice", &topics).await;

        assert_eq!(get_topics(&store, "alice").await, topics);
    }

    #[tokio::test]
    async fn preferences_key_value() {
        let store = InMemoryUserStore::new(None);
        set_pref(&store, "alice", "theme", "dark").await;
        set_pref(&store, "alice", "lang", "en").await;

        assert_eq!(
            get_pref(&store, "alice", "theme").await.as_deref(),
            Some("dark")
        );
        let all = UserStore::get_all_preferences(&store, "alice")
            .await
            .unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn read_tracking() {
        let store = InMemoryUserStore::new(None);
        let items = vec!["item-1".to_string(), "item-2".to_string()];
        mark_read(&store, "alice", &items).await;

        assert!(is_read(&store, "alice", "item-1").await);
        assert!(!is_read(&store, "alice", "item-3").await);

        let candidates = vec![
            "item-1".to_string(),
            "item-2".to_string(),
            "item-3".to_string(),
        ];
        assert_eq!(unread(&store, "alice", &candidates).await, 1);
    }

    #[tokio::test]
    async fn get_profile_returns_full_state() {
        let store = InMemoryUserStore::new(None);
        upsert(&store, "alice", Some("a@x.com"), None).await;
        set_topics(&store, "alice", &["rust".to_string()]).await;
        set_pref(&store, "alice", "theme", "dark").await;

        let profile = UserStore::get_profile(&store, "alice")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(profile.user.id, "alice");
        assert_eq!(profile.topics, vec!["rust"]);
        assert_eq!(profile.preferences.get("theme").unwrap(), "dark");
    }

    #[tokio::test]
    async fn unknown_user_returns_none() {
        let store = InMemoryUserStore::new(None);
        assert!(
            UserStore::get_user(&store, "ghost")
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            UserStore::get_profile(&store, "ghost")
                .await
                .unwrap()
                .is_none()
        );
        assert!(get_topics(&store, "ghost").await.is_empty());
    }

    #[tokio::test]
    async fn separate_users() {
        let store = InMemoryUserStore::new(None);
        set_topics(&store, "alice", &["rust".to_string()]).await;
        set_topics(&store, "bob", &["ai".to_string()]).await;

        assert_eq!(get_topics(&store, "alice").await, vec!["rust"]);
        assert_eq!(get_topics(&store, "bob").await, vec!["ai"]);
    }
}
