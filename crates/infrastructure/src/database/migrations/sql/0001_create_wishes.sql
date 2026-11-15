-- Миграция 1: таблица истории молитв.
--
-- Одна строка = одна молитва. Поля соответствуют domain::wish::Wish.
-- Миграции никогда не меняются после выпуска: изменения схемы — только
-- новым файлом 0002_..., 0003_... (см. migrations.rs).

CREATE TABLE wishes (
    -- Порядковый номер добавления. Нужен, чтобы молитвы с одинаковым
    -- временем (10 молитв за раз) сохраняли исходный порядок.
    row_id      INTEGER PRIMARY KEY,
    -- Игра: 'genshin', 'hsr', ...
    game_id     TEXT    NOT NULL,
    -- Идентификатор молитвы от игры (строка: 19 цифр).
    wish_id     TEXT    NOT NULL,
    -- Код баннера (gacha_type), например 301.
    banner_code INTEGER NOT NULL,
    -- Идентификатор предмета, например 'hu_tao'.
    item_id     TEXT    NOT NULL,
    -- Тип предмета.
    item_kind   TEXT    NOT NULL CHECK (item_kind IN ('character', 'weapon')),
    -- Редкость в звёздах.
    rarity      INTEGER NOT NULL CHECK (rarity BETWEEN 3 AND 5),
    -- Время по серверу игры: 'ГГГГ-ММ-ДД чч:мм:сс'.
    wish_time   TEXT    NOT NULL,
    -- Одна и та же молитва не может быть сохранена дважды.
    UNIQUE (game_id, wish_id)
);

-- Индекс под основной запрос: все молитвы игры по времени.
CREATE INDEX idx_wishes_game_time ON wishes (game_id, wish_time, row_id);
