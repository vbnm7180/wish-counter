//! Разбор JavaScript-литералов из файлов данных Paimon.moe.
//!
//! Файлы Paimon.moe — это JavaScript-модули вида
//! `export const banners = { characters: [ { name: 'Ballad', ... } ] };`.
//! Выполнять скачанный код небезопасно, поэтому здесь написан маленький
//! разборщик, который **только читает значения**: объекты, массивы, строки,
//! числа, `true/false/null` и ссылки вида `weapons.claymore` (как текст).
//! Функции, шаблоны и выражения не поддерживаются — их в этих файлах нет.

use thiserror::Error;

/// Максимальная вложенность объектов и массивов. Защищает от переполнения
/// стека на испорченном или специально составленном файле.
const MAX_NESTING_DEPTH: usize = 64;

/// Значение JavaScript-литерала.
#[derive(Debug, Clone, PartialEq)]
pub enum JsValue {
    /// `null` или `undefined`.
    Null,
    /// `true` / `false`.
    Bool(bool),
    /// Число.
    Number(f64),
    /// Строка в одинарных, двойных или обратных кавычках.
    String(String),
    /// Массив `[ ... ]`.
    Array(Vec<JsValue>),
    /// Объект `{ ключ: значение }`. Порядок полей сохраняется.
    Object(Vec<(String, JsValue)>),
    /// Ссылка на другую переменную, например `weapons.claymore`.
    /// Значение не вычисляется — сохраняется только имя.
    Reference(String),
}

impl JsValue {
    /// Возвращает строку, если значение — строка.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(text) => Some(text),
            _ => None,
        }
    }

    /// Возвращает число, если значение — число.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(number) => Some(*number),
            _ => None,
        }
    }

    /// Возвращает элементы, если значение — массив.
    pub fn as_array(&self) -> Option<&[JsValue]> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    /// Возвращает поля, если значение — объект.
    pub fn as_object(&self) -> Option<&[(String, JsValue)]> {
        match self {
            Self::Object(fields) => Some(fields),
            _ => None,
        }
    }

    /// Возвращает поле объекта по имени (`None`, если это не объект или поля нет).
    pub fn get(&self, key: &str) -> Option<&JsValue> {
        self.as_object()?
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    /// `true`, если значение — `true`. Отсутствие поля и любые другие значения — `false`.
    pub fn is_true(&self) -> bool {
        matches!(self, Self::Bool(true))
    }
}

/// Ошибка разбора.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum JsParseError {
    /// В файле нет `export const <имя> =`.
    #[error("в файле не найдено «export const {0} =»")]
    ExportNotFound(String),
    /// Файл закончился посреди значения.
    #[error("неожиданный конец файла")]
    UnexpectedEnd,
    /// Встретился символ, которого не должно быть в этом месте.
    #[error("неожиданный символ «{character}» в строке {line}")]
    UnexpectedCharacter {
        /// Символ.
        character: char,
        /// Номер строки (с 1).
        line: usize,
    },
    /// Неправильная запись числа или escape-последовательности.
    #[error("неправильное значение «{text}» в строке {line}")]
    InvalidLiteral {
        /// Текст значения.
        text: String,
        /// Номер строки (с 1).
        line: usize,
    },
    /// Слишком глубокая вложенность.
    #[error("вложенность больше {MAX_NESTING_DEPTH} уровней в строке {line}")]
    TooDeep {
        /// Номер строки (с 1).
        line: usize,
    },
}

/// Находит в исходном тексте `export const <name> = ...` и разбирает значение.
pub fn parse_export(source: &str, name: &str) -> Result<JsValue, JsParseError> {
    let marker = format!("export const {name} =");
    let start = source
        .find(&marker)
        .ok_or_else(|| JsParseError::ExportNotFound(name.to_owned()))?;
    let line = source[..start].lines().count().max(1);
    let mut parser = Parser::new(&source[start + marker.len()..], line);
    parser.parse_value()
}

/// Разборщик: идёт по символам текста слева направо (рекурсивный спуск).
struct Parser {
    characters: Vec<char>,
    position: usize,
    line: usize,
    depth: usize,
}

impl Parser {
    /// Создаёт разборщик для текста, который начинается на строке `line`.
    fn new(text: &str, line: usize) -> Self {
        Self {
            characters: text.chars().collect(),
            position: 0,
            line,
            depth: 0,
        }
    }

    /// Текущий символ (без сдвига).
    fn peek(&self) -> Option<char> {
        self.characters.get(self.position).copied()
    }

    /// Символ через один от текущего.
    fn peek_next(&self) -> Option<char> {
        self.characters.get(self.position + 1).copied()
    }

    /// Возвращает текущий символ и сдвигается на следующий, считая строки.
    fn advance(&mut self) -> Result<char, JsParseError> {
        let character = self.peek().ok_or(JsParseError::UnexpectedEnd)?;
        self.position += 1;
        if character == '\n' {
            self.line += 1;
        }
        Ok(character)
    }

    /// Ошибка «неожиданный символ» для символа `character`.
    fn unexpected(&self, character: char) -> JsParseError {
        JsParseError::UnexpectedCharacter {
            character,
            line: self.line,
        }
    }

    /// Пропускает пробелы, переводы строк и комментарии `//` и `/* */`.
    fn skip_insignificant(&mut self) {
        while let Some(character) = self.peek() {
            match (character, self.peek_next()) {
                (c, _) if c.is_whitespace() => self.position_skip(1),
                ('/', Some('/')) => self.skip_while(|c| c != '\n'),
                ('/', Some('*')) => self.skip_block_comment(),
                _ => return,
            }
        }
    }

    /// Сдвигается на `count` символов (с подсчётом строк).
    fn position_skip(&mut self, count: usize) {
        for _ in 0..count {
            let _ = self.advance();
        }
    }

    /// Пропускает символы, пока выполняется условие.
    fn skip_while(&mut self, condition: impl Fn(char) -> bool) {
        while self.peek().is_some_and(&condition) {
            self.position_skip(1);
        }
    }

    /// Пропускает комментарий `/* ... */`.
    fn skip_block_comment(&mut self) {
        self.position_skip(2);
        while self.peek().is_some() && !(self.peek() == Some('*') && self.peek_next() == Some('/'))
        {
            self.position_skip(1);
        }
        self.position_skip(2);
    }

    /// Проверяет, что следующий значимый символ — `expected`, и пропускает его.
    fn expect(&mut self, expected: char) -> Result<(), JsParseError> {
        self.skip_insignificant();
        match self.advance()? {
            character if character == expected => Ok(()),
            other => Err(self.unexpected(other)),
        }
    }

    /// Разбирает любое значение: выбирает способ по первому символу.
    fn parse_value(&mut self) -> Result<JsValue, JsParseError> {
        self.skip_insignificant();
        match self.peek().ok_or(JsParseError::UnexpectedEnd)? {
            '{' => self.nested(Self::parse_object),
            '[' => self.nested(Self::parse_array),
            '\'' | '"' | '`' => self.parse_string().map(JsValue::String),
            '-' | '0'..='9' => self.parse_number(),
            c if is_identifier_start(c) => Ok(self.parse_identifier_value()),
            other => Err(self.unexpected(other)),
        }
    }

    /// Выполняет разбор вложенного значения, следя за глубиной вложенности.
    fn nested(
        &mut self,
        parse: fn(&mut Self) -> Result<JsValue, JsParseError>,
    ) -> Result<JsValue, JsParseError> {
        if self.depth >= MAX_NESTING_DEPTH {
            return Err(JsParseError::TooDeep { line: self.line });
        }
        self.depth += 1;
        let result = parse(self);
        self.depth -= 1;
        result
    }

    /// Разбирает объект `{ ключ: значение, ... }` (допускается запятая в конце).
    fn parse_object(&mut self) -> Result<JsValue, JsParseError> {
        self.expect('{')?;
        let mut fields = Vec::new();
        loop {
            self.skip_insignificant();
            if self.peek() == Some('}') {
                self.position_skip(1);
                return Ok(JsValue::Object(fields));
            }
            let key = self.parse_key()?;
            self.expect(':')?;
            fields.push((key, self.parse_value()?));
            if !self.consume_separator('}')? {
                return Ok(JsValue::Object(fields));
            }
        }
    }

    /// Разбирает массив `[ значение, ... ]` (допускается запятая в конце).
    fn parse_array(&mut self) -> Result<JsValue, JsParseError> {
        self.expect('[')?;
        let mut items = Vec::new();
        loop {
            self.skip_insignificant();
            if self.peek() == Some(']') {
                self.position_skip(1);
                return Ok(JsValue::Array(items));
            }
            items.push(self.parse_value()?);
            if !self.consume_separator(']')? {
                return Ok(JsValue::Array(items));
            }
        }
    }

    /// После элемента ожидается `,` (вернуть `true` — читать дальше)
    /// или закрывающая скобка `closing` (вернуть `false` — конец).
    fn consume_separator(&mut self, closing: char) -> Result<bool, JsParseError> {
        self.skip_insignificant();
        match self.advance()? {
            ',' => Ok(true),
            character if character == closing => Ok(false),
            other => Err(self.unexpected(other)),
        }
    }

    /// Разбирает ключ объекта: строку в кавычках или имя без кавычек.
    fn parse_key(&mut self) -> Result<String, JsParseError> {
        match self.peek().ok_or(JsParseError::UnexpectedEnd)? {
            '\'' | '"' | '`' => self.parse_string(),
            c if is_identifier_part(c) => Ok(self.read_identifier()),
            other => Err(self.unexpected(other)),
        }
    }

    /// Разбирает строку в кавычках с escape-последовательностями.
    fn parse_string(&mut self) -> Result<String, JsParseError> {
        let quote = self.advance()?;
        let mut text = String::new();
        loop {
            match self.advance()? {
                character if character == quote => return Ok(text),
                '\\' => text.push(self.parse_escape()?),
                '$' if quote == '`' && self.peek() == Some('{') => {
                    // Подстановка `${...}` — это уже код, а не данные.
                    return Err(self.unexpected('$'));
                }
                character => text.push(character),
            }
        }
    }

    /// Разбирает символ после `\` внутри строки.
    fn parse_escape(&mut self) -> Result<char, JsParseError> {
        Ok(match self.advance()? {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            'u' => self.parse_unicode_escape()?,
            other => other, // \' \" \\ \` и прочие — сам символ
        })
    }

    /// Разбирает `\uXXXX` (4 шестнадцатеричные цифры).
    fn parse_unicode_escape(&mut self) -> Result<char, JsParseError> {
        let digits: String = (0..4).map(|_| self.advance()).collect::<Result<_, _>>()?;
        u32::from_str_radix(&digits, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or_else(|| JsParseError::InvalidLiteral {
                text: format!("\\u{digits}"),
                line: self.line,
            })
    }

    /// Разбирает число, например `-5`, `46`, `0.75`, `1e3`.
    fn parse_number(&mut self) -> Result<JsValue, JsParseError> {
        let start = self.position;
        self.skip_while(|c| c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E'));
        let text: String = self.characters[start..self.position].iter().collect();
        text.parse::<f64>()
            .map(JsValue::Number)
            .map_err(|_| JsParseError::InvalidLiteral {
                text,
                line: self.line,
            })
    }

    /// Разбирает `true`, `false`, `null`, `undefined` или ссылку `a.b.c`.
    fn parse_identifier_value(&mut self) -> JsValue {
        match self.read_identifier().as_str() {
            "true" => JsValue::Bool(true),
            "false" => JsValue::Bool(false),
            "null" | "undefined" => JsValue::Null,
            reference => JsValue::Reference(reference.to_owned()),
        }
    }

    /// Читает имя: буквы, цифры, `_`, `$` и точки (для `weapons.claymore`).
    fn read_identifier(&mut self) -> String {
        let start = self.position;
        self.skip_while(|c| is_identifier_part(c) || c == '.');
        self.characters[start..self.position].iter().collect()
    }
}

/// Может ли символ начинать имя в JavaScript.
fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || matches!(character, '_' | '$')
}

/// Может ли символ входить в имя в JavaScript.
fn is_identifier_part(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '$')
}

// Тесты лежат рядом, в файле `js_literal.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "js_literal.tests.rs"]
mod tests;
