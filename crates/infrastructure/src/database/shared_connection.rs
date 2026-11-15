//! Общее соединение с базой для нескольких хранилищ.

use std::sync::{Arc, Mutex, MutexGuard};

use application::ports::RepositoryError;
use rusqlite::Connection;

/// Соединение с базой, которое делят между собой все хранилища
/// (молитвы, профили игр и т.д.).
///
/// - `Mutex` пропускает к соединению только один поток за раз:
///   `rusqlite::Connection` нельзя использовать из нескольких потоков одновременно;
/// - `Arc` позволяет нескольким хранилищам владеть одним соединением.
///
/// `#[derive(Clone)]` копирует только указатель `Arc`, а не само соединение.
#[derive(Clone)]
pub struct SharedConnection(Arc<Mutex<Connection>>);

impl SharedConnection {
    /// Оборачивает открытое соединение (см. `open_database`).
    pub fn new(connection: Connection) -> Self {
        Self(Arc::new(Mutex::new(connection)))
    }

    /// Захватывает соединение на время одного запроса.
    ///
    /// `Mutex` «отравляется» (poisoned), если поток упал, удерживая его.
    /// Тогда возвращаем ошибку, а не падаем следом.
    pub fn lock(&self) -> Result<MutexGuard<'_, Connection>, RepositoryError> {
        self.0
            .lock()
            .map_err(|_| RepositoryError::Io("соединение с базой недоступно после сбоя".into()))
    }
}
