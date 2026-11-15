//! Настройка журнала работы приложения (логов).
//!
//! Логи пишутся в `data/logs/wish-counter.<дата>.log`. Каждый день
//! создаётся новый файл, хранятся последние `MAX_LOG_FILES` (7) файлов.
//! Уровень подробности можно изменить переменной окружения `RUST_LOG`,
//! например `RUST_LOG=debug`.
//!
//! **Безопасность:** в логи нельзя записывать ключи доступа (`authkey`)
//! и другие секреты — это правило действует для всего проекта.

use std::path::Path;

use thiserror::Error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

/// Сколько дневных файлов логов хранить.
const MAX_LOG_FILES: usize = 7;
/// Уровень логирования, если `RUST_LOG` не задана.
const DEFAULT_LOG_FILTER: &str = "info";

/// Ошибка инициализации логирования.
#[derive(Debug, Error)]
pub enum LoggingError {
    /// Не удалось создать файл логов.
    #[error("не удалось создать файл логов: {0}")]
    FileAppender(#[from] tracing_appender::rolling::InitError),
    /// Логирование уже было инициализировано ранее.
    #[error("логирование уже инициализировано: {0}")]
    AlreadyInitialized(#[from] tracing_subscriber::util::TryInitError),
}

/// Включает логирование в файл и в консоль.
///
/// Возвращает «охранника» [`WorkerGuard`]: пока он жив, фоновый поток
/// дописывает логи в файл. Его нужно хранить до конца работы программы,
/// иначе последние записи могут потеряться.
pub fn init_logging(logs_dir: &Path) -> Result<WorkerGuard, LoggingError> {
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("wish-counter")
        .filename_suffix("log")
        .max_log_files(MAX_LOG_FILES)
        .build(logs_dir)?;
    // Запись в файл идёт в отдельном потоке, чтобы не тормозить интерфейс.
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(file_writer).with_ansi(false))
        .with(fmt::layer())
        .try_init()?;

    Ok(guard)
}
