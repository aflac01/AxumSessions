use axum::{extract::State, routing::get, Router};
use axum_session::{Session, SessionConfig, SessionLayer, SessionRedisPool, SessionStore};
use redis_pool::{RedisPool, SingleRedisPool};
#[tokio::main]
async fn main() {
    // please consider using dotenvy to get this
    // please check the docker-compose file included for the redis image used here
    let redis_url = "redis://default@127.0.0.1:6379/0";

    let client =
        redis::Client::open(redis_url).expect("Error while tryiong to open the redis connection");

    let pool = RedisPool::from(client);
    // No need here to specify a table name because redis does not support tables
    let session_config = SessionConfig::default();

    // create SessionStore and initiate the database tables
    let session_store =
        SessionStore::<SessionRedisPool>::new(Some(pool.clone().into()), session_config)
            .await
            .unwrap();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(root))
        // `POST /users` goes to `counter`
        .route("/counter", get(counter))
        .layer(SessionLayer::new(session_store))
        .with_state(pool); // adding the crate plugin ( layer ) to the project

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn counter(session: Session<SessionRedisPool>, pool: State<SingleRedisPool>) -> String {
    let mut count: usize = session.get("count").unwrap_or(0);
    count += 1;
    session.set("count", count);

    // consider use better Option handling here instead of expect
    let new_count = session.get::<usize>("count").expect("error setting count");

    let count: i64 = redis::cmd("DBSIZE")
        .query_async(&mut pool.aquire().await.unwrap())
        .await
        .unwrap();

    format!("We have set the counter to redis: {new_count}, DBSIZE: {count}")
}
