#![forbid(unsafe_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::time::{Duration, SystemTime};
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Peer identity advertised by a viewer/agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerId {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caps: Option<String>,
    pub last_seen: u64,
}

/// SQLite schema version.
const SCHEMA_VERSION: i64 = 1;

/// Heartbeat timeout: entries not seen in this long are considered stale.
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(120);

mod queries {
    use super::*;

    /// Create the peers table if it does not exist.
    pub async fn ensure_pool(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS peers (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            caps        TEXT,
            last_seen   INTEGER NOT NULL
        )
        "#,
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)?;
        Ok(())
    }

    /// Register or update a peer ID.
    pub async fn upsert_peer(pool: &SqlitePool, peer: &PeerId) -> Result<()> {
        sqlx::query(
            r#"INSERT OR REPLACE INTO peers (id, name, caps, last_seen)
        VALUES (?1, ?2, ?3, ?4)"#,
        )
        .bind(peer.id.as_str())
        .bind(peer.name.as_str())
        .bind(&peer.caps)
        .bind(peer.last_seen as i64)
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)?;
        Ok(())
    }

    /// List all known peers.
    pub async fn list_peers(pool: &SqlitePool) -> Result<Vec<PeerId>> {
        let rows = sqlx::query_as!(
            PeerIdDbRow,
            "SELECT id, name, caps, last_seen FROM peers",
        )
        .fetch_all(pool)
        .await
        .map_err(anyhow::Error::from)?;
        Ok(rows
            .into_iter()
            .map(|r| PeerId {
                id: r.id,
                name: r.name,
                caps: r.caps.map(|c| c.to_owned()),
                last_seen: r.last_seen as u64,
            })
            .collect())
    }

    /// Remove stale peers (last_seen older than HEARTBEAT_TIMEOUT).
    pub async fn gc_stale(pool: &SqlitePool) -> Result<usize> {
        // Compute cutoff: last_seen must be newer than now - HEARTBEAT_TIMEOUT
        let now = SystemTime::now();
        let cutoff = now - HEARTBEAT_TIMEOUT;
        let result = sqlx::query!(
            "DELETE FROM peers WHERE last_seen < ?1",
            cutoff
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)?;
        Ok(result.rows_affected() as usize)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct PeerIdDbRow {
    id: String,
    name: String,
    caps: Option<String>,
    last_seen: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    vd_common::init_tracing();

    let db_path = "vd-signal.db";
    let pool = SqlitePool::connect(format!("sqlite:{db_path}").as_str()).await?;
    queries::ensure_pool(&pool).await?;

    // Startup GC
    let gc = queries::gc_stale(&pool).await?;
    if gc > 0 {
        info!(n = gc, "gc: removed stale peers on startup");
    }

    // Register a peer
    let peer = PeerId {
        id: "agent-42".into(),
        name: "Test Agent".into(),
        caps: Some(r#"{"video": true, "cursor": true}"#.into()),
        last_seen: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(anyhow::Error::from)?
            .as_secs(),
    };
    queries::upsert_peer(&pool, &peer).await?;

    // List peers
    let peers = queries::list_peers(&pool).await?;
    info!(n = peers.len(), "peers in DB");

    // Garbage collection
    let gc2 = queries::gc_stale(&pool).await?;
    info!(n = gc2, "gc: removed stale peers");

    Ok(())
}