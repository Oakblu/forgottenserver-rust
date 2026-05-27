# forgottenserver-rust

Rust migration of the ForgottenServer C++ MMORPG server.

---

## Playing Locally with Docker

This guide covers running the **C++ ForgottenServer** (located at `../forgottenserver/`) + **OTClient** in Docker on macOS so you can play the game. All steps are reproducible from scratch.

### Architecture

```
macOS Host
├── Docker Desktop
│   └── compose network
│       ├── db   (mariadb:11 — internal only)
│       └── tfs  (forgottenserver — ports 7171/7172/8080 → host)
└── XQuartz (X11 server on :0)
    └── otclient container (docker run with X11 forwarding)
```

OTClient runs as a standalone `docker run` (not in compose) because interactive X11 display access is incompatible with detached compose on macOS.

---

### Prerequisites (one-time setup)

**Docker Desktop** — must be running.

```bash
docker --version        # 24+ required
docker compose version  # v2.x required
```

**XQuartz** — required for the GUI client on macOS.

```bash
brew install --cask xquartz
# After install: log out and log back in (XQuartz hooks into the session at login)
```

**Configure XQuartz to allow TCP connections** (required for Docker X11 forwarding):

```bash
defaults write org.xquartz.X11 nolisten_tcp 0
# Fully quit XQuartz and relaunch it
```

Alternatively: open XQuartz → Preferences → Security → check **"Allow connections from network clients"**.

---

### Step 1 — Start the server stack

The `docker-compose.yml` at `../docker-compose.yml` (relative to this repo, at `apps/poketibia/docker-compose.yml`) orchestrates MariaDB and ForgottenServer.

```bash
cd /path/to/apps/poketibia

# First boot: builds the TFS image from source (~5–10 min)
docker compose up --build -d

# Watch logs — wait for "Game server running on port 7172"
docker compose logs -f tfs

# Check service health
docker compose ps
```

The `db` service must reach `(healthy)` before `tfs` starts (enforced by `depends_on: condition: service_healthy`).

---

### Step 2 — Create a game account and character

ForgottenServer has no web interface. Insert rows directly into MariaDB:

```bash
# Create account
docker compose exec db \
  mariadb -u forgottenserver -ptfs_secret forgottenserver \
  -e "INSERT INTO accounts (name, password, type, email)
      VALUES ('myaccount', SHA1('password123'), 1, 'me@local.dev');"

# Create character
docker compose exec db \
  mariadb -u forgottenserver -ptfs_secret forgottenserver \
  -e "INSERT INTO players (name, group_id, account_id, level, vocation,
                            health, healthmax, mana, manamax, soul, cap,
                            sex, town_id, looktype, posx, posy, posz)
      SELECT 'MyHero', 1, id, 1, 0,
             150, 150, 0, 0, 100, 400, 1, 1, 136, 160, 55, 7
      FROM accounts WHERE name='myaccount';"
```

---

### Step 3 — Clone and build OTClient (one-time, ~15–20 min)

```bash
cd /path/to/apps/poketibia
git clone https://github.com/opentibiabr/otclient
cd otclient
docker build -t otclient:local .
```

OTClient uses a 3-stage Ubuntu + vcpkg build. The first build is slow; subsequent builds use the Docker layer cache.

---

### Step 4 — Launch OTClient with X11 forwarding

```bash
# Ensure XQuartz is running
open -a XQuartz

# Authorize Docker loopback to connect to X11
xhost + 127.0.0.1

# Verify DISPLAY is :0
echo $DISPLAY   # should print :0

# Run the client
docker run --rm -it \
  -e DISPLAY=host.docker.internal:0 \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  --name otclient \
  otclient:local
```

The game client window will appear on your macOS desktop through XQuartz.

**If the window doesn't open (X11 auth error)**, try the explicit LAN IP form:

```bash
MY_IP=$(ifconfig en0 | grep "inet " | awk '{print $2}')
xhost + "$MY_IP"
docker run --rm -it \
  -e DISPLAY="${MY_IP}:0" \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  --name otclient \
  otclient:local
```

---

### Step 5 — Connect in the client UI

When the OTClient window appears, enter:

| Field    | Value         |
|----------|---------------|
| Server   | `127.0.0.1`   |
| Port     | `7171`        |
| Account  | `myaccount`   |
| Password | `password123` |

---

### Verification

```bash
# DB healthy?
docker compose ps                      # "db" should show "(healthy)"

# Server ports open?
nc -zv 127.0.0.1 7171                  # "Connection ... succeeded!"
nc -zv 127.0.0.1 7172

# TFS responding to status query?
printf '\x06\x00\xff\xff\x79\x6c\x00\x00' | nc -w 2 127.0.0.1 7171 | xxd | head

# Player logged in?
docker compose logs tfs | grep -i "MyHero"
```

---

### Lifecycle

```bash
# Stop (preserves all player data in the named volume)
docker compose down

# Wipe all data and start fresh
docker compose down -v && docker compose up --build -d

# Rebuild only TFS after C++ changes
docker compose build tfs && docker compose up -d tfs
```

---

### Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| TFS exits immediately | DB not ready | Wait for `db (healthy)`, then `docker compose restart tfs` |
| "Cannot connect to database" in TFS logs | Wrong host/password | Verify `mysqlHost = "db"` and `mysqlPass = "tfs_secret"` in `../forgottenserver/config.lua` lines 80/82 |
| OTClient: black window or immediate close | X11 not authorized | Run `xhost + 127.0.0.1` before `docker run` |
| "No protocol specified" in XQuartz | TCP disabled or wrong DISPLAY | Ensure `nolisten_tcp 0` and restart XQuartz; verify `echo $DISPLAY` is `:0` |
| Client: "Cannot connect to server" | Wrong IP in client UI | Use `127.0.0.1:7171` exactly (not `localhost`) |
| Schema errors in DB logs on restart | Init script re-run attempted | Safe to ignore — schema uses `CREATE TABLE IF NOT EXISTS` throughout |
