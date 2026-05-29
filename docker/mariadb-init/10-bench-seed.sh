#!/bin/bash
#
# 10-bench-seed.sh — Apply bench-seed.sql to tibia_cpp and tibia_rs if present.
#
# Only runs when /opt/bench-seed.sql is mounted (i.e. in the perf compose stack).
# The 10- prefix ensures it runs after 00-init-tibia-dbs.sh has created the DBs.

set -euo pipefail

SEED="/opt/bench-seed.sql"

if [ ! -f "$SEED" ]; then
  echo "No bench-seed.sql found at $SEED — skipping bot account creation."
  exit 0
fi

echo "Applying bench-seed.sql to tibia_cpp and tibia_rs..."
for db in tibia_cpp tibia_rs; do
  echo "  → seeding '$db'"
  mariadb -uroot -p"$MARIADB_ROOT_PASSWORD" "$db" < "$SEED"
done

echo "Bot accounts seeded successfully."
