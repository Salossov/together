# together-auth

Бэкенд авторизации/скинов/обновлений для офлайн-сервера Minecraft (NeoForge 1.21.1).

## Сборка и запуск

Все три драйвера (SQLite, PostgreSQL, MySQL/MariaDB) вкомпилированы всегда -
драйвер выбирается **в рантайме** по схеме `DATABASE_URL`:

```bash
cp .env.example .env       # отредактировать DATABASE_URL и JWT_SECRET
cargo build --release
cargo run   --release
```

Миграции применяются автоматически при старте (из `./migrations/<backend>`,
определяется по схеме URL).

### Примеры `DATABASE_URL`

```env
DATABASE_URL=sqlite://./data/together.db?mode=rwc
DATABASE_URL=postgres://user:pass@localhost:5432/together
DATABASE_URL=mysql://user:pass@localhost:3306/together
```

### Резервная БД для сохранности (опционально)

`BACKUP_DATABASE_URL` указывает на вторую базу - может быть **другого типа**,
например, основная PostgreSQL + бэкап SQLite-файл:

```env
DATABASE_URL=postgres://user:pass@db/together
BACKUP_DATABASE_URL=sqlite://./data/together.backup.db?mode=rwc
```

Каждая успешная запись (INSERT/UPDATE/DELETE) дублируется в бэкап.
Ошибка записи в бэкап **не валит запрос** - только пишется `warn`-лог
(`target=backup_db`). Чтение всегда идёт с основной БД.

Чтобы отключить - оставьте переменную пустой или закомментируйте.

## Эндпоинты

### Фронт / панель игрока
| Метод | Путь | Назначение |
|---|---|---|
| POST | `/api/auth/register`  | Регистрация (LOCAL): `{username, email?, password}` → JWT |
| POST | `/api/auth/login`     | Логин по паролю |
| POST | `/api/auth/microsoft` | OAuth-обмен `{code}` → JWT (auth_type=MICROSOFT) |
| POST | `/api/profile/skin`   | multipart `file` (PNG 64×64 / 64×32). Bearer JWT |
| GET  | `/api/profile/me`     | Профиль текущего пользователя. Bearer JWT |

### Лаунчер / клиентский мод
| Метод | Путь | Назначение |
|---|---|---|
| POST | `/api/launcher/verify_token`  | `{token}` → `{status: "valid"\|"invalid", username?, internal_uuid?}` |
| GET  | `/api/launcher/version_check` | `{files: {path: sha256}}` → `{update: [...], remove: [...]}` |

### Серверный мод (LOGIN handshake)
| Метод | Путь | Назначение |
|---|---|---|
| POST | `/api/mc-server/validate_handshake` | `{username, jwt_token}` → `{status, internal_uuid, skin}` либо 401 |

## JWT

- Алгоритм: HS256
- TTL: `JWT_TTL_DAYS` (по умолчанию 30)
- Claims: `sub` = `internal_uuid`, `username`, `auth_type`, `iat`, `exp`
- Заголовок: `Authorization: Bearer <jwt>`

## CORS

Список доменов через запятую в `CORS_ORIGINS`, либо `*` для разрешения всех источников
(в этом случае `allow_credentials` отключается автоматически).

## Манифест обновлений

`MANIFEST_PATH` указывает на JSON вида:

```json
{
  "files": [
    { "path": "mods/create-aeronautics.jar", "sha256": "abc...", "url": "https://cdn/.../file.jar", "size": 12345 }
  ]
}
```

Клиент шлёт `{path: sha256}` своих файлов; сервер возвращает что докачать
(`update`) и что удалить (`remove`).

## Безопасность

- `#![deny(unsafe_code)]`
- bcrypt хэширование пароля (вынесено в `spawn_blocking`)
- HS256 JWT, секрет в `JWT_SECRET` (≥ 32 символов)
- Лимит тела запроса 2 MiB, таймаут запроса 20 с
- При смене ника аккаунт остаётся прежним - идентифицируется по `internal_uuid`
