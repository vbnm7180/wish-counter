-- Миграция 3: в профиль игры добавляется UID игрока, сервер становится необязательным.
--
-- UID может быть известен раньше сервера (например, из файла без сервера),
-- поэтому оба поля допускают NULL. SQLite не умеет снимать NOT NULL у столбца,
-- поэтому таблица пересоздаётся: новая → копирование данных → удаление старой.

CREATE TABLE game_profiles_new (
    -- Игра: 'genshin', 'hsr', ...
    game_id       TEXT PRIMARY KEY,
    -- Сервер игрока; NULL — не выбран.
    server_region TEXT CHECK (server_region IN ('america', 'europe', 'asia', 'china')),
    -- UID игрока (только цифры, 6–12 знаков); NULL — неизвестен.
    uid           TEXT CHECK (uid NOT GLOB '*[^0-9]*' AND length(uid) BETWEEN 6 AND 12)
);

INSERT INTO game_profiles_new (game_id, server_region)
SELECT game_id, server_region FROM game_profiles;

DROP TABLE game_profiles;

ALTER TABLE game_profiles_new RENAME TO game_profiles;
