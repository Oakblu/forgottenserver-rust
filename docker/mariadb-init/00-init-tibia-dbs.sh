#!/bin/bash
#
# 00-init-tibia-dbs.sh — Initialize databases for the forgottenserver stack.
#
# Mounted into the db container at
# /docker-entrypoint-initdb.d/00-init-tibia-dbs.sh. Runs once on first
# container start, creates the databases used by the server and harness,
# grants permissions to the existing `forgottenserver` user, and applies
# schema.sql to each.
#
# forgottenserver — production/dev database (matches config.lua default)
# tibia_cpp       — C++ reference server database (harness lane)
# tibia_rs        — Rust port database (harness lane)
# tibia_test      — E2E test database (harness lane)
#
# Schema source is mounted at /opt/tfs-schema.sql by docker-compose.yml.

set -euo pipefail

SCHEMA="/opt/tfs-schema.sql"

if [ ! -f "$SCHEMA" ]; then
  echo "ERROR: schema not found at $SCHEMA" >&2
  exit 1
fi

echo "Initializing forgottenserver, tibia_cpp, tibia_rs, and tibia_test databases..."

for db in forgottenserver tibia_cpp tibia_rs tibia_test; do
  echo "  → creating database '$db'"
  mariadb -uroot -p"$MARIADB_ROOT_PASSWORD" -e "CREATE DATABASE IF NOT EXISTS \`$db\` DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
  echo "  → granting permissions on '$db' to 'forgottenserver'"
  mariadb -uroot -p"$MARIADB_ROOT_PASSWORD" -e "GRANT ALL PRIVILEGES ON \`$db\`.* TO 'forgottenserver'@'%';"
  echo "  → applying schema to '$db'"
  mariadb -uroot -p"$MARIADB_ROOT_PASSWORD" "$db" < "$SCHEMA"
done

mariadb -uroot -p"$MARIADB_ROOT_PASSWORD" -e "FLUSH PRIVILEGES;"
echo "Database initialization complete."
