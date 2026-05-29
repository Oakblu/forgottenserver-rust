#![cfg(feature = "e2e")]

pub mod seed;

use std::net::SocketAddr;
use std::time::Duration;

fn repo_root() -> std::path::PathBuf {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/e2e → crates → repo root (where Cargo.toml workspace lives)
    manifest_dir
        .ancestors()
        .find(|p| p.join("Cargo.toml").exists() && p.join("docker-compose.yml").exists())
        .expect("could not locate repo root (expected Cargo.toml + docker-compose.yml)")
        .to_path_buf()
}

use testcontainers::{
    core::{wait::LogWaitStrategy, IntoContainerPort, Mount, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

pub struct ServerFixture {
    _mariadb: ContainerAsync<GenericImage>,
    _server: ContainerAsync<GenericImage>,
    // _config_dir must outlive the server container (config file still mounted)
    _config_dir: tempfile::TempDir,
    status_port: u16,
    game_port: u16,
    http_port: u16,
    // Runtime declared last — drops last, after container handles
    _rt: tokio::runtime::Runtime,
}

impl ServerFixture {
    pub fn start() -> Self {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

        let (mariadb, server, config_dir, status_port, game_port, http_port) = rt.block_on(async {
            // ── 1. MariaDB ────────────────────────────────────────────────
            // We mount the shared init script and schema so the database is
            // pre-bootstrapped the same way the production stack does it
            // (via `mariadb < schema.sql`).  This avoids the DELIMITER //
            // syntax error that occurs when the server's own bootstrap code
            // tries to execute the raw SQL file.
            let repo_root = repo_root();
            let schema_path = repo_root.join("schema.sql");
            let init_script_path = repo_root.join("docker/mariadb-init/00-init-tibia-dbs.sh");

            let mariadb = GenericImage::new("mariadb", "11")
                // "ready for connections" on stderr fires TWICE: once for the
                // temporary server (before docker-entrypoint-initdb.d scripts run)
                // and once for the real server after init scripts complete.
                // We must wait for the SECOND occurrence so that the init script
                // has applied schema.sql before the forgottenserver-rust container
                // tries to connect.
                .with_wait_for(WaitFor::log(
                    LogWaitStrategy::stderr("ready for connections").with_times(2),
                ))
                .with_env_var("MARIADB_ROOT_PASSWORD", "forgottenserver")
                .with_env_var("MARIADB_USER", "forgottenserver")
                .with_env_var("MARIADB_PASSWORD", "forgottenserver")
                .with_mount(Mount::bind_mount(
                    schema_path.to_str().expect("schema path not UTF-8"),
                    "/opt/tfs-schema.sql",
                ))
                .with_mount(Mount::bind_mount(
                    init_script_path
                        .to_str()
                        .expect("init script path not UTF-8"),
                    "/docker-entrypoint-initdb.d/00-init-tibia-dbs.sh",
                ))
                .start()
                .await
                .expect("MariaDB container failed to start — ensure Docker is running");

            let mariadb_port = mariadb
                .get_host_port_ipv4(3306)
                .await
                .expect("MariaDB port mapping missing");

            // ── 2. config.lua with dynamic MariaDB port ───────────────────
            // Must use /tmp — Docker Desktop on macOS only shares /tmp by default.
            let config_dir = tempfile::Builder::new()
                .prefix("forgottenserver-e2e-")
                .tempdir_in("/tmp")
                .expect("failed to create temp dir under /tmp");

            let config_content = format!(
                r#"worldType = "pvp"
ip = "127.0.0.1"
bindOnlyGlobalAddress = false
gameProtocolPort = 7172
statusProtocolPort = 7171
httpPort = 8080
httpWorkers = 1
maxPlayers = 100

mysqlHost = "host.docker.internal"
mysqlUser = "forgottenserver"
mysqlPass = "forgottenserver"
mysqlDatabase = "tibia_rs"
mysqlPort = {mariadb_port}

serverName = "E2E Test Server"
ownerName = "Test Owner"
ownerEmail = "test@example.com"
url = "http://localhost/"
location = "Test"
motd = "E2E test instance."
adminPassword = "test-admin"

mapName = "forgotten"
mapAuthor = "Komic"
"#
            );

            let config_path = config_dir.path().join("config.lua");
            std::fs::write(&config_path, &config_content).expect("failed to write test config.lua");

            // ── 3. forgottenserver-rust ────────────────────────────────────
            let server = GenericImage::new("monorepo-forgottenserver-rust", "latest")
                .with_wait_for(WaitFor::message_on_stdout(">> Forgotten Server Online!"))
                .with_mount(Mount::bind_mount(
                    config_path.to_str().expect("config path not UTF-8"),
                    "/srv/config.lua",
                ))
                .with_mapped_port(0, 7171u16.tcp())
                .with_mapped_port(0, 7172u16.tcp())
                .with_mapped_port(0, 8080u16.tcp())
                .start()
                .await
                .expect(
                    "forgottenserver-rust container failed to start — \
                     ensure the image is built: \
                     docker compose build forgottenserver-rust",
                );

            let status_port = server
                .get_host_port_ipv4(7171)
                .await
                .expect("status port mapping missing");
            let game_port = server
                .get_host_port_ipv4(7172)
                .await
                .expect("game port mapping missing");
            let http_port = server
                .get_host_port_ipv4(8080)
                .await
                .expect("http port mapping missing");

            seed::seed_db(&mariadb).await;

            (
                mariadb,
                server,
                config_dir,
                status_port,
                game_port,
                http_port,
            )
        });

        ServerFixture {
            _mariadb: mariadb,
            _server: server,
            _config_dir: config_dir,
            status_port,
            game_port,
            http_port,
            _rt: rt,
        }
    }

    pub fn status_port(&self) -> u16 {
        self.status_port
    }

    pub fn game_port(&self) -> u16 {
        self.game_port
    }

    pub fn status_addr(&self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], self.status_port))
    }

    pub fn game_addr(&self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], self.game_port))
    }

    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    pub fn http_addr(&self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], self.http_port))
    }
}

/// Open a TCP connection to `addr`, write `request`, read until EOF or 5 s,
/// return all bytes received. Panics on connect/write failure (genuine errors).
/// Returns an empty Vec on read timeout (server closed before sending anything).
pub fn tcp_roundtrip(addr: SocketAddr, request: &[u8]) -> Vec<u8> {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let mut stream =
        TcpStream::connect_timeout(&addr, Duration::from_secs(5)).expect("TCP connect failed");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream.write_all(request).expect("TCP write failed");
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);
    buf
}
