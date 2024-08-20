use bollard::secret::HealthConfig;
#[cfg(feature = "postgres")]
use datafusion_table_providers::sql::db_connection_pool::postgrespool::PostgresConnectionPool;
use rand::Rng;
use secrecy::SecretString;
use std::collections::HashMap;
use tracing::instrument;

use crate::{
    container_registry,
    docker::{ContainerRunnerBuilder, RunningContainer},
};

const PG_PASSWORD: &str = "runtime-integration-test-pw";
const PG_DOCKER_CONTAINER: &str = "runtime-integration-test-postgres";

fn get_pg_params(port: usize) -> HashMap<String, SecretString> {
    let mut params = HashMap::new();
    params.insert(
        "pg_host".to_string(),
        SecretString::from("localhost".to_string()),
    );
    params.insert("pg_port".to_string(), SecretString::from(port.to_string()));
    params.insert(
        "pg_user".to_string(),
        SecretString::from("postgres".to_string()),
    );
    params.insert(
        "pg_pass".to_string(),
        SecretString::from(PG_PASSWORD.to_string()),
    );
    params.insert(
        "pg_db".to_string(),
        SecretString::from("postgres".to_string()),
    );
    params.insert(
        "pg_sslmode".to_string(),
        SecretString::from("disable".to_string()),
    );
    params
}

pub(super) fn get_random_port() -> usize {
    rand::thread_rng().gen_range(15432..65535)
}

#[instrument]
pub(super) async fn start_postgres_docker_container(
    port: usize,
) -> Result<RunningContainer<'static>, anyhow::Error> {
    let container_name = format!("{PG_DOCKER_CONTAINER}-{port}");
    let port = if let Ok(port) = port.try_into() {
        port
    } else {
        15432
    };

    let running_container = ContainerRunnerBuilder::new(container_name)
        .image(format!("{}postgres:latest", container_registry()))
        .add_port_binding(5432, port)
        .add_env_var("POSTGRES_PASSWORD", PG_PASSWORD)
        .healthcheck(HealthConfig {
            test: Some(vec![
                "CMD-SHELL".to_string(),
                "pg_isready -U postgres".to_string(),
            ]),
            interval: Some(250_000_000), // 250ms
            timeout: Some(100_000_000),  // 100ms
            retries: Some(5),
            start_period: Some(500_000_000), // 100ms
            start_interval: None,
        })
        .build()?
        .run()
        .await?;

    tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
    Ok(running_container)
}

#[instrument]
pub(super) async fn get_postgres_connection_pool(
    port: usize,
) -> Result<PostgresConnectionPool, anyhow::Error> {
    let pool = PostgresConnectionPool::new(get_pg_params(port)).await?;

    Ok(pool)
}
