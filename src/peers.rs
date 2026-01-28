use clap::ValueEnum;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::io;
use std::net::{IpAddr, Ipv6Addr};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

use crate::{
    AddPeerArgs, RemovePeerArgs,
    client::standard::{ClientRoute, action},
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum PeerType {
    Public,
    Lan,
    Vpn,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerExport {
    pub peer_id: String,
    #[serde(default)]
    pub name: String,
    pub ty: PeerType,
    pub addr: String, // host:port
}

impl PeerExport {
    /// This is very janky,
    /// I should write a handshake for this,
    /// but im kind of against the clock here lol
    pub async fn connect(&self, timeout_ms: u64) -> anyhow::Result<TcpStream> {
        let dur = Duration::from_millis(timeout_ms.max(1));
        let mut stream = timeout(dur, TcpStream::connect(&self.addr))
            .await
            .map_err(|_| anyhow::anyhow!("connect timeout: {}", self.addr))?
            .map_err(|e| anyhow::anyhow!("connect failed: {}: {e}", self.addr))?;

        timeout(dur, action(&mut stream, ClientRoute::IsAlive))
            .await
            .map_err(|_| anyhow::anyhow!("handshake timeout: {}", self.addr))?
            .map_err(|e| anyhow::anyhow!("handshake failed: {}: {e}", self.addr))?;

        // WHAT IM DOING HERE IS VERY JANK AND I SHOULD FIX IT
        // when i do an is_alive, i wrote it in the server handler as if it was a normal operation
        // this means, with the current flow of my server, that the server drops the TcpStream
        // after the is alive, meaning this current stream is dead.
        // instead of fixing my server logic right now, im going to save it for later and just open another stream!

        stream.shutdown().await?;

        Ok(TcpStream::connect(&self.addr).await?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerList {
    pub peers: Vec<PeerExport>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestView {
    Public,
    Lan,
    Vpn,
}

pub struct PeerStore {
    conn: Connection,
}

impl PeerStore {
    pub fn open(path: impl AsRef<std::path::Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let s = Self { conn };
        s.init()?;
        Ok(s)
    }

    fn init(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS peers (
                peer_id TEXT PRIMARY KEY,
                name    TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS peer_addrs (
                peer_id TEXT NOT NULL,
                ty      TEXT NOT NULL,      -- 'public'|'lan'|'vpn'
                addr    TEXT NOT NULL,      -- 'host:port'
                FOREIGN KEY(peer_id) REFERENCES peers(peer_id) ON DELETE CASCADE,
                UNIQUE(peer_id, ty)
            );

            CREATE INDEX IF NOT EXISTS idx_peer_addrs_peer ON peer_addrs(peer_id);
            "#,
        )?;
        Ok(())
    }

    pub fn upsert_peer(&self, peer_id: &str, name: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO peers(peer_id, name) VALUES(?1, ?2)
            ON CONFLICT(peer_id) DO UPDATE SET
                name = CASE WHEN excluded.name <> '' THEN excluded.name ELSE peers.name END
            "#,
            params![peer_id, name],
        )?;
        Ok(())
    }

    pub fn set_addr(&self, peer_id: &str, ty: PeerType, addr: &str) -> rusqlite::Result<()> {
        validate_hostport(addr)?;
        self.conn.execute(
            r#"
            INSERT INTO peers(peer_id, name) VALUES(?1, '')
            ON CONFLICT(peer_id) DO NOTHING
            "#,
            params![peer_id],
        )?;

        self.conn.execute(
            r#"
            INSERT INTO peer_addrs(peer_id, ty, addr) VALUES(?1, ?2, ?3)
            ON CONFLICT(peer_id, ty) DO UPDATE SET addr = excluded.addr
            "#,
            params![peer_id, ty_to_str(ty), addr],
        )?;
        Ok(())
    }

    pub fn remove_peer(&self, peer_id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            r#"DELETE FROM peers WHERE peer_id = ?1"#,
            rusqlite::params![peer_id],
        )?;
        Ok(())
    }
    pub fn remove_peers_by_name(&self, name: &str) -> rusqlite::Result<usize> {
        let n = self.conn.execute(
            r#"DELETE FROM peers WHERE name = ?1"#,
            rusqlite::params![name],
        )?;
        Ok(n)
    }
    pub fn clear_addr(&self, peer_id: &str, ty: PeerType) -> rusqlite::Result<()> {
        self.conn.execute(
            r#"DELETE FROM peer_addrs WHERE peer_id = ?1 AND ty = ?2"#,
            params![peer_id, ty_to_str(ty)],
        )?;
        Ok(())
    }

    pub fn export_peer_list_json(
        &self,
        limit: usize,
        view: RequestView,
    ) -> rusqlite::Result<String> {
        let mut stmt = self
            .conn
            .prepare(r#"SELECT peer_id, name FROM peers LIMIT ?1"#)?;

        let rows = stmt.query_map(params![limit as i64], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;

        let mut out = Vec::new();
        for row in rows {
            let (peer_id, name) = row?;
            if let Some((ty, addr)) = self.pick_one_addr(&peer_id, view)? {
                out.push(PeerExport {
                    peer_id,
                    name,
                    ty,
                    addr,
                });
            }
        }

        Ok(serde_json::to_string(&PeerList { peers: out }).unwrap())
    }

    pub fn peer_list(&self, limit: usize, view: RequestView) -> rusqlite::Result<PeerList> {
        let mut stmt = self
            .conn
            .prepare(r#"SELECT peer_id, name FROM peers LIMIT ?1"#)?;

        let rows = stmt.query_map(params![limit as i64], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;

        let mut out = Vec::new();
        for row in rows {
            let (peer_id, name) = row?;
            if let Some((ty, addr)) = self.pick_one_addr(&peer_id, view)? {
                out.push(PeerExport {
                    peer_id,
                    name,
                    ty,
                    addr,
                });
            }
        }

        Ok(PeerList { peers: out })
    }

    pub fn merge_peer_list_json(&mut self, json: &str) -> rusqlite::Result<()> {
        let list: PeerList = serde_json::from_str(json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?;

        let tx = self.conn.transaction()?;
        for p in list.peers {
            tx.execute(
                r#"
                INSERT INTO peers(peer_id, name) VALUES(?1, ?2)
                ON CONFLICT(peer_id) DO UPDATE SET
                    name = CASE WHEN excluded.name <> '' THEN excluded.name ELSE peers.name END
                "#,
                params![p.peer_id, p.name],
            )?;

            validate_hostport(&p.addr)?;
            tx.execute(
                r#"
                INSERT INTO peer_addrs(peer_id, ty, addr) VALUES(?1, ?2, ?3)
                ON CONFLICT(peer_id, ty) DO UPDATE SET addr = excluded.addr
                "#,
                params![p.peer_id, ty_to_str(p.ty), p.addr],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    fn pick_one_addr(
        &self,
        peer_id: &str,
        view: RequestView,
    ) -> rusqlite::Result<Option<(PeerType, String)>> {
        // Preference order depends on requester view:
        let order: &[PeerType] = match view {
            RequestView::Public => &[PeerType::Public, PeerType::Vpn],
            RequestView::Lan => &[PeerType::Lan, PeerType::Public, PeerType::Vpn],
            RequestView::Vpn => &[PeerType::Vpn, PeerType::Lan, PeerType::Public],
        };

        for ty in order {
            let addr: Option<String> = self
                .conn
                .query_row(
                    r#"SELECT addr FROM peer_addrs WHERE peer_id = ?1 AND ty = ?2"#,
                    params![peer_id, ty_to_str(*ty)],
                    |r| r.get(0),
                )
                .optional()?;

            if let Some(a) = addr {
                if !a.is_empty() {
                    return Ok(Some((*ty, a)));
                }
            }
        }
        Ok(None)
    }
}

fn ty_to_str(t: PeerType) -> &'static str {
    match t {
        PeerType::Public => "public",
        PeerType::Lan => "lan",
        PeerType::Vpn => "vpn",
    }
}

fn validate_hostport(s: &str) -> rusqlite::Result<()> {
    let (host, port) = s
        .rsplit_once(':')
        .ok_or_else(|| rusqlite::Error::InvalidParameterName("expected host:port".into()))?;
    if host.is_empty() {
        return Err(rusqlite::Error::InvalidParameterName("empty host".into()));
    }
    let port: u32 = port
        .parse()
        .map_err(|_| rusqlite::Error::InvalidParameterName("bad port".into()))?;
    if port == 0 || port > 65535 {
        return Err(rusqlite::Error::InvalidParameterName(
            "port out of range".into(),
        ));
    }
    Ok(())
}

pub fn infer_view_from_remote(ip: IpAddr) -> RequestView {
    if is_privateish(ip) {
        RequestView::Lan
    } else {
        RequestView::Public
    }
}

// Big thanks to reddit for this one
fn is_privateish(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            v4.is_loopback()
                || v4.is_link_local()
                || o[0] == 10
                || (o[0] == 172 && (16..=31).contains(&o[1]))
                || (o[0] == 192 && o[1] == 168)
        }
        IpAddr::V6(v6) => v6.is_loopback() || is_link_local_v6(v6) || is_unique_local(v6),
    }
}

fn is_unique_local(v6: Ipv6Addr) -> bool {
    (v6.segments()[0] & 0xfe00) == 0xfc00
}

fn is_link_local_v6(v6: Ipv6Addr) -> bool {
    (v6.segments()[0] & 0xffc0) == 0xfe80
}

pub fn peer_id_from_addr(addr: &str) -> String {
    // normalize: lowercase, trim, no whitespace
    let norm = addr.trim().to_lowercase();

    let hash = blake3::hash(norm.as_bytes());
    format!("addr:{}", hash.to_hex())
}

pub fn add_peer(args: AddPeerArgs) -> io::Result<()> {
    let store = PeerStore::open(args.peer_db).map_err(io::Error::other)?;

    let peer_id = peer_id_from_addr(&args.host);

    store
        .upsert_peer(&peer_id, &args.name)
        .map_err(io::Error::other)?;

    store
        .set_addr(&peer_id, args.ty, &args.host)
        .map_err(io::Error::other)?;

    Ok(())
}

pub fn remove_peer(args: RemovePeerArgs) -> io::Result<()> {
    let store = PeerStore::open(args.peer_db).map_err(io::Error::other)?;

    store
        .remove_peers_by_name(&args.name)
        .map_err(io::Error::other)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::isalive::is_alive;
    use crate::server::standard::route;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn peer_connectivity() -> anyhow::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        let server = tokio::task::spawn(async move {
            let (mut stream, _) = listener.accept().await?;
            route(&mut stream).await?;
            is_alive(&mut stream).await?;

            Ok::<(), anyhow::Error>(())
        });

        let peer = PeerExport {
            peer_id: "".to_owned(),
            name: "".to_owned(),
            ty: PeerType::Lan,
            addr: addr.to_string(),
        };

        tokio::time::sleep(Duration::from_millis(2000)).await;

        assert!(peer.connect(1000).await.is_ok());
        server.await??;

        Ok(())
    }

    #[tokio::test]
    async fn failed_connection() {
        let peer = PeerExport {
            peer_id: "".to_owned(),
            name: "".to_owned(),
            ty: PeerType::Lan,
            addr: "127.0.0.1:0".to_owned(),
        };

        assert!(peer.connect(1000).await.is_err());
    }
}
