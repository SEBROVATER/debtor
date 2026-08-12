//! Process-boundary restart coverage for the local application contract.

#![cfg(unix)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use debtor_infra::db::connect;
use sqlx::SqlitePool;

static DATABASE_COUNTER: AtomicUsize = AtomicUsize::new(0);

struct RunningProcess {
    child: Option<Child>,
}

impl Drop for RunningProcess {
    fn drop(&mut self) {
        let Some(mut child) = self.child.take() else {
            return;
        };
        request_termination(&child);
        let _ = child.wait();
    }
}

impl RunningProcess {
    async fn stop(mut self) {
        let mut child = self.child.take().expect("running child");
        request_termination(&child);
        tokio::task::spawn_blocking(move || child.wait())
            .await
            .expect("wait task")
            .expect("child wait");
    }
}

fn request_termination(child: &Child) {
    Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .expect("send SIGTERM");
}

fn database_path() -> PathBuf {
    let id = DATABASE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "debtor-process-restart-{}-{timestamp}-{id}.db",
        std::process::id()
    ))
}

fn database_url(path: &Path) -> String {
    format!("sqlite://{}?mode=rwc", path.display())
}

fn free_address() -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("reserve listener");
    listener.local_addr().expect("reserved address")
}

async fn start_process(path: &Path, address: SocketAddr) -> RunningProcess {
    let mut child = Command::new(env!("CARGO_BIN_EXE_debtor"))
        .env("APP_ADMIN_PASSWORD_HASH", "\u{24}argon2id\u{24}v=19\u{24}m=19456,t=2,p=1\u{24}AAAAAAAAAAAAAAAAAAAAAA\u{24}AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        .env("APP_DATABASE_URL", database_url(path))
        .env("APP_BIND", address.to_string())
        .env("APP_SESSION_COOKIE_SECURE", "false")
        .env("APP_EXCHANGE_BASE_URL", "http://127.0.0.1:1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start debtor process");

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if TcpStream::connect_timeout(&address, Duration::from_millis(50)).is_ok() {
            return RunningProcess { child: Some(child) };
        }
        assert!(
            child.try_wait().expect("poll child").is_none(),
            "debtor exited before socket admission"
        );
        assert!(
            Instant::now() < deadline,
            "debtor did not reach socket admission"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn persisted_group(pool: &SqlitePool) -> String {
    sqlx::query_scalar("SELECT name FROM groups WHERE name = 'ProcessRestarted'")
        .fetch_one(pool)
        .await
        .expect("persisted process-restart group")
}

#[tokio::test]
async fn process_restart_reuses_the_same_database_and_reaches_admission() {
    let path = database_path();
    let first_address = free_address();
    let first = start_process(&path, first_address).await;

    let pool = connect(&database_url(&path)).await.expect("first database");
    sqlx::query("INSERT INTO groups (name, currency) VALUES ('ProcessRestarted', 'USD')")
        .execute(&pool)
        .await
        .expect("persist process-restart state");
    assert_eq!(persisted_group(&pool).await, "ProcessRestarted");
    pool.close().await;
    first.stop().await;

    let second_address = free_address();
    let second = start_process(&path, second_address).await;
    let reopened = connect(&database_url(&path))
        .await
        .expect("restarted database");
    assert_eq!(persisted_group(&reopened).await, "ProcessRestarted");
    let migrations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&reopened)
        .await
        .expect("migration history");
    assert!(migrations > 0);
    reopened.close().await;
    second.stop().await;

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}
