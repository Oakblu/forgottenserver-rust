#![cfg(feature = "e2e")]

use testcontainers::{core::ExecCommand, ContainerAsync, GenericImage};

/// Insert a test account and one character into the MariaDB container.
///
/// Account: name="test", password=SHA1("test"), type=1
/// Character: name="Testchar", account_id=1, town_id=1, pos=(160,54,7)
///
/// Schema reference: forgottenserver/schema.sql
pub async fn seed_db(mariadb: &ContainerAsync<GenericImage>) {
    let sql = concat!(
        "INSERT IGNORE INTO accounts (id, name, password, type) ",
        "VALUES (1, 'test', SHA1('test'), 1); ",
        "INSERT IGNORE INTO players ",
        "(id, name, account_id, vocation, level, health, healthmax, town_id, posx, posy, posz, cap, sex) ",
        "VALUES (1, 'Testchar', 1, 0, 1, 150, 150, 1, 160, 54, 7, 400, 0);"
    );

    let exec_result = mariadb
        .exec(ExecCommand::new([
            "mariadb",
            "-uforgottenserver",
            "-pforgottenserver",
            "tibia_rs",
            "-e",
            sql,
        ]))
        .await
        .expect("seed_db: failed to exec in container");

    let exit_code = exec_result
        .exit_code()
        .await
        .expect("seed_db: failed to read exit code");
    assert_eq!(
        exit_code,
        Some(0),
        "seed_db: MariaDB SQL failed with exit code {exit_code:?}"
    );
}
