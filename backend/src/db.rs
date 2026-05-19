//! БД-слой с runtime-выбором драйвера.
//!
//! Драйвер определяется по схеме URL (`sqlite://`, `postgres://`, `mysql://`).
//! Все три всегда вкомпилированы — пересборка при смене не нужна.
//!
//! Опционально вторая БД (`BACKUP_DATABASE_URL`) выступает зеркалом:
//! каждая запись (INSERT/UPDATE/DELETE), отправленная через `Db::write`,
//! дублируется в бэкап. Ошибки бэкапа только логируются, не валят запрос.

use std::{borrow::Cow, path::Path};

use sqlx::{any::AnyPoolOptions, AnyPool};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DbKind {
    Sqlite,
    Postgres,
    Mysql,
}

impl DbKind {
    pub fn from_url(url: &str) -> anyhow::Result<Self> {
        let scheme = url.split(':').next().unwrap_or("").trim().to_ascii_lowercase();
        match scheme.as_str() {
            "sqlite" => Ok(Self::Sqlite),
            "postgres" | "postgresql" => Ok(Self::Postgres),
            "mysql" | "mariadb" => Ok(Self::Mysql),
            other => Err(anyhow::anyhow!(
                "unsupported DATABASE_URL scheme: '{other}://...' (ожидается sqlite|postgres|mysql)"
            )),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
        }
    }

    fn migrations_dir(self) -> &'static str {
        match self {
            Self::Sqlite => "./migrations/sqlite",
            Self::Postgres => "./migrations/postgres",
            Self::Mysql => "./migrations/mysql",
        }
    }
}

pub struct Backup {
    pub pool: AnyPool,
    pub kind: DbKind,
}

pub struct Db {
    pub primary: AnyPool,
    pub kind: DbKind,
    pub backup: Option<Backup>,
}

pub async fn init(primary_url: &str, backup_url: Option<&str>) -> anyhow::Result<Db> {
    sqlx::any::install_default_drivers();

    let primary_kind = DbKind::from_url(primary_url)?;
    ensure_sqlite_dir(primary_url);
    let primary = AnyPoolOptions::new()
        .max_connections(16)
        .connect(primary_url)
        .await?;
    run_migrations(&primary, primary_kind).await?;
    tracing::info!("primary DB connected: {}", primary_kind.name());

    let backup = match backup_url {
        Some(u) if !u.is_empty() => {
            let k = DbKind::from_url(u)?;
            ensure_sqlite_dir(u);
            let pool = AnyPoolOptions::new()
                .max_connections(8)
                .connect(u)
                .await?;
            run_migrations(&pool, k).await?;
            tracing::info!("backup DB connected: {} (writes mirrored)", k.name());
            Some(Backup { pool, kind: k })
        }
        _ => None,
    };

    Ok(Db {
        primary,
        kind: primary_kind,
        backup,
    })
}

fn ensure_sqlite_dir(url: &str) {
    if let Some(rest) = url.strip_prefix("sqlite://") {
        let path = rest.split('?').next().unwrap_or(rest);
        if !path.is_empty() && path != ":memory:" {
            if let Some(parent) = Path::new(path).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
        }
    }
}

async fn run_migrations(pool: &AnyPool, kind: DbKind) -> anyhow::Result<()> {
    let m = sqlx::migrate::Migrator::new(Path::new(kind.migrations_dir())).await?;
    m.run(pool).await?;
    Ok(())
}

/// Переписывает плейсхолдеры `?` (стиль sqlite/mysql) в `$N` (postgres).
pub fn sql_for(kind: DbKind, s: &'static str) -> Cow<'static, str> {
    if kind != DbKind::Postgres {
        return Cow::Borrowed(s);
    }
    let mut out = String::with_capacity(s.len() + 8);
    let mut n: u32 = 0;
    let mut in_str = false;
    for c in s.chars() {
        if c == '\'' {
            in_str = !in_str;
            out.push(c);
        } else if c == '?' && !in_str {
            n += 1;
            out.push('$');
            out.push_str(&n.to_string());
        } else {
            out.push(c);
        }
    }
    Cow::Owned(out)
}

/// Типобезопасный enum параметров для зеркалируемых запросов.
#[derive(Clone, Debug)]
pub enum P {
    Str(String),
    OptStr(Option<String>),
    I64(i64),
}

impl From<&str> for P { fn from(v: &str) -> Self { P::Str(v.to_string()) } }
impl From<String> for P { fn from(v: String) -> Self { P::Str(v) } }
impl From<&String> for P { fn from(v: &String) -> Self { P::Str(v.clone()) } }
impl From<Option<String>> for P { fn from(v: Option<String>) -> Self { P::OptStr(v) } }
impl From<&Option<String>> for P { fn from(v: &Option<String>) -> Self { P::OptStr(v.clone()) } }
impl From<i64> for P { fn from(v: i64) -> Self { P::I64(v) } }

impl Db {
    /// Удобный shortcut для перевода запроса под нужный диалект.
    pub fn sql(&self, s: &'static str) -> Cow<'static, str> {
        sql_for(self.kind, s)
    }

    /// Выполнить write-запрос на primary; при наличии backup — продублировать.
    /// Ошибки backup не валят запрос (только log::warn).
    pub async fn write(
        &self,
        query: &'static str,
        params: Vec<P>,
    ) -> Result<(), sqlx::Error> {
        exec_write(&self.primary, self.kind, query, &params).await?;
        if let Some(b) = &self.backup {
            if let Err(e) = exec_write(&b.pool, b.kind, query, &params).await {
                tracing::warn!(
                    target: "backup_db",
                    "mirror write to {} failed: {e}",
                    b.kind.name()
                );
            }
        }
        Ok(())
    }
}

async fn exec_write(
    pool: &AnyPool,
    kind: DbKind,
    raw: &'static str,
    params: &[P],
) -> Result<(), sqlx::Error> {
    let translated = sql_for(kind, raw);
    let mut q = sqlx::query(&translated);
    for p in params {
        q = match p {
            P::Str(s) => q.bind(s.clone()),
            P::OptStr(s) => q.bind(s.clone()),
            P::I64(n) => q.bind(*n),
        };
    }
    q.execute(pool).await.map(|_| ())
}
