use crate::{DatabasePool, Session, SessionError, SessionStore};
use async_trait::async_trait;
use redis_pool::ClusterRedisPool;

///Redis's Session Helper type for the DatabasePool.
pub type SessionRedisClusterSession = Session<SessionRedisClusterPool>;
///Redis's Session Store Helper type for the DatabasePool.
pub type SessionRedisClusterSessionStore = SessionStore<SessionRedisClusterPool>;

///Redis's Cluster Pool type for the DatabasePool. Needs a redis ClusterClient.
#[derive(Clone)]
pub struct SessionRedisClusterPool {
    pool: ClusterRedisPool,
}

impl From<ClusterRedisPool> for SessionRedisClusterPool {
    fn from(pool: ClusterRedisPool) -> Self {
        SessionRedisClusterPool { pool }
    }
}

impl std::fmt::Debug for SessionRedisClusterPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionRedisClusterPool").finish()
    }
}

#[async_trait]
impl DatabasePool for SessionRedisClusterPool {
    async fn initiate(&self, _table_name: &str) -> Result<(), SessionError> {
        // Redis does not actually use Tables so there is no way we can make one.
        Ok(())
    }

    async fn delete_by_expiry(&self, _table_name: &str) -> Result<Vec<String>, SessionError> {
        // Redis does this for use using the Expiry Options.
        Ok(Vec::new())
    }

    async fn count(&self, _table_name: &str) -> Result<i64, SessionError> {
        let mut con = self.pool.aquire().await?;
        let count: i64 = redis::cmd("DBSIZE").query_async(&mut con).await?;

        Ok(count)
    }

    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        _table_name: &str,
    ) -> Result<(), SessionError> {
        let mut con = self.pool.aquire().await?;
        redis::pipe()
            .set(id, session)
            .ignore()
            .expire_at(id, expires as usize)
            .ignore()
            .query_async(&mut con)
            .await?;
        Ok(())
    }

    async fn load(&self, id: &str, _table_name: &str) -> Result<Option<String>, SessionError> {
        let mut con = self.pool.aquire().await?;
        let result: String = redis::cmd("GET").arg(id).query_async(&mut con).await?;
        Ok(Some(result))
    }

    async fn delete_one_by_id(&self, id: &str, _table_name: &str) -> Result<(), SessionError> {
        let mut con = self.pool.aquire().await?;
        redis::cmd("DEL").arg(id).query_async(&mut con).await?;
        Ok(())
    }

    async fn exists(&self, id: &str, _table_name: &str) -> Result<bool, SessionError> {
        let mut con = self.pool.aquire().await?;
        let exists: bool = redis::cmd("EXISTS").arg(id).query_async(&mut con).await?;

        Ok(exists)
    }

    async fn delete_all(&self, _table_name: &str) -> Result<(), SessionError> {
        let mut con = self.pool.aquire().await?;
        redis::cmd("FLUSHDB").query_async(&mut con).await?;
        Ok(())
    }

    async fn get_ids(&self, _table_name: &str) -> Result<Vec<String>, SessionError> {
        let mut con = self.pool.aquire().await?;
        let result: Vec<String> = redis::cmd("KEYS").arg("*").query_async(&mut con).await?;
        Ok(result)
    }

    fn auto_handles_expiry(&self) -> bool {
        true
    }
}
