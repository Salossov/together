# Together

Custom authentication backend and web frontend for a small private
**Minecraft: Java Edition** server running **NeoForge 1.21.1** with
the *Create Aeronautics* modpack.

The server runs in offline-mode (`online-mode=false`) for performance
and modpack flexibility. This project provides a separate verification
layer so we still know who is who - both for licensed Microsoft accounts
and for friends who only want a local password.

> Personal, non-commercial project for ~10–30 friends.
> Not affiliated with Mojang or Microsoft.

## Features

- 🔐 **Local accounts** - username + bcrypt password, stored in our own DB
- 🅼 **Microsoft sign-in** - full XBL → XSTS → Mojang chain to verify
  Java Edition ownership and pull the player's official UUID and skin
- 🆔 **Permanent `internal_uuid`** - survives nickname changes
- 🎨 **Skin upload** - PNG 64×64 / 64×32, optional Mojang signature via Mineskin
- 🤝 **Server handshake** - server-side mod calls our API during login phase
  to validate JWT and pull the player's UUID/skin
- 📦 **Update manifest** - launcher checks SHA-256 of mod files against a
  server manifest

## Stack

- **Backend** - Rust 1.95+, Axum 0.7, sqlx 0.8 (SQLite/PostgreSQL/MySQL,
  picked at runtime via `DATABASE_URL`), JWT (HS256), bcrypt
- **Frontend** - Vue 3.5, Vite 8, TypeScript 6, Pinia 3, vue-router 5,
  TailwindCSS 4
- **DB** - choose at runtime; optional secondary `BACKUP_DATABASE_URL`
  for write mirroring (e.g. Postgres + SQLite snapshot)

## Architecture

```
              ┌──────────────┐
              │   Browser    │
              │  (Vue SPA)   │
              └──────┬───────┘
                     │ JWT
                     ▼
   ┌──────────────────────────────────┐
   │      Axum backend (Rust)         │
   │  /api/auth/*  /api/profile/*     │
   │  /api/launcher/*                 │
   │  /api/mc-server/validate_handshake
   └──────┬─────────────────┬─────────┘
          │                 │
          ▼                 ▼
   ┌─────────────┐  ┌──────────────────┐
   │ DB (any of  │  │ Microsoft / Xbox │
   │ sqlite/pg/  │  │ Live / Mojang    │
   │ mysql)      │  │ (login_with_xbox)│
   └─────────────┘  └──────────────────┘
```

During in-game login the server-side companion mod calls
`POST /api/mc-server/validate_handshake` with the player's username and
JWT. If the JWT is valid and matches, the backend returns the canonical
`internal_uuid` and the player's skin (signed if available); otherwise
401 and the mod kicks the player.

## Endpoints (high level)

| Method | Path | For |
|---|---|---|
| `POST` | `/api/auth/register`              | Frontend - local sign-up |
| `POST` | `/api/auth/login`                 | Frontend - local sign-in |
| `GET`  | `/api/auth/microsoft/url`         | Frontend - get authorize URL |
| `POST` | `/api/auth/microsoft`             | Frontend - exchange MS auth code |
| `POST` | `/api/profile/skin`               | Frontend - skin upload (auth) |
| `GET`  | `/api/profile/me`                 | Frontend - current user (auth) |
| `POST` | `/api/launcher/verify_token`      | Launcher - JWT validity check |
| `GET`  | `/api/launcher/version_check`     | Launcher - mod hash diff |
| `POST` | `/api/mc-server/validate_handshake` | Server mod - login phase |

See `backend/README.md` for full details and field-by-field schemas.

## Running

```bash
# Backend
cd backend
cp .env.example .env       # edit DATABASE_URL, JWT_SECRET
cargo run --release

# Frontend
cd frontend
cp .env.example .env       # edit VITE_API_BASE
bun install
bun run dev                # http://127.0.0.1:5173
```

See `backend/README.md` and `frontend/README.md` for details.

## Privacy

We collect the absolute minimum needed to make the auth work and the
skin show up in-game. See [PRIVACY.md](./PRIVACY.md).

## Security

- `#![deny(unsafe_code)]` across the Rust backend
- Bcrypt password hashing with `tokio::task::spawn_blocking`
- HS256 JWT, secret kept server-side, 30-day TTL
- Request body limit 2 MiB, request timeout 20 s
- Microsoft OAuth: confidential client, `client_secret` only on the
  server, no Microsoft tokens stored after the chain completes
- All Mojang/Microsoft API requests are server-to-server over HTTPS

## License

[MIT](./LICENSE)

## Contact

Maintainer: **Salossov** - aplyt228@gmail.com

For Microsoft Identity Platform / Mojang AppID review questions, please
contact the email above.
