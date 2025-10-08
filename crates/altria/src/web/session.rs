//! Session management module for web applications
//!
//! This module provides a flexible and efficient session management system with:
//! - Generic session data support with a convenient default implementation
//! - Thread-safe operations using `Arc<RwLock<_>>`
//! - Optional expiration tracking
//! - Change tracking for efficient persistence
//! - Full serialization support via serde
//! - Extensible storage backend via the `SessionStore` trait
//! - Customizable session ID generation via builder pattern
//!
//! # Examples
//!
//! ```
//! use altria::web::session::{SessionBuilder, DefaultSessionData};
//! use std::time::Duration;
//!
//! // Create a session using builder
//! let data = DefaultSessionData {
//!     user_id: 42,
//!     username: "alice".to_string(),
//! };
//! let session = SessionBuilder::new()
//!     .data(data)
//!     .expires_in(Duration::from_secs(3600))
//!     .build();
//!
//! // Access session properties
//! assert!(!session.is_expired());
//! println!("Session ID: {}", session.id());
//!
//! // Set context data
//! session.set_context("last_page", "/dashboard");
//!
//! // Check if modified
//! assert!(session.is_modified());
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Default session data structure with essential user information
///
/// This is a simple data structure with public fields for direct access.
///
/// # Examples
///
/// ```
/// use altria::web::session::DefaultSessionData;
///
/// let data = DefaultSessionData {
///     user_id: 123,
///     username: "alice".to_string(),
/// };
///
/// assert_eq!(data.user_id, 123);
/// assert_eq!(data.username, "alice");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultSessionData {
    /// User ID
    pub user_id: u64,
    /// Username  
    pub username: String,
}

/// Trait for custom session ID generation
///
/// Implement this trait to provide custom session ID generation logic.
/// The default implementation uses UUID v4.
///
/// # Thread Safety
///
/// Implementations must be Send + Sync as they may be used across threads.
///
/// # Examples
///
/// ```
/// use altria::web::session::SessionIdGenerator;
///
/// #[derive(Debug, Clone)]
/// struct CustomIdGenerator {
///     prefix: String,
/// }
///
/// impl SessionIdGenerator for CustomIdGenerator {
///     fn generate(&self) -> String {
///         format!("{}-{}", self.prefix, uuid::Uuid::new_v4())
///     }
/// }
/// ```
pub trait SessionIdGenerator: Send + Sync {
    /// Generate a new session ID
    fn generate(&self) -> String;
}

/// Default UUID v4 session ID generator
///
/// This is a zero-sized type that implements `SessionIdGenerator`
/// using UUID v4 for session ID generation.
#[derive(Debug, Clone, Copy, Default)]
pub struct UuidV4Generator;

impl SessionIdGenerator for UuidV4Generator {
    fn generate(&self) -> String {
        Uuid::new_v4().to_string()
    }
}

/// Internal session state that requires synchronization
///
/// This contains all mutable session data that needs to be protected
/// by a lock for thread-safe access.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionState<T> {
    /// Optional session data
    data: Option<T>,
    /// Context/extra data as key-value pairs
    context: HashMap<String, String>,
    /// Optional expiration time (None means never expires)
    expires_at: Option<SystemTime>,
    /// Whether the session has been modified since last save
    #[serde(skip)]
    modified: bool,
    /// Whether the session is marked for deletion
    #[serde(skip)]
    discarded: bool,
}

/// A thread-safe session with generic data support
///
/// The `Session` type manages user sessions with:
/// - A unique, immutable session ID
/// - An immutable creation timestamp
/// - Optional generic session data
/// - Optional expiration tracking (None means never expires)
/// - Context data for storing additional key-value pairs
/// - Thread-safe operations via interior mutability
/// - Change tracking for efficient persistence
/// - Full serialization support
///
/// # Thread Safety
///
/// All operations are thread-safe. The session uses `Arc<RwLock<_>>` internally
/// for mutable state, allowing multiple readers or a single writer at a time.
/// The `Session` type itself is `Clone + Send + Sync`.
///
/// # Type Parameters
///
/// - `T`: The session data type (must implement `Clone + Serialize + DeserializeOwned`)
///
/// # Examples
///
/// ```
/// use altria::web::session::{SessionBuilder, DefaultSessionData};
/// use std::time::Duration;
///
/// // Create a session with builder
/// let data = DefaultSessionData {
///     user_id: 1,
///     username: "alice".to_string(),
/// };
/// let session = SessionBuilder::new()
///     .data(data)
///     .expires_in(Duration::from_secs(3600))
///     .build();
///
/// // Session ID and creation time are immutable
/// let id = session.id();
/// let created = session.created_at();
///
/// // Mutable operations are thread-safe
/// session.set_context("theme", "dark");
/// assert!(session.is_modified());
/// ```
#[derive(Clone)]
pub struct Session<T = DefaultSessionData>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    /// Unique session identifier (immutable)
    id: String,
    /// Creation timestamp (immutable)
    created_at: SystemTime,
    /// Internal state protected by `RwLock` for thread safety
    state: Arc<RwLock<SessionState<T>>>,
}

impl<T> Session<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    /// Get the session ID
    ///
    /// The session ID is immutable and set at creation time.
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    ///
    /// let session = SessionBuilder::<()>::new().build();
    /// let id = session.id();
    /// assert!(!id.is_empty());
    /// ```
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the creation timestamp
    ///
    /// The creation timestamp is immutable and set at creation time.
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    ///
    /// let session = SessionBuilder::<()>::new().build();
    /// let created = session.created_at();
    /// assert!(created <= std::time::SystemTime::now());
    /// ```
    #[must_use]
    pub const fn created_at(&self) -> SystemTime {
        self.created_at
    }

    /// Get the expiration timestamp
    ///
    /// Returns `None` if the session never expires or if the lock is poisoned.
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    /// use std::time::Duration;
    ///
    /// let session = SessionBuilder::<()>::new()
    ///     .expires_in(Duration::from_secs(3600))
    ///     .build();
    /// assert!(session.expires_at().is_some());
    ///
    /// let session = SessionBuilder::<()>::new().build();
    /// assert!(session.expires_at().is_none());
    /// ```
    #[must_use]
    pub fn expires_at(&self) -> Option<SystemTime> {
        self.state.read().ok().and_then(|state| state.expires_at)
    }

    /// Check if the session has expired
    ///
    /// Returns `false` if the session has no expiration time (permanent session).
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    /// use std::time::Duration;
    ///
    /// let session = SessionBuilder::<()>::new()
    ///     .expires_in(Duration::from_secs(3600))
    ///     .build();
    /// assert!(!session.is_expired());
    ///
    /// // Permanent session never expires
    /// let session = SessionBuilder::<()>::new().build();
    /// assert!(!session.is_expired());
    /// ```
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.state
            .read()
            .ok()
            .and_then(|state| state.expires_at)
            .is_some_and(|expires_at| SystemTime::now() >= expires_at)
    }

    /// Check if the session has been modified since last save
    ///
    /// This flag is automatically set when:
    /// - Session data is updated via `update_data()`
    /// - Context values are set via `set_context()`
    /// - Expiration is extended via `extend_expiration()`
    /// - Session is discarded via `discard()`
    #[must_use]
    pub fn is_modified(&self) -> bool {
        self.state.read().ok().is_some_and(|state| state.modified)
    }

    /// Check if the session has been marked for deletion
    #[must_use]
    pub fn is_discarded(&self) -> bool {
        self.state.read().ok().is_some_and(|state| state.discarded)
    }

    /// Check if the session has data
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::{SessionBuilder, DefaultSessionData};
    ///
    /// let data = DefaultSessionData {
    ///     user_id: 1,
    ///     username: "alice".to_string(),
    /// };
    /// let session = SessionBuilder::new().data(data).build();
    /// assert!(session.has_data());
    ///
    /// let session = SessionBuilder::<DefaultSessionData>::new().build();
    /// assert!(!session.has_data());
    /// ```
    #[must_use]
    pub fn has_data(&self) -> bool {
        self.state
            .read()
            .ok()
            .is_some_and(|state| state.data.is_some())
    }

    /// Get a clone of the session data
    ///
    /// Returns `None` if there's no data or if the lock is poisoned.
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::{SessionBuilder, DefaultSessionData};
    ///
    /// let data = DefaultSessionData {
    ///     user_id: 1,
    ///     username: "alice".to_string(),
    /// };
    /// let session = SessionBuilder::new().data(data).build();
    ///
    /// if let Some(data) = session.data() {
    ///     assert_eq!(data.user_id, 1);
    ///     assert_eq!(data.username, "alice");
    /// }
    /// ```
    #[must_use]
    pub fn data(&self) -> Option<T> {
        self.state.read().ok().and_then(|state| state.data.clone())
    }

    /// Update the session data and mark as modified
    ///
    /// This replaces the entire session data and marks the session as modified.
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::{SessionBuilder, DefaultSessionData};
    ///
    /// let session = SessionBuilder::<DefaultSessionData>::new().build();
    ///
    /// let data = DefaultSessionData {
    ///     user_id: 1,
    ///     username: "alice".to_string(),
    /// };
    /// session.update_data(Some(data));
    /// assert!(session.is_modified());
    /// assert!(session.has_data());
    /// ```
    pub fn update_data(&self, data: Option<T>) {
        if let Ok(mut state) = self.state.write() {
            state.data = data;
            state.modified = true;
        }
    }

    /// Get a context value by key
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    ///
    /// let session = SessionBuilder::<()>::new().build();
    ///
    /// session.set_context("theme", "dark");
    /// assert_eq!(session.get_context("theme"), Some("dark".to_string()));
    /// assert_eq!(session.get_context("nonexistent"), None);
    /// ```
    #[must_use]
    pub fn get_context(&self, key: &str) -> Option<String> {
        self.state
            .read()
            .ok()
            .and_then(|state| state.context.get(key).cloned())
    }

    /// Set a context value by key and mark as modified
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    ///
    /// let session = SessionBuilder::<()>::new().build();
    ///
    /// session.set_context("last_page", "/dashboard");
    /// session.set_context("cart_items", "5");
    ///
    /// assert!(session.is_modified());
    /// ```
    pub fn set_context(&self, key: impl Into<String>, value: impl Into<String>) {
        if let Ok(mut state) = self.state.write() {
            state.context.insert(key.into(), value.into());
            state.modified = true;
        }
    }

    /// Get all context data as a cloned `HashMap`
    #[must_use]
    pub fn context(&self) -> HashMap<String, String> {
        self.state
            .read()
            .ok()
            .map(|state| state.context.clone())
            .unwrap_or_default()
    }

    /// Extend the session expiration time
    ///
    /// If the session has no expiration time, this sets one.
    /// If the session already has an expiration, this extends it.
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    /// use std::time::Duration;
    ///
    /// let session = SessionBuilder::<()>::new()
    ///     .expires_in(Duration::from_secs(1800))
    ///     .build();
    ///
    /// // Extend by another 30 minutes
    /// session.extend_expiration(Duration::from_secs(1800));
    /// assert!(session.is_modified());
    /// ```
    pub fn extend_expiration(&self, additional_time: Duration) {
        if let Ok(mut state) = self.state.write() {
            state.expires_at =
                Some(state.expires_at.unwrap_or_else(SystemTime::now) + additional_time);
            state.modified = true;
        }
    }

    /// Set an absolute expiration time
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    /// use std::time::{Duration, SystemTime};
    ///
    /// let session = SessionBuilder::<()>::new().build();
    ///
    /// let expires = SystemTime::now() + Duration::from_secs(3600);
    /// session.set_expiration(Some(expires));
    /// assert!(session.is_modified());
    /// ```
    pub fn set_expiration(&self, expires_at: Option<SystemTime>) {
        if let Ok(mut state) = self.state.write() {
            state.expires_at = expires_at;
            state.modified = true;
        }
    }

    /// Mark the session as discarded (e.g., after user logout)
    ///
    /// This marks the session for deletion and sets the modified flag,
    /// so the store can persist this change.
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    ///
    /// let session = SessionBuilder::<()>::new().build();
    ///
    /// session.discard();
    /// assert!(session.is_discarded());
    /// assert!(session.is_modified());
    /// ```
    pub fn discard(&self) {
        if let Ok(mut state) = self.state.write() {
            state.discarded = true;
            state.modified = true;
        }
    }

    /// Clear the modified flag
    ///
    /// This is typically called by the session store after successfully
    /// persisting the session.
    pub fn clear_modified(&self) {
        if let Ok(mut state) = self.state.write() {
            state.modified = false;
        }
    }
}

// Implement Debug manually to show relevant fields
impl<T> fmt::Debug for Session<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state.read();
        match state {
            Ok(state) => f
                .debug_struct("Session")
                .field("id", &self.id)
                .field("created_at", &self.created_at)
                .field("data", &state.data)
                .field("context", &state.context)
                .field("expires_at", &state.expires_at)
                .field("modified", &state.modified)
                .field("discarded", &state.discarded)
                .finish(),
            Err(_) => f
                .debug_struct("Session")
                .field("id", &self.id)
                .field("created_at", &self.created_at)
                .field("state", &"<poisoned>")
                .finish(),
        }
    }
}

// Implement PartialEq - only compare session IDs
impl<T> PartialEq for Session<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

// Manual Serialize implementation to handle Arc<RwLock<_>>
impl<T> Serialize for Session<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let state = self
            .state
            .read()
            .map_err(|_| serde::ser::Error::custom("failed to acquire read lock"))?;

        let mut s = serializer.serialize_struct("Session", 5)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("created_at", &self.created_at)?;
        s.serialize_field("data", &state.data)?;
        s.serialize_field("context", &state.context)?;
        s.serialize_field("expires_at", &state.expires_at)?;
        drop(state);
        s.end()
    }
}

// Manual Deserialize implementation to handle Arc<RwLock<_>>
impl<'de, T> Deserialize<'de> for Session<T>
where
    T: Clone + Serialize + for<'d> Deserialize<'d> + Send + Sync,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Id,
            CreatedAt,
            Data,
            Context,
            ExpiresAt,
        }

        struct SessionVisitor<T>(std::marker::PhantomData<T>);

        impl<'de, T> Visitor<'de> for SessionVisitor<T>
        where
            T: Clone + Serialize + for<'d> Deserialize<'d> + Send + Sync,
        {
            type Value = Session<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Session")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Session<T>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut created_at = None;
                let mut data = None;
                let mut context = None;
                let mut expires_at = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Field::CreatedAt => {
                            if created_at.is_some() {
                                return Err(de::Error::duplicate_field("created_at"));
                            }
                            created_at = Some(map.next_value()?);
                        }
                        Field::Data => {
                            if data.is_some() {
                                return Err(de::Error::duplicate_field("data"));
                            }
                            data = Some(map.next_value()?);
                        }
                        Field::Context => {
                            if context.is_some() {
                                return Err(de::Error::duplicate_field("context"));
                            }
                            context = Some(map.next_value()?);
                        }
                        Field::ExpiresAt => {
                            if expires_at.is_some() {
                                return Err(de::Error::duplicate_field("expires_at"));
                            }
                            expires_at = Some(map.next_value()?);
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let created_at =
                    created_at.ok_or_else(|| de::Error::missing_field("created_at"))?;
                let data = data.ok_or_else(|| de::Error::missing_field("data"))?;
                let context = context.ok_or_else(|| de::Error::missing_field("context"))?;
                let expires_at =
                    expires_at.ok_or_else(|| de::Error::missing_field("expires_at"))?;

                Ok(Session {
                    id,
                    created_at,
                    state: Arc::new(RwLock::new(SessionState {
                        data,
                        context,
                        expires_at,
                        modified: false,
                        discarded: false,
                    })),
                })
            }
        }

        const FIELDS: &[&str] = &["id", "created_at", "data", "context", "expires_at"];
        deserializer.deserialize_struct("Session", FIELDS, SessionVisitor(std::marker::PhantomData))
    }
}

/// Builder for creating `Session` instances
///
/// This builder provides a flexible and ergonomic way to create sessions
/// with custom configuration, including custom ID generators.
///
/// # Thread Safety
///
/// The builder itself doesn't need to be Sync as it's consumed on build.
/// However, it's Send to allow building sessions in different threads.
///
/// # Examples
///
/// ```
/// use altria::web::session::{SessionBuilder, DefaultSessionData, SessionIdGenerator};
/// use std::time::Duration;
///
/// // Simple session
/// let session = SessionBuilder::<()>::new().build();
///
/// // Session with data
/// let data = DefaultSessionData {
///     user_id: 42,
///     username: "alice".to_string(),
/// };
/// let session = SessionBuilder::new()
///     .data(data)
///     .expires_in(Duration::from_secs(3600))
///     .build();
///
/// // Session with custom ID generator
/// #[derive(Clone)]
/// struct CustomGen;
/// impl SessionIdGenerator for CustomGen {
///     fn generate(&self) -> String {
///         format!("custom-{}", uuid::Uuid::new_v4())
///     }
/// }
///
/// let session = SessionBuilder::<(), CustomGen>::with_id_generator(CustomGen)
///     .expires_in(Duration::from_secs(7200))
///     .build();
/// ```
pub struct SessionBuilder<T, G = UuidV4Generator>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
    G: SessionIdGenerator,
{
    id_generator: G,
    data: Option<T>,
    expires_in: Option<Duration>,
    context: HashMap<String, String>,
}

impl<T> SessionBuilder<T, UuidV4Generator>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    /// Create a new session builder with default UUID v4 ID generator
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    ///
    /// let session = SessionBuilder::<()>::new().build();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            id_generator: UuidV4Generator,
            data: None,
            expires_in: None,
            context: HashMap::new(),
        }
    }
}

impl<T> Default for SessionBuilder<T, UuidV4Generator>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, G> SessionBuilder<T, G>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
    G: SessionIdGenerator,
{
    /// Create a session builder with a custom ID generator
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::{SessionBuilder, SessionIdGenerator};
    ///
    /// #[derive(Clone)]
    /// struct MyGen;
    /// impl SessionIdGenerator for MyGen {
    ///     fn generate(&self) -> String {
    ///         "my-session-id".to_string()
    ///     }
    /// }
    ///
    /// let session = SessionBuilder::<(), MyGen>::with_id_generator(MyGen).build();
    /// assert_eq!(session.id(), "my-session-id");
    /// ```
    #[must_use]
    pub fn with_id_generator(id_generator: G) -> Self {
        Self {
            id_generator,
            data: None,
            expires_in: None,
            context: HashMap::new(),
        }
    }

    /// Set the session data
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::{SessionBuilder, DefaultSessionData};
    ///
    /// let data = DefaultSessionData {
    ///     user_id: 123,
    ///     username: "alice".to_string(),
    /// };
    /// let session = SessionBuilder::new().data(data).build();
    /// assert!(session.has_data());
    /// ```
    #[must_use]
    pub fn data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }

    /// Set the expiration duration
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    /// use std::time::Duration;
    ///
    /// let session = SessionBuilder::<()>::new()
    ///     .expires_in(Duration::from_secs(3600))
    ///     .build();
    /// assert!(session.expires_at().is_some());
    /// ```
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Duration operations aren't const
    pub fn expires_in(mut self, duration: Duration) -> Self {
        self.expires_in = Some(duration);
        self
    }

    /// Add a context key-value pair
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    ///
    /// let session = SessionBuilder::<()>::new()
    ///     .context("theme", "dark")
    ///     .context("language", "en")
    ///     .build();
    ///
    /// assert_eq!(session.get_context("theme"), Some("dark".to_string()));
    /// ```
    #[must_use]
    pub fn context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Build the session
    ///
    /// This consumes the builder and creates a new `Session` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::web::session::SessionBuilder;
    ///
    /// let session = SessionBuilder::<()>::new().build();
    /// assert!(!session.id().is_empty());
    /// ```
    #[must_use]
    pub fn build(self) -> Session<T> {
        let now = SystemTime::now();
        let expires_at = self.expires_in.map(|duration| now + duration);

        Session {
            id: self.id_generator.generate(),
            created_at: now,
            state: Arc::new(RwLock::new(SessionState {
                data: self.data,
                context: self.context,
                expires_at,
                modified: false,
                discarded: false,
            })),
        }
    }
}

/// Trait for session storage backends
///
/// Implement this trait to provide custom session storage solutions
/// (e.g., in-memory, Redis, database, etc.).
///
/// # Type Parameters
///
/// - `T`: The session data type
///
/// # Examples
///
/// ```
/// use altria::web::session::{SessionStore, Session, DefaultSessionData};
/// use std::collections::HashMap;
/// use std::sync::{Arc, RwLock};
/// use std::fmt;
///
/// #[derive(Debug)]
/// struct StoreError(String);
///
/// impl fmt::Display for StoreError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "{}", self.0)
///     }
/// }
///
/// impl std::error::Error for StoreError {}
///
/// struct MemoryStore {
///     sessions: Arc<RwLock<HashMap<String, Session<DefaultSessionData>>>>,
/// }
///
/// impl SessionStore<DefaultSessionData> for MemoryStore {
///     type Error = StoreError;
///
///     async fn save(&self, session: &Session<DefaultSessionData>) -> Result<(), Self::Error> {
///         let mut store = self.sessions.write()
///             .map_err(|e| StoreError(format!("Lock error: {}", e)))?;
///         store.insert(session.id().to_string(), session.clone());
///         session.clear_modified();
///         Ok(())
///     }
///
///     async fn load(&self, session_id: &str) -> Result<Option<Session<DefaultSessionData>>, Self::Error> {
///         let store = self.sessions.read()
///             .map_err(|e| StoreError(format!("Lock error: {}", e)))?;
///         Ok(store.get(session_id).cloned())
///     }
///
///     async fn delete(&self, session_id: &str) -> Result<(), Self::Error> {
///         let mut store = self.sessions.write()
///             .map_err(|e| StoreError(format!("Lock error: {}", e)))?;
///         store.remove(session_id);
///         Ok(())
///     }
///
///     async fn cleanup_expired(&self) -> Result<usize, Self::Error> {
///         let mut store = self.sessions.write()
///             .map_err(|e| StoreError(format!("Lock error: {}", e)))?;
///         let before = store.len();
///         store.retain(|_, session| !session.is_expired());
///         Ok(before - store.len())
///     }
/// }
/// ```
pub trait SessionStore<T>: Send + Sync
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    /// Error type for storage operations
    type Error: std::error::Error + Send + Sync + 'static;

    /// Save a session to the store
    ///
    /// This should persist the session and typically call `session.clear_modified()`
    /// after successful save.
    async fn save(&self, session: &Session<T>) -> Result<(), Self::Error>;

    /// Load a session by ID
    ///
    /// Returns `None` if the session doesn't exist.
    ///
    /// # Note on Return Type
    ///
    /// This returns `Option` rather than an error for missing sessions because:
    /// - In Rust, `Option` is the idiomatic way to represent "value may not exist"
    /// - Missing sessions are a normal case, not an error condition
    /// - Allows for clean pattern matching: `if let Some(session) = store.load(id).await?`
    /// - Errors should be reserved for actual failures (network issues, corruption, etc.)
    async fn load(&self, session_id: &str) -> Result<Option<Session<T>>, Self::Error>;

    /// Delete a session by ID
    async fn delete(&self, session_id: &str) -> Result<(), Self::Error>;

    /// Clean up expired sessions
    ///
    /// Returns the number of sessions deleted.
    async fn cleanup_expired(&self) -> Result<usize, Self::Error>;
}

// Ensure Session is Send + Sync for thread safety
#[allow(dead_code)]
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}

    const fn check_session<T>()
    where
        T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
    {
        assert_send_sync::<Session<T>>();
    }
};

// Ensure SessionBuilder is Send (doesn't need Sync as it's consumed)
#[allow(dead_code)]
const _: () = {
    const fn assert_send<T: Send>() {}

    const fn check_builder<T, G>()
    where
        T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
        G: SessionIdGenerator,
    {
        assert_send::<SessionBuilder<T, G>>();
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_default_session_data() {
        let data = DefaultSessionData {
            user_id: 123,
            username: "alice".to_string(),
        };

        assert_eq!(data.user_id, 123);
        assert_eq!(data.username, "alice");
    }

    #[test]
    fn test_session_builder_basic() {
        let session = SessionBuilder::<()>::new().build();

        assert!(!session.id().is_empty());
        assert!(!session.is_expired());
        assert!(!session.is_modified());
        assert!(!session.is_discarded());
        assert!(!session.has_data());
    }

    #[test]
    fn test_session_builder_with_data() {
        let data = DefaultSessionData {
            user_id: 1,
            username: "bob".to_string(),
        };
        let session = SessionBuilder::new().data(data.clone()).build();

        assert!(session.has_data());
        let retrieved_data = session.data().unwrap();
        assert_eq!(retrieved_data.user_id, 1);
        assert_eq!(retrieved_data.username, "bob");
    }

    #[test]
    fn test_session_builder_with_expiration() {
        let session = SessionBuilder::<()>::new()
            .expires_in(Duration::from_secs(3600))
            .build();

        assert!(session.expires_at().is_some());
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_builder_with_context() {
        let session = SessionBuilder::<()>::new()
            .context("theme", "dark")
            .context("language", "en")
            .build();

        assert_eq!(session.get_context("theme"), Some("dark".to_string()));
        assert_eq!(session.get_context("language"), Some("en".to_string()));
    }

    #[test]
    fn test_session_builder_chaining() {
        let data = DefaultSessionData {
            user_id: 42,
            username: "alice".to_string(),
        };

        let session = SessionBuilder::new()
            .data(data)
            .expires_in(Duration::from_secs(7200))
            .context("theme", "dark")
            .context("timezone", "UTC")
            .build();

        assert!(session.has_data());
        assert!(session.expires_at().is_some());
        assert_eq!(session.get_context("theme"), Some("dark".to_string()));
        assert_eq!(session.get_context("timezone"), Some("UTC".to_string()));
    }

    #[test]
    fn test_custom_id_generator() {
        #[derive(Clone)]
        struct CustomGen;
        impl SessionIdGenerator for CustomGen {
            fn generate(&self) -> String {
                "custom-id-123".to_string()
            }
        }

        let session = SessionBuilder::<(), CustomGen>::with_id_generator(CustomGen).build();
        assert_eq!(session.id(), "custom-id-123");
    }

    #[test]
    fn test_session_immutable_fields() {
        let session = SessionBuilder::<()>::new().build();

        let id1 = session.id();
        let created1 = session.created_at();

        // Modify session
        session.set_context("key", "value");

        // ID and created_at should not change
        let id2 = session.id();
        let created2 = session.created_at();

        assert_eq!(id1, id2);
        assert_eq!(created1, created2);
    }

    #[test]
    fn test_session_update_data() {
        let session = SessionBuilder::<DefaultSessionData>::new().build();
        assert!(!session.has_data());

        let data = DefaultSessionData {
            user_id: 1,
            username: "alice".to_string(),
        };
        session.update_data(Some(data));

        assert!(session.has_data());
        assert!(session.is_modified());
    }

    #[test]
    fn test_session_context() {
        let session = SessionBuilder::<()>::new().build();

        session.set_context("theme", "dark");
        session.set_context("language", "en");

        assert!(session.is_modified());
        assert_eq!(session.get_context("theme"), Some("dark".to_string()));
        assert_eq!(session.context().len(), 2);
    }

    #[test]
    fn test_session_expiration() {
        let session = SessionBuilder::<()>::new()
            .expires_in(Duration::from_millis(1))
            .build();

        assert!(!session.is_expired());
        thread::sleep(Duration::from_millis(10));
        assert!(session.is_expired());
    }

    #[test]
    fn test_session_no_expiration() {
        let session = SessionBuilder::<()>::new().build();
        assert!(session.expires_at().is_none());
        assert!(!session.is_expired());
    }

    #[test]
    fn test_extend_expiration() {
        let session = SessionBuilder::<()>::new()
            .expires_in(Duration::from_millis(100))
            .build();

        let initial_expires = session.expires_at().unwrap();
        session.extend_expiration(Duration::from_secs(3600));

        let new_expires = session.expires_at().unwrap();
        assert!(new_expires > initial_expires);
        assert!(session.is_modified());
    }

    #[test]
    fn test_set_expiration() {
        let session = SessionBuilder::<()>::new().build();
        assert!(session.expires_at().is_none());

        let expires = SystemTime::now() + Duration::from_secs(3600);
        session.set_expiration(Some(expires));

        assert!(session.expires_at().is_some());
        assert!(session.is_modified());
    }

    #[test]
    fn test_session_discard() {
        let session = SessionBuilder::<()>::new().build();

        assert!(!session.is_discarded());
        session.discard();
        assert!(session.is_discarded());
        assert!(session.is_modified());
    }

    #[test]
    fn test_clear_modified() {
        let session = SessionBuilder::<()>::new().build();

        session.set_context("key", "value");
        assert!(session.is_modified());

        session.clear_modified();
        assert!(!session.is_modified());
    }

    #[test]
    fn test_session_clone() {
        let data = DefaultSessionData {
            user_id: 1,
            username: "alice".to_string(),
        };
        let session = SessionBuilder::new().data(data).build();

        session.set_context("key", "value");

        let cloned = session.clone();

        assert_eq!(session.id(), cloned.id());
        assert_eq!(session.created_at(), cloned.created_at());
        assert_eq!(
            session.data().unwrap().user_id,
            cloned.data().unwrap().user_id
        );
        assert_eq!(session.get_context("key"), cloned.get_context("key"));
    }

    #[test]
    fn test_session_equality() {
        let session1 = SessionBuilder::<()>::new().build();
        let session2 = session1.clone();

        // Same ID = equal
        assert_eq!(session1, session2);

        // Different session = not equal
        let session3 = SessionBuilder::<()>::new().build();
        assert_ne!(session1, session3);
    }

    #[test]
    fn test_thread_safety() {
        let session = SessionBuilder::<()>::new().build();

        let session_clone1 = session.clone();
        let session_clone2 = session.clone();

        let handle1 = thread::spawn(move || {
            for i in 0..100 {
                session_clone1.set_context(format!("key{i}"), format!("value{i}"));
            }
        });

        let handle2 = thread::spawn(move || {
            for i in 100..200 {
                session_clone2.set_context(format!("key{i}"), format!("value{i}"));
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        assert_eq!(session.context().len(), 200);
    }

    #[test]
    fn test_session_debug() {
        let data = DefaultSessionData {
            user_id: 1,
            username: "alice".to_string(),
        };
        let session = SessionBuilder::new().data(data).build();

        let debug_str = format!("{session:?}");
        assert!(debug_str.contains("Session"));
        assert!(debug_str.contains("alice"));
    }

    #[test]
    fn test_serialization() {
        let data = DefaultSessionData {
            user_id: 123,
            username: "alice".to_string(),
        };
        let session = SessionBuilder::new()
            .data(data)
            .expires_in(Duration::from_secs(3600))
            .context("theme", "dark")
            .build();

        // Serialize to JSON
        let json = serde_json::to_string(&session).expect("Failed to serialize");

        // Deserialize from JSON
        let restored: Session<DefaultSessionData> =
            serde_json::from_str(&json).expect("Failed to deserialize");

        // Check that immutable fields are preserved
        assert_eq!(session.id(), restored.id());
        assert_eq!(session.created_at(), restored.created_at());

        // Check that data is preserved
        assert_eq!(
            session.data().unwrap().user_id,
            restored.data().unwrap().user_id
        );
        assert_eq!(session.get_context("theme"), restored.get_context("theme"));
    }
}
