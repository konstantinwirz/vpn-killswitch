use log::info;
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};
use rusqlite::{Connection, OptionalExtension};
use std::fmt::{Display, Formatter, write};
use time::Duration;
//
// 1. create an instance of Ip2Asn
// 2. if the sqlite db doesn't exist, create it and initialize tables and indices
//    required tables: ip2asn and ip2asn_status
//    ip2asn: ip_start(idx), ip_end(idx), asn
//    ip2asn_status: last_updated
// 3. db does exist and is initialized
// 4. check the status (ip2asn_state table) if the last update is older than 1hr, get new data and replace the db
//    (make it in a loop)
// 5. provide an interface to get the asn for a given ip
#[derive(Debug)]
pub struct Ip2Asn {
    db_path: String,
    ip2asn_db_url: String,
    conn: Connection,
}

#[derive(Debug)]
pub enum Ip2AsnError {
    DbError(rusqlite::Error),
    IoError(std::io::Error),
    HttpError(reqwest::Error),
}

impl Display for Ip2AsnError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Ip2AsnError::DbError(err) => write!(f, "DB error: {}", err),
            Ip2AsnError::IoError(err) => write!(f, "IO error: {}", err),
            Self::HttpError(err) => write!(f, "HTTP Error: {}", err),
        }
    }
}

impl std::error::Error for Ip2AsnError {}

impl From<rusqlite::Error> for Ip2AsnError {
    fn from(err: rusqlite::Error) -> Self {
        Ip2AsnError::DbError(err)
    }
}

impl From<std::io::Error> for Ip2AsnError {
    fn from(value: std::io::Error) -> Self {
        Ip2AsnError::IoError(value)
    }
}

impl From<reqwest::Error> for Ip2AsnError {
    fn from(value: reqwest::Error) -> Self {
        Ip2AsnError::HttpError(value)
    }
}

const DEFAULT_DB_PATH: &str = "ip2asn.sqlite";
const UPDATE_INTERVAL_HOURS: u8 = 1;

#[derive(Debug, Clone)]
pub struct Ip2AsnEntry {
    ip_start: u32,
    ip_end: u32,
    asn: u32,
    updated_at: Option<time::OffsetDateTime>,
}

impl Ip2AsnEntry {
    pub fn new(ip_start: u32, ip_end: u32, asn: u32) -> Self {
        Self {
            ip_start,
            ip_end,
            asn,
            updated_at: None,
        }
    }
}

impl Ip2Asn {
    pub fn new(db_path: &str, ip2asn_db_url: &str) -> Result<Self, Ip2AsnError> {
        Ok(Self {
            conn: Self::init_db()?,
            db_path: db_path.to_owned(),
            ip2asn_db_url: ip2asn_db_url.to_owned(),
        })
    }

    fn init_db() -> Result<Connection, Ip2AsnError> {
        if std::fs::exists(DEFAULT_DB_PATH)? {
            return Ok(Connection::open(DEFAULT_DB_PATH)?);
        }

        let conn = Connection::open(DEFAULT_DB_PATH)?;
        // db doesn't exist, create it
        conn.execute_batch(
            "BEGIN;
            CREATE TABLE ip2asn (ip_start INTEGER, ip_end INTEGER, asn INTEGER, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, PRIMARY KEY (ip_start, ip_end));
            CREATE INDEX ip2asn_idx_start ON ip2asn (ip_start);
            CREATE INDEX ip2asn_idx_end ON ip2asn (ip_end);
            CREATE INDEX ip2asn_idx_updated_at ON ip2asn (updated_at);
            COMMIT;",
        )?;

        Ok(conn)
    }

    fn oldest_updated_at(&self) -> Result<Option<time::OffsetDateTime>, Ip2AsnError> {
        let mut stmt = self
            .conn
            .prepare("SELECT updated_at FROM ip2asn ORDER BY updated_at DESC LIMIT 1")?;
        let updated_at: Option<time::OffsetDateTime> =
            stmt.query_one([], |row| row.get(0).into()).optional()?;
        Ok(updated_at)
    }

    fn store(&self, entry: Ip2AsnEntry) -> Result<(), Ip2AsnError> {
        let mut stmt = self
            .conn
            .prepare("INSERT INTO ip2asn (ip_start, ip_end, asn) VALUES (?, ?, ?)")?;
        stmt.execute(&[&entry.ip_start, &entry.ip_end, &entry.asn])?;
        Ok(())
    }

    fn store_many(&self, entries: &[Ip2AsnEntry]) -> Result<usize, Ip2AsnError> {
        //self.conn.execute_batch()
        todo!()
    }

    pub async fn run(&self) -> Result<(), Ip2AsnError> {
        loop {
            // check the oldest entry in ip2asn table, if it's older than 1hr, update the db
            // if we don't have to update - sleep
            // if we have to update -
            //     get the latest list
            //     update the entries
            //
            // return err if any error occurs

            let should_update_db = self
                .oldest_updated_at()?
                .map(|updated_at| {
                    updated_at + Duration::hours(UPDATE_INTERVAL_HOURS as i64)
                        > time::OffsetDateTime::now_utc()
                })
                .unwrap_or(true);

            dbg!(should_update_db);

            info!("Should update db?: {}", should_update_db);

            if should_update_db {
                let client = reqwest::Client::new();
                // download a file from self.ip2asn_db_url
                let response: reqwest::Response = client.get(&self.ip2asn_db_url).send().await?;
                dbg!(response);
                //let content = response.text()?;
                // parse the content and store the entries
                //let entries = parse_ip2asn_content(&content)?;
                //self.store_many(&entries)?;
            }

            break Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[tokio::test]
    async fn ip2asn_create_db() -> Result<(), Ip2AsnError> {
        env_logger::init();

        // make sure db doesn't exist
        assert!(std::fs::remove_file(DEFAULT_DB_PATH).is_ok());

        let ip2asn = Ip2Asn::new(DEFAULT_DB_PATH, "some.url")?;
        assert!(ip2asn.oldest_updated_at()?.is_none());

        ip2asn.store(Ip2AsnEntry::new(1, 2, 3))?;

        assert!(ip2asn.oldest_updated_at()?.is_some());

        let ts = ip2asn.oldest_updated_at()?.unwrap();

        dbg!(ts);

        ip2asn.run().await?;

        // make sure db
        Ok(())
    }
}
