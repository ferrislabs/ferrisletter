//! Favorites storage — save, list, and remove favorite articles.
//!
//! The [`FavoriteStore`] trait defines the interface. Two implementations:
//!
//! - [`InMemoryFavoriteStore`] — for stdio/local mode, with optional JSON file
//!   persistence at `~/.config/ferrisletter/favorites.json`.
//! - External crates (e.g. Lattice) can provide a database-backed implementation.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// A single saved favorite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteEntry {
    pub item_id: String,
    pub saved_at: DateTime<Utc>,
}

/// Async trait for favorite storage backends.
pub trait FavoriteStore: Send + Sync {
    fn add_favorite(
        &self,
        user_id: &str,
        item_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

    fn remove_favorite(
        &self,
        user_id: &str,
        item_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

    /// Returns item IDs, newest-saved first.
    fn list_favorites(
        &self,
        user_id: &str,
        limit: Option<usize>,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<FavoriteEntry>>> + Send;

    fn is_favorite(
        &self,
        user_id: &str,
        item_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<bool>> + Send;

    fn count_favorites(
        &self,
        user_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<usize>> + Send;
}

// ── In-memory implementation ───────────────────────────────────────────────

/// In-memory favorite store with optional JSON file persistence.
///
/// When `file_path` is `Some`, changes are flushed to disk after every
/// mutation so that favorites survive server restarts.
pub struct InMemoryFavoriteStore {
    /// user_id → list of favorite entries (newest first).
    data: Arc<RwLock<HashMap<String, Vec<FavoriteEntry>>>>,
    file_path: Option<PathBuf>,
}

impl InMemoryFavoriteStore {
    /// Create a new store. If `file_path` is provided and the file exists,
    /// previously saved favorites are loaded from it.
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

    /// Default path: `~/.config/ferrisletter/favorites.json`.
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("ferrisletter").join("favorites.json"))
    }

    fn load_from_file(path: &std::path::Path) -> Option<HashMap<String, Vec<FavoriteEntry>>> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    async fn flush(&self) {
        let Some(ref path) = self.file_path else {
            return;
        };
        let snapshot = self.data.read().await.clone();
        let path = path.clone();
        // Fire-and-forget: write in a blocking task so we don't block the
        // async runtime, and don't fail the caller if disk write fails.
        tokio::task::spawn_blocking(move || {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match serde_json::to_string_pretty(&snapshot) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&path, json) {
                        tracing::warn!("failed to write favorites file: {e}");
                    }
                }
                Err(e) => tracing::warn!("failed to serialize favorites: {e}"),
            }
        });
    }
}

impl FavoriteStore for InMemoryFavoriteStore {
    async fn add_favorite(&self, user_id: &str, item_id: &str) -> anyhow::Result<()> {
        let mut data = self.data.write().await;
        let entries = data.entry(user_id.to_string()).or_default();
        // Idempotent: don't add if already favorited.
        if entries.iter().any(|e| e.item_id == item_id) {
            return Ok(());
        }
        entries.insert(
            0,
            FavoriteEntry {
                item_id: item_id.to_string(),
                saved_at: Utc::now(),
            },
        );
        drop(data);
        self.flush().await;
        Ok(())
    }

    async fn remove_favorite(&self, user_id: &str, item_id: &str) -> anyhow::Result<()> {
        let mut data = self.data.write().await;
        if let Some(entries) = data.get_mut(user_id) {
            entries.retain(|e| e.item_id != item_id);
        }
        drop(data);
        self.flush().await;
        Ok(())
    }

    async fn list_favorites(
        &self,
        user_id: &str,
        limit: Option<usize>,
    ) -> anyhow::Result<Vec<FavoriteEntry>> {
        let data = self.data.read().await;
        let entries = data.get(user_id).cloned().unwrap_or_default();
        match limit {
            Some(n) => Ok(entries.into_iter().take(n).collect()),
            None => Ok(entries),
        }
    }

    async fn is_favorite(&self, user_id: &str, item_id: &str) -> anyhow::Result<bool> {
        let data = self.data.read().await;
        Ok(data
            .get(user_id)
            .is_some_and(|entries| entries.iter().any(|e| e.item_id == item_id)))
    }

    async fn count_favorites(&self, user_id: &str) -> anyhow::Result<usize> {
        let data = self.data.read().await;
        Ok(data.get(user_id).map_or(0, |e| e.len()))
    }
}

// ── Type-erased wrapper ────────────────────────────────────────────────────

/// Type-erased favorite store for use behind `Arc<dyn …>`.
///
/// Uses the same pattern as `BoxedConnector` — an internal `ErasedFavoriteStore`
/// trait with boxed futures to erase the RPIT lifetime from `FavoriteStore`.
///
/// Every method takes `&self` with a unified `'a` lifetime that covers all
/// borrowed parameters so the returned boxed future can reference them.
trait ErasedFavoriteStore: Send + Sync {
    fn add_favorite<'a>(
        &'a self,
        user_id: &'a str,
        item_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;

    fn remove_favorite<'a>(
        &'a self,
        user_id: &'a str,
        item_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;

    fn list_favorites<'a>(
        &'a self,
        user_id: &'a str,
        limit: Option<usize>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<Vec<FavoriteEntry>>> + Send + 'a>,
    >;

    fn is_favorite<'a>(
        &'a self,
        user_id: &'a str,
        item_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<bool>> + Send + 'a>>;

    fn count_favorites<'a>(
        &'a self,
        user_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<usize>> + Send + 'a>>;
}

impl<T: FavoriteStore> ErasedFavoriteStore for T {
    fn add_favorite<'a>(
        &'a self,
        user_id: &'a str,
        item_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(FavoriteStore::add_favorite(self, user_id, item_id))
    }

    fn remove_favorite<'a>(
        &'a self,
        user_id: &'a str,
        item_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(FavoriteStore::remove_favorite(self, user_id, item_id))
    }

    fn list_favorites<'a>(
        &'a self,
        user_id: &'a str,
        limit: Option<usize>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<Vec<FavoriteEntry>>> + Send + 'a>,
    > {
        Box::pin(FavoriteStore::list_favorites(self, user_id, limit))
    }

    fn is_favorite<'a>(
        &'a self,
        user_id: &'a str,
        item_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<bool>> + Send + 'a>>
    {
        Box::pin(FavoriteStore::is_favorite(self, user_id, item_id))
    }

    fn count_favorites<'a>(
        &'a self,
        user_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<usize>> + Send + 'a>>
    {
        Box::pin(FavoriteStore::count_favorites(self, user_id))
    }
}

/// Type-erased favorite store that can be shared across async tasks.
pub struct BoxedFavoriteStore(Box<dyn ErasedFavoriteStore>);

impl BoxedFavoriteStore {
    pub fn new<T: FavoriteStore + 'static>(store: T) -> Self {
        Self(Box::new(store))
    }

    pub async fn add_favorite(&self, user_id: &str, item_id: &str) -> anyhow::Result<()> {
        self.0.add_favorite(user_id, item_id).await
    }

    pub async fn remove_favorite(&self, user_id: &str, item_id: &str) -> anyhow::Result<()> {
        self.0.remove_favorite(user_id, item_id).await
    }

    pub async fn list_favorites(
        &self,
        user_id: &str,
        limit: Option<usize>,
    ) -> anyhow::Result<Vec<FavoriteEntry>> {
        self.0.list_favorites(user_id, limit).await
    }

    pub async fn is_favorite(&self, user_id: &str, item_id: &str) -> anyhow::Result<bool> {
        self.0.is_favorite(user_id, item_id).await
    }

    pub async fn count_favorites(&self, user_id: &str) -> anyhow::Result<usize> {
        self.0.count_favorites(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Alias to disambiguate from the ErasedFavoriteStore blanket impl.
    async fn add(s: &InMemoryFavoriteStore, u: &str, id: &str) {
        FavoriteStore::add_favorite(s, u, id).await.unwrap();
    }
    async fn remove(s: &InMemoryFavoriteStore, u: &str, id: &str) {
        FavoriteStore::remove_favorite(s, u, id).await.unwrap();
    }
    async fn list(s: &InMemoryFavoriteStore, u: &str, limit: Option<usize>) -> Vec<FavoriteEntry> {
        FavoriteStore::list_favorites(s, u, limit).await.unwrap()
    }
    async fn is_fav(s: &InMemoryFavoriteStore, u: &str, id: &str) -> bool {
        FavoriteStore::is_favorite(s, u, id).await.unwrap()
    }
    async fn count(s: &InMemoryFavoriteStore, u: &str) -> usize {
        FavoriteStore::count_favorites(s, u).await.unwrap()
    }

    #[tokio::test]
    async fn add_and_list_favorites() {
        let store = InMemoryFavoriteStore::new(None);
        add(&store, "anon", "item-1").await;
        add(&store, "anon", "item-2").await;

        let favs = list(&store, "anon", None).await;
        assert_eq!(favs.len(), 2);
        // Newest first.
        assert_eq!(favs[0].item_id, "item-2");
        assert_eq!(favs[1].item_id, "item-1");
    }

    #[tokio::test]
    async fn add_favorite_is_idempotent() {
        let store = InMemoryFavoriteStore::new(None);
        add(&store, "anon", "item-1").await;
        add(&store, "anon", "item-1").await;
        assert_eq!(count(&store, "anon").await, 1);
    }

    #[tokio::test]
    async fn remove_favorite() {
        let store = InMemoryFavoriteStore::new(None);
        add(&store, "anon", "item-1").await;
        add(&store, "anon", "item-2").await;
        remove(&store, "anon", "item-1").await;

        let favs = list(&store, "anon", None).await;
        assert_eq!(favs.len(), 1);
        assert_eq!(favs[0].item_id, "item-2");
    }

    #[tokio::test]
    async fn is_favorite() {
        let store = InMemoryFavoriteStore::new(None);
        add(&store, "anon", "item-1").await;

        assert!(is_fav(&store, "anon", "item-1").await);
        assert!(!is_fav(&store, "anon", "item-2").await);
    }

    #[tokio::test]
    async fn list_favorites_with_limit() {
        let store = InMemoryFavoriteStore::new(None);
        for i in 0..5 {
            add(&store, "anon", &format!("item-{i}")).await;
        }
        let favs = list(&store, "anon", Some(3)).await;
        assert_eq!(favs.len(), 3);
    }

    #[tokio::test]
    async fn separate_users() {
        let store = InMemoryFavoriteStore::new(None);
        add(&store, "alice", "item-1").await;
        add(&store, "bob", "item-2").await;

        assert_eq!(count(&store, "alice").await, 1);
        assert_eq!(count(&store, "bob").await, 1);
        assert!(is_fav(&store, "alice", "item-1").await);
        assert!(!is_fav(&store, "alice", "item-2").await);
    }
}
