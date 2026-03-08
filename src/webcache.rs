use reqwest::StatusCode;
use std::convert::Infallible;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CACHE_EXPIRE: Duration = Duration::from_secs(60 * 60 * 24);

enum Cached<T> {
    /// We're caching actual contents.
    Valid(T),

    /// The cache entry is missing or expired.
    Invalid,

    /// The cache entry is present and fresh, but represents an error.
    Error,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error accessing cache")]
    Cache(#[from] sled::Error),
    #[error("failed to load URL")]
    Web(#[from] reqwest::Error),
}

/// Unwrap the sled `Result` for a transaction that cannot abort.
///
/// The only way these transactions can fail is with an underlying storage
/// error, not with an error raised by our own code.
fn unwrap_res<T>(res: sled::transaction::TransactionResult<T, Infallible>) -> sled::Result<T> {
    res.map_err(|err| match err {
        sled::transaction::TransactionError::Storage(e) => e,
        sled::transaction::TransactionError::Abort(_) => panic!("transaction cannot abort"),
    })
}

/// Get the current cached URL contents.
fn cache_get(db: &sled::Db, url: &str) -> sled::Result<Cached<sled::IVec>> {
    let ts_key = format!("ts:{url}");

    unwrap_res(db.transaction(|tx| {
        if let Some(ts_data) = tx.get(ts_key.as_bytes())? {
            let ts = u64::from_le_bytes(ts_data.as_ref().try_into().unwrap());
            let time = UNIX_EPOCH + Duration::from_secs(ts);

            // Is the cache entry expired?
            let age = SystemTime::now().duration_since(time).unwrap();
            if age > CACHE_EXPIRE {
                tx.remove(ts_key.as_bytes())?;
                tx.remove(url)?;
                Ok(Cached::Invalid)
            } else {
                match tx.get(url)? {
                    Some(body) => Ok(Cached::Valid(body)),
                    None => Ok(Cached::Error),
                }
            }
        } else {
            // Cold miss.
            Ok(Cached::Invalid)
        }
    }))
}

/// Set the current cached contents of the URL.
fn cache_set(db: &sled::Db, url: &str, body: Cached<&[u8]>) -> sled::Result<()> {
    let ts_key = format!("ts:{url}");
    let ts_data = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_le_bytes();

    unwrap_res(db.transaction(|tx| {
        tx.insert(ts_key.as_bytes(), &ts_data)?;
        match body {
            Cached::Valid(data) => tx.insert(url, data)?,
            Cached::Error => tx.remove(url)?, // Missing contents indicates error.
            Cached::Invalid => unimplemented!(),
        };
        Ok(())
    }))
}

pub async fn fetch(
    db: &sled::Db,
    client: &reqwest::Client,
    url: &str,
) -> Result<Option<sled::IVec>, Error> {
    match cache_get(db, url)? {
        Cached::Valid(body) => Ok(Some(body)),
        Cached::Error => Ok(None),
        Cached::Invalid => {
            // Cache miss.
            let res = client
                .get(url)
                .header("Accept", "application/json")
                .send()
                .await?;
            if res.status() == StatusCode::OK {
                let body = res.bytes().await?;
                cache_set(db, url, Cached::Valid(body.as_ref()))?;
                Ok(Some(body.as_ref().into()))
            } else {
                cache_set(db, url, Cached::Error)?;
                Ok(None)
            }
        }
    }
}
