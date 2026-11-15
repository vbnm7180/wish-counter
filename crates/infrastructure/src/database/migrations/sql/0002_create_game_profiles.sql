-- Миграция 2: профиль игры — настройки, относящиеся к аккаунту в конкретной игре.
--
-- Пока здесь только сервер игрока: от него зависит часовой пояс времени
-- в истории молитв (см. domain::region::ServerRegion).

CREATE TABLE game_profiles (
    -- Игра: 'genshin', 'hsr', ...
    game_id       TEXT PRIMARY KEY,
    -- Сервер игрока.
    server_region TEXT NOT NULL CHECK (server_region IN ('america', 'europe', 'asia', 'china'))
);
