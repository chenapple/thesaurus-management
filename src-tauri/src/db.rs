use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

static DB: OnceCell<Mutex<Connection>> = OnceCell::new();

// 多语言停用词表（英语、德语、法语、意大利语、西班牙语）
fn get_stopwords() -> HashSet<&'static str> {
    [
        // 英语 English
        "a", "an", "the", "and", "or", "but", "if", "then", "else", "for", "of", "to", "in", "on",
        "at", "by", "with", "from", "as", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could", "should", "may",
        "might", "must", "shall", "can", "need", "into", "through", "during", "before", "after",
        "above", "below", "between", "under", "again", "further", "once", "here", "there",
        "when", "where", "why", "how", "all", "each", "every", "both", "few", "more", "most",
        "other", "some", "such", "no", "nor", "not", "only", "own", "same", "so", "than", "too",
        "very", "just", "also", "now", "new", "used", "one", "two", "first", "way", "even",
        "because", "any", "these", "those", "its", "it", "this", "that", "what", "which", "who",
        "whom", "while", "about", "against", "over", "out", "up", "down", "off", "your", "our",

        // 德语 German
        "der", "die", "das", "ein", "eine", "einer", "einem", "einen", "und", "oder", "aber",
        "wenn", "dann", "für", "von", "zu", "in", "an", "auf", "mit", "aus", "ist", "sind",
        "war", "waren", "sein", "haben", "hat", "hatte", "werden", "wird", "wurde", "können",
        "kann", "konnte", "müssen", "muss", "musste", "sollen", "soll", "sollte", "wollen",
        "will", "wollte", "dürfen", "darf", "durfte", "nicht", "auch", "nur", "noch", "schon",
        "immer", "wieder", "hier", "dort", "wo", "wann", "wie", "warum", "was", "wer", "wen",
        "wem", "welche", "welcher", "welches", "dieser", "diese", "dieses", "jener", "jene",
        "jenes", "alle", "alles", "andere", "anderer", "anderes", "mehr", "viel", "viele",
        "wenig", "wenige", "einige", "manche", "jeder", "jede", "jedes", "kein", "keine",
        "durch", "über", "unter", "zwischen", "vor", "nach", "bei", "seit", "bis", "ohne",
        "gegen", "um", "per", "pro",

        // 法语 French
        "le", "la", "les", "un", "une", "des", "du", "de", "et", "ou", "mais", "si", "que",
        "qui", "quoi", "dont", "où", "pour", "par", "sur", "sous", "dans", "en", "avec",
        "sans", "chez", "vers", "entre", "contre", "avant", "après", "pendant", "depuis",
        "est", "sont", "était", "étaient", "être", "avoir", "ai", "as", "avons", "avez",
        "ont", "fait", "faire", "peut", "peuvent", "pouvoir", "doit", "doivent", "devoir",
        "veut", "veulent", "vouloir", "sait", "savent", "savoir", "ne", "pas", "plus",
        "moins", "très", "bien", "mal", "peu", "beaucoup", "trop", "assez", "aussi",
        "encore", "toujours", "jamais", "souvent", "parfois", "ici", "là", "ce", "cette",
        "ces", "cet", "mon", "ma", "mes", "ton", "ta", "tes", "son", "sa", "ses", "notre",
        "nos", "votre", "vos", "leur", "leurs", "tout", "tous", "toute", "toutes", "quel",
        "quelle", "quels", "quelles", "chaque", "autre", "autres", "même", "mêmes",

        // 意大利语 Italian
        "il", "lo", "la", "i", "gli", "le", "un", "uno", "una", "di", "a", "da", "in", "con",
        "su", "per", "tra", "fra", "e", "ed", "o", "ma", "se", "che", "chi", "cui", "dove",
        "come", "quando", "perché", "quale", "quali", "quanto", "quanta", "quanti", "quante",
        "questo", "questa", "questi", "queste", "quello", "quella", "quelli", "quelle",
        "è", "sono", "era", "erano", "essere", "avere", "ho", "hai", "ha", "abbiamo",
        "avete", "hanno", "fare", "può", "possono", "potere", "deve", "devono", "dovere",
        "vuole", "vogliono", "volere", "non", "più", "meno", "molto", "molti", "molte",
        "poco", "pochi", "poche", "troppo", "tanto", "tanti", "tante", "tutto", "tutti",
        "tutte", "ogni", "altro", "altri", "altre", "stesso", "stessi", "stesse", "proprio",
        "anche", "ancora", "sempre", "mai", "già", "ora", "poi", "qui", "là", "dove",
        "solo", "soltanto", "circa", "quasi", "appena", "proprio", "verso", "durante",
        "dopo", "prima", "sopra", "sotto", "dentro", "fuori", "oltre", "attraverso",

        // 西班牙语 Spanish
        "el", "la", "los", "las", "un", "una", "unos", "unas", "de", "del", "al", "a",
        "en", "con", "por", "para", "sin", "sobre", "bajo", "entre", "hacia", "desde",
        "hasta", "según", "durante", "mediante", "y", "e", "o", "u", "pero", "sino",
        "que", "quien", "quienes", "cual", "cuales", "cuyo", "cuya", "cuyos", "cuyas",
        "donde", "cuando", "como", "porque", "qué", "quién", "cuál", "dónde", "cuándo",
        "cómo", "es", "son", "está", "están", "era", "eran", "fue", "fueron", "ser",
        "estar", "haber", "he", "has", "ha", "hemos", "habéis", "han", "tener", "tiene",
        "tienen", "hacer", "hace", "hacen", "poder", "puede", "pueden", "deber", "debe",
        "deben", "querer", "quiere", "quieren", "saber", "sabe", "saben", "no", "sí",
        "más", "menos", "muy", "mucho", "mucha", "muchos", "muchas", "poco", "poca",
        "pocos", "pocas", "tanto", "tanta", "tantos", "tantas", "todo", "toda", "todos",
        "todas", "otro", "otra", "otros", "otras", "mismo", "misma", "mismos", "mismas",
        "cada", "algún", "alguna", "algunos", "algunas", "ningún", "ninguna", "este",
        "esta", "estos", "estas", "ese", "esa", "esos", "esas", "aquel", "aquella",
        "aquellos", "aquellas", "también", "tampoco", "además", "todavía", "ya",
        "ahora", "antes", "después", "siempre", "nunca", "aquí", "allí", "así",
    ]
    .into_iter()
    .collect()
}

// 检查是否为有效词根（非停用词、非纯数字、长度>=2）
fn is_valid_root(word: &str) -> bool {
    let stopwords = get_stopwords();
    let word_lower = word.to_lowercase();

    // 长度至少2个字符
    if word.len() < 2 {
        return false;
    }

    // 不是纯数字
    if word.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // 不在停用词表中
    !stopwords.contains(word_lower.as_str())
}

// 产品结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub country: Option<String>,            // 国家代码: US, UK, DE, FR, IT, ES
    pub cpc_header: Option<String>,         // Excel中CPC列的表头（包含货币符号）
    pub bid_range_header: Option<String>,   // Excel中竞价范围列的表头（包含货币符号）
    pub big_word_threshold: Option<i64>,    // 大词阈值（默认20000）
    pub medium_word_threshold: Option<i64>, // 中词阈值（默认100000）
}

// 流量级别统计
#[derive(Debug, Serialize, Deserialize)]
pub struct TrafficLevelStats {
    pub big_count: i64,
    pub medium_count: i64,
    pub small_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub name_en: Option<String>,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RootWithCategories {
    pub id: i64,
    pub word: String,
    pub translation: Option<String>,
    pub contains_count: i64,
    pub percentage: f64,
    pub categories: Vec<i64>,
}

// 关键词完整数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeywordData {
    pub id: i64,
    pub product_id: i64,
    // 原始Excel列 (A-P)
    pub keyword: String,                        // A: 关键词
    pub translation: Option<String>,            // B: 翻译
    pub relevance_score: Option<String>,        // C: 相关性得分
    pub relevance_level: Option<String>,        // D: 相关性档位
    pub traffic_total: Option<f64>,             // E: 流量总和
    pub avg_keyword_rank: Option<String>,       // F: 周平均关键词排名
    pub avg_search_volume: Option<f64>,         // G: 周平均搜索量
    pub cpc_bid: Option<String>,                // H: CPC建议竞价(元)
    pub bid_range: Option<String>,              // I: 建议竞价范围(元)
    pub click_rate: Option<String>,             // J: 点击转化率/周
    pub conversion_competition: Option<String>, // K: 周转化竞争
    pub competition_level: Option<String>,      // L: 竞争度档位
    pub natural_position_flow: Option<String>,  // M: 自然位流动率%
    pub top3_click_share: Option<String>,       // N: Top3周平均点击份额
    pub avg_conversion_share: Option<String>,   // O: 周平均转化份额
    pub asin_count: Option<i64>,                // P: asin数量
    // 新增计算列
    pub traffic_level: Option<String>,          // 流量级别 (大词/中词/小词)
    pub negative_word: Option<String>,          // 否词
    pub orderliness: Option<String>,            // 有序性
    pub phrase_tag: Option<String>,             // 词组标签
    pub primary_category: Option<String>,       // 一级分类
    pub secondary_category: Option<String>,     // 二级分类
    pub search_intent: Option<String>,          // 搜索意图
    pub traffic_share: Option<f64>,             // 流量占比
    // ASIN动态列（JSON格式存储）
    pub asin_data: Option<String>,
}

pub fn init_db(app_data_dir: PathBuf) -> Result<()> {
    std::fs::create_dir_all(&app_data_dir).ok();
    let db_path = app_data_dir.join("thesaurus.db");
    let conn = Connection::open(db_path)?;

    // 显式禁用外键约束（解决 Windows 上的兼容性问题）
    conn.execute("PRAGMA foreign_keys = OFF", [])?;

    // 先创建产品表（不依赖其他表）
    conn.execute_batch(
        "
        -- 产品表
        CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sku TEXT,
            asin TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- 分类表
        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            name_en TEXT,
            parent_id INTEGER,
            FOREIGN KEY (parent_id) REFERENCES categories(id)
        );

        -- 词根-分类关联表
        CREATE TABLE IF NOT EXISTS root_categories (
            root_id INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            PRIMARY KEY (root_id, category_id),
            FOREIGN KEY (root_id) REFERENCES roots(id) ON DELETE CASCADE,
            FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
        );

        -- 搜索词-词根关联表
        CREATE TABLE IF NOT EXISTS keyword_roots (
            keyword_id INTEGER NOT NULL,
            root_id INTEGER NOT NULL,
            PRIMARY KEY (keyword_id, root_id),
            FOREIGN KEY (keyword_id) REFERENCES keywords(id) ON DELETE CASCADE,
            FOREIGN KEY (root_id) REFERENCES roots(id) ON DELETE CASCADE
        );
        ",
    )?;

    // 数据库迁移：为旧数据添加产品支持（在创建新表结构之前）
    migrate_add_product_support(&conn)?;

    // 迁移产品表：添加表头字段
    migrate_product_headers(&conn)?;

    // 迁移产品表：添加阈值字段
    migrate_product_thresholds(&conn)?;

    // 迁移产品表：添加国家字段
    migrate_product_country(&conn)?;

    // 迁移 keyword_data 表：检查是否需要重建表（列名变更）
    migrate_keyword_data_table(&conn)?;

    // 创建 keyword_data 表（存储完整Excel数据）
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS keyword_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_id INTEGER NOT NULL,
            -- 原始Excel列 (A-P)
            keyword TEXT NOT NULL,
            translation TEXT,
            relevance_score TEXT,
            relevance_level TEXT,
            traffic_total REAL,
            avg_keyword_rank TEXT,
            avg_search_volume REAL,
            cpc_bid TEXT,
            bid_range TEXT,
            click_rate TEXT,
            conversion_competition TEXT,
            competition_level TEXT,
            natural_position_flow TEXT,
            top3_click_share TEXT,
            avg_conversion_share TEXT,
            asin_count INTEGER,
            -- 新增计算列
            traffic_level TEXT,
            negative_word TEXT,
            orderliness TEXT,
            phrase_tag TEXT,
            primary_category TEXT,
            secondary_category TEXT,
            search_intent TEXT,
            traffic_share REAL,
            -- ASIN动态列
            asin_data TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(keyword, product_id),
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
        );
        ",
    )?;

    // 创建索引（迁移完成后）
    conn.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_roots_word ON roots(word);
        CREATE INDEX IF NOT EXISTS idx_roots_product ON roots(product_id);
        CREATE INDEX IF NOT EXISTS idx_keywords_keyword ON keywords(keyword);
        CREATE INDEX IF NOT EXISTS idx_keywords_product ON keywords(product_id);
        CREATE INDEX IF NOT EXISTS idx_keyword_data_product ON keyword_data(product_id);
        CREATE INDEX IF NOT EXISTS idx_keyword_data_keyword ON keyword_data(keyword);
        ",
    )?;

    // 创建备份表
    conn.execute_batch(
        "
        -- 备份元数据表
        CREATE TABLE IF NOT EXISTS backups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_id INTEGER NOT NULL,
            backup_name TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            keyword_data_count INTEGER DEFAULT 0,
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
        );

        -- 备份 keyword_data 表
        CREATE TABLE IF NOT EXISTS backup_keyword_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            backup_id INTEGER NOT NULL,
            product_id INTEGER NOT NULL,
            keyword TEXT NOT NULL,
            translation TEXT,
            relevance_score TEXT,
            relevance_level TEXT,
            traffic_total REAL,
            avg_keyword_rank TEXT,
            avg_search_volume REAL,
            cpc_bid TEXT,
            bid_range TEXT,
            click_rate TEXT,
            conversion_competition TEXT,
            competition_level TEXT,
            natural_position_flow TEXT,
            top3_click_share TEXT,
            avg_conversion_share TEXT,
            asin_count INTEGER,
            traffic_level TEXT,
            negative_word TEXT,
            orderliness TEXT,
            phrase_tag TEXT,
            primary_category TEXT,
            secondary_category TEXT,
            search_intent TEXT,
            traffic_share REAL,
            asin_data TEXT,
            FOREIGN KEY (backup_id) REFERENCES backups(id) ON DELETE CASCADE
        );

        -- 备份表索引
        CREATE INDEX IF NOT EXISTS idx_backups_product ON backups(product_id);
        CREATE INDEX IF NOT EXISTS idx_backup_keyword_data_backup ON backup_keyword_data(backup_id);

        -- 设置表（存储API Key等配置）
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        ",
    )?;

    // 初始化分类数据
    init_categories(&conn)?;

    // 初始化关键词监控表
    init_monitoring_tables(&conn)?;

    // 迁移优化事件表：添加 event_sub_type 列
    migrate_events_add_sub_type(&conn)?;
    migrate_events_add_asin(&conn)?;

    // 迁移关键词监控表：添加 tags 列
    migrate_keyword_monitoring_tags(&conn)?;

    // 初始化知识库表
    init_knowledge_base_tables(&conn)?;

    // 初始化智能文案表
    init_smart_copy_tables(&conn)?;

    // 初始化市场调研监控表
    init_market_research_tables(&conn)?;

    // 初始化竞品情报监控表
    init_competitor_tables(&conn)?;

    // 初始化快捷备忘录表
    init_quick_notes_table(&conn)?;

    // 初始化汇率缓存表
    init_exchange_rate_table(&conn)?;

    DB.set(Mutex::new(conn))
        .map_err(|_| rusqlite::Error::InvalidQuery)?;

    Ok(())
}

// 数据库迁移：为产品表添加表头字段
fn migrate_product_headers(conn: &Connection) -> Result<()> {
    // 检查 products 表是否存在 cpc_header 列
    let has_cpc_header: bool = conn
        .prepare("SELECT cpc_header FROM products LIMIT 1")
        .is_ok();

    if !has_cpc_header {
        // 添加新列
        conn.execute("ALTER TABLE products ADD COLUMN cpc_header TEXT", [])?;
        conn.execute("ALTER TABLE products ADD COLUMN bid_range_header TEXT", [])?;
    }

    Ok(())
}

// 数据库迁移：为产品表添加阈值字段
fn migrate_product_thresholds(conn: &Connection) -> Result<()> {
    // 检查 products 表是否存在 big_word_threshold 列
    let has_threshold: bool = conn
        .prepare("SELECT big_word_threshold FROM products LIMIT 1")
        .is_ok();

    if !has_threshold {
        // 添加阈值列
        conn.execute("ALTER TABLE products ADD COLUMN big_word_threshold INTEGER", [])?;
        conn.execute("ALTER TABLE products ADD COLUMN medium_word_threshold INTEGER", [])?;
    }

    Ok(())
}

// 数据库迁移：为产品表添加国家字段
fn migrate_product_country(conn: &Connection) -> Result<()> {
    // 检查 products 表是否存在 country 列
    let has_country: bool = conn
        .prepare("SELECT country FROM products LIMIT 1")
        .is_ok();

    if !has_country {
        // 添加国家列
        conn.execute("ALTER TABLE products ADD COLUMN country TEXT", [])?;
    }

    Ok(())
}

// 数据库迁移：为优化事件表添加 event_sub_type 列
fn migrate_events_add_sub_type(conn: &Connection) -> Result<()> {
    // 检查表是否存在
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='optimization_events'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) > 0;

    if !table_exists {
        return Ok(());
    }

    // 检查是否已有 event_sub_type 列
    let has_sub_type: bool = conn
        .prepare("SELECT event_sub_type FROM optimization_events LIMIT 1")
        .is_ok();

    if !has_sub_type {
        // 添加 event_sub_type 列
        conn.execute("ALTER TABLE optimization_events ADD COLUMN event_sub_type TEXT", [])?;
        // 迁移旧数据：根据 event_type 设置默认 sub_type
        conn.execute("UPDATE optimization_events SET event_sub_type = 'title' WHERE event_type = 'listing' OR event_type IS NULL", [])?;
        conn.execute("UPDATE optimization_events SET event_sub_type = 'bid', event_type = 'ad' WHERE event_type IN ('ad_keyword', 'ad_bid', 'ad_budget')", [])?;
        conn.execute("UPDATE optimization_events SET event_sub_type = 'title', event_type = 'listing' WHERE event_type = 'other'", [])?;
    }

    Ok(())
}

// 数据库迁移：为 optimization_events 表添加 target_asin 列
fn migrate_events_add_asin(conn: &Connection) -> Result<()> {
    // 检查表是否存在
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='optimization_events'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) > 0;

    if !table_exists {
        return Ok(());
    }

    // 检查是否已有 target_asin 列
    let has_target_asin: bool = conn
        .prepare("SELECT target_asin FROM optimization_events LIMIT 1")
        .is_ok();

    if !has_target_asin {
        // 添加 target_asin 列
        conn.execute("ALTER TABLE optimization_events ADD COLUMN target_asin TEXT", [])?;
    }

    Ok(())
}

// 数据库迁移：为关键词监控表添加 tags 列
fn migrate_keyword_monitoring_tags(conn: &Connection) -> Result<()> {
    // 检查 keyword_monitoring 表是否存在 tags 列
    let has_tags: bool = conn
        .prepare("SELECT tags FROM keyword_monitoring LIMIT 1")
        .is_ok();

    if !has_tags {
        // 添加 tags 列
        conn.execute("ALTER TABLE keyword_monitoring ADD COLUMN tags TEXT", [])?;
    }

    Ok(())
}

// 数据库迁移：检查并重建 keyword_data 表（列名变更）
fn migrate_keyword_data_table(conn: &Connection) -> Result<()> {
    // 检查 keyword_data 表是否存在
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='keyword_data'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);

    if !table_exists {
        return Ok(()); // 表不存在，稍后会创建
    }

    // 检查是否有新列 relevance_level（如果没有说明是旧表结构）
    let has_new_column: bool = conn
        .prepare("SELECT relevance_level FROM keyword_data LIMIT 1")
        .is_ok();

    if !has_new_column {
        // 旧表结构，需要删除并重建（因为列名变更太多，直接重建更简单）
        // 注意：这会清空现有数据，但由于这是新功能，影响应该很小
        conn.execute("DROP TABLE IF EXISTS keyword_data", [])?;
    }

    Ok(())
}

// 数据库迁移：检查并添加产品支持
fn migrate_add_product_support(conn: &Connection) -> Result<()> {
    // 检查keywords表是否存在
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='keywords'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);

    if !table_exists {
        // 表不存在，创建新表结构
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS keywords (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                keyword TEXT NOT NULL,
                product_id INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(keyword, product_id),
                FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS roots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                word TEXT NOT NULL,
                product_id INTEGER NOT NULL,
                translation TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(word, product_id),
                FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
            );
            ",
        )?;
        return Ok(());
    }

    // 检查keywords表是否有product_id列
    let has_product_id: bool = conn
        .prepare("SELECT product_id FROM keywords LIMIT 1")
        .is_ok();

    if !has_product_id {
        // 旧表结构，需要迁移
        // 1. 创建默认产品
        conn.execute(
            "INSERT OR IGNORE INTO products (id, name, sku, asin) VALUES (1, '默认产品', NULL, NULL)",
            [],
        )?;

        // 2. 重命名旧表
        conn.execute_batch(
            "
            ALTER TABLE keywords RENAME TO keywords_old;
            ALTER TABLE roots RENAME TO roots_old;
            ALTER TABLE keyword_roots RENAME TO keyword_roots_old;
            ",
        )?;

        // 3. 创建新表
        conn.execute_batch(
            "
            CREATE TABLE keywords (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                keyword TEXT NOT NULL,
                product_id INTEGER NOT NULL DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(keyword, product_id),
                FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
            );

            CREATE TABLE roots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                word TEXT NOT NULL,
                product_id INTEGER NOT NULL DEFAULT 1,
                translation TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(word, product_id),
                FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
            );
            ",
        )?;

        // 4. 迁移数据
        conn.execute_batch(
            "
            INSERT INTO keywords (id, keyword, product_id, created_at)
            SELECT id, keyword, 1, created_at FROM keywords_old;

            INSERT INTO roots (id, word, product_id, translation, created_at)
            SELECT id, word, 1, translation, created_at FROM roots_old;

            INSERT INTO keyword_roots (keyword_id, root_id)
            SELECT keyword_id, root_id FROM keyword_roots_old;
            ",
        )?;

        // 5. 删除旧表
        conn.execute_batch(
            "
            DROP TABLE keyword_roots_old;
            DROP TABLE keywords_old;
            DROP TABLE roots_old;
            ",
        )?;
    }

    Ok(())
}

fn init_categories(conn: &Connection) -> Result<()> {
    // 检查是否已经初始化
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }

    // 一级分类
    let primary_categories = [
        ("品类词", "category"),
        ("品牌", "brand"),
        ("颜色", "color"),
        ("形状", "shape"),
        ("功能", "function"),
    ];

    // 二级分类
    let secondary_categories = [
        ("适用人群", "target_audience"),
        ("材质", "material"),
        ("尺寸", "size"),
        ("使用场景", "scenario"),
        ("情绪价值", "emotion"),
        ("使用地点", "location"),
        ("节假日", "holiday"),
        ("适配", "compatibility"),
        ("其他", "other"),
    ];

    for (name, name_en) in primary_categories {
        conn.execute(
            "INSERT INTO categories (name, name_en, parent_id) VALUES (?1, ?2, NULL)",
            [name, name_en],
        )?;
    }

    for (name, name_en) in secondary_categories {
        conn.execute(
            "INSERT INTO categories (name, name_en, parent_id) VALUES (?1, ?2, NULL)",
            [name, name_en],
        )?;
    }

    Ok(())
}

pub fn get_db() -> &'static Mutex<Connection> {
    DB.get().expect("Database not initialized")
}

// 获取所有分类
pub fn get_categories() -> Result<Vec<Category>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare("SELECT id, name, name_en, parent_id FROM categories ORDER BY id")?;
    let categories = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                name_en: row.get(2)?,
                parent_id: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(categories)
}

// ==================== 产品管理 ====================

// 获取所有产品
pub fn get_products() -> Result<Vec<Product>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare("SELECT id, name, country, cpc_header, bid_range_header, big_word_threshold, medium_word_threshold FROM products ORDER BY id")?;
    let products = stmt
        .query_map([], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                country: row.get(2)?,
                cpc_header: row.get(3)?,
                bid_range_header: row.get(4)?,
                big_word_threshold: row.get(5)?,
                medium_word_threshold: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(products)
}

// 创建产品
pub fn create_product(name: String, country: Option<String>) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO products (name, country) VALUES (?1, ?2)",
        rusqlite::params![name, country],
    )?;
    Ok(conn.last_insert_rowid())
}

// 更新产品
pub fn update_product(id: i64, name: String, country: Option<String>) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE products SET name = ?1, country = ?2 WHERE id = ?3",
        rusqlite::params![name, country, id],
    )?;
    Ok(())
}

// 更新产品的Excel表头信息
pub fn update_product_headers(id: i64, cpc_header: Option<String>, bid_range_header: Option<String>) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE products SET cpc_header = ?1, bid_range_header = ?2 WHERE id = ?3",
        rusqlite::params![cpc_header, bid_range_header, id],
    )?;
    Ok(())
}

// 删除产品（同时删除关联的关键词和词根）
pub fn delete_product(id: i64) -> Result<()> {
    let conn = get_db().lock();

    // 显式禁用外键约束（解决 Windows 兼容性问题）
    // 注意：禁用外键后级联删除不会自动工作，需要手动删除关联数据
    conn.execute("PRAGMA foreign_keys = OFF", [])?;

    // 手动删除关联数据（因为外键被禁用，CASCADE 不会工作）
    conn.execute("DELETE FROM keyword_roots WHERE root_id IN (SELECT id FROM roots WHERE product_id = ?1)", [id])?;
    conn.execute("DELETE FROM root_categories WHERE root_id IN (SELECT id FROM roots WHERE product_id = ?1)", [id])?;
    conn.execute("DELETE FROM roots WHERE product_id = ?1", [id])?;
    conn.execute("DELETE FROM keywords WHERE product_id = ?1", [id])?;
    conn.execute("DELETE FROM products WHERE id = ?1", [id])?;

    Ok(())
}

// ==================== 关键词和词根 ====================

// 导入关键词并分析词根（关联到指定产品）
pub fn import_keywords(product_id: i64, keywords: Vec<String>) -> Result<()> {
    let conn = get_db().lock();

    // 显式禁用外键约束（解决 Windows 兼容性问题）
    conn.execute("PRAGMA foreign_keys = OFF", [])?;

    // 使用事务大幅提升导入速度（特别是在 Windows 上）
    conn.execute("BEGIN TRANSACTION", [])?;

    let result = (|| {
        for keyword in keywords {
            let keyword = keyword.trim().to_lowercase();
            if keyword.is_empty() {
                continue;
            }

            // 插入关键词（忽略重复）
            conn.execute(
                "INSERT OR IGNORE INTO keywords (keyword, product_id) VALUES (?1, ?2)",
                rusqlite::params![&keyword, product_id],
            )?;

            // 获取关键词ID
            let keyword_id: i64 = conn.query_row(
                "SELECT id FROM keywords WHERE keyword = ?1 AND product_id = ?2",
                rusqlite::params![&keyword, product_id],
                |row| row.get(0),
            )?;

            // 分词并插入词根
            let words: Vec<&str> = keyword.split_whitespace().collect();
            for word in words {
                let word = word.trim();
                if word.is_empty() {
                    continue;
                }

                // 过滤停用词（多语言：英语、德语、法语、意大利语、西班牙语）
                if !is_valid_root(word) {
                    continue;
                }

                // 插入词根（忽略重复，按产品独立）
                conn.execute(
                    "INSERT OR IGNORE INTO roots (word, product_id) VALUES (?1, ?2)",
                    rusqlite::params![word, product_id],
                )?;

                // 获取词根ID
                let root_id: i64 = conn.query_row(
                    "SELECT id FROM roots WHERE word = ?1 AND product_id = ?2",
                    rusqlite::params![word, product_id],
                    |row| row.get(0),
                )?;

                // 建立关联（忽略重复）
                conn.execute(
                    "INSERT OR IGNORE INTO keyword_roots (keyword_id, root_id) VALUES (?1, ?2)",
                    [keyword_id, root_id],
                )?;
            }
        }
        Ok::<(), rusqlite::Error>(())
    })();

    match result {
        Ok(_) => {
            conn.execute("COMMIT", [])?;
            Ok(())
        }
        Err(e) => {
            conn.execute("ROLLBACK", []).ok();
            Err(e)
        }
    }
}

// 获取词根列表（带统计信息，按产品筛选）
pub fn get_roots(
    product_id: Option<i64>,
    search: Option<String>,
    category_ids: Option<Vec<i64>>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<RootWithCategories>, i64)> {
    let conn = get_db().lock();

    // 获取该产品的总关键词数
    let total_keywords: i64 = if let Some(pid) = product_id {
        conn.query_row(
            "SELECT COUNT(*) FROM keywords WHERE product_id = ?1",
            [pid],
            |row| row.get(0),
        )?
    } else {
        conn.query_row("SELECT COUNT(*) FROM keywords", [], |row| row.get(0))?
    };

    // 构建查询
    let mut sql = String::from(
        "
        SELECT DISTINCT r.id, r.word, r.translation,
               (SELECT COUNT(*) FROM keyword_roots WHERE root_id = r.id) as contains_count
        FROM roots r
        ",
    );

    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    // 按产品筛选
    if let Some(pid) = product_id {
        conditions.push("r.product_id = ?".to_string());
        params.push(Box::new(pid));
    }

    if let Some(ref cat_ids) = category_ids {
        if !cat_ids.is_empty() {
            sql.push_str(" JOIN root_categories rc ON r.id = rc.root_id ");
            let placeholders: Vec<String> = cat_ids.iter().map(|_| "?".to_string()).collect();
            conditions.push(format!("rc.category_id IN ({})", placeholders.join(",")));
            for id in cat_ids {
                params.push(Box::new(*id));
            }
        }
    }

    if let Some(ref search_term) = search {
        if !search_term.is_empty() {
            conditions.push("r.word LIKE ?".to_string());
            params.push(Box::new(format!("%{}%", search_term)));
        }
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    // 排序
    let sort_column = match sort_by.as_deref() {
        Some("word") => "r.word",
        Some("translation") => "r.translation",
        Some("contains_count") => "contains_count",
        Some("percentage") => "contains_count",
        _ => "contains_count",
    };
    let order = match sort_order.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };
    sql.push_str(&format!(" ORDER BY {} {}", sort_column, order));

    // 分页
    sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, (page - 1) * page_size));

    // 执行查询
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let roots = stmt
        .query_map(params_refs.as_slice(), |row| {
            let contains_count: i64 = row.get(3)?;
            let percentage = if total_keywords > 0 {
                (contains_count as f64 / total_keywords as f64) * 100.0
            } else {
                0.0
            };
            Ok(RootWithCategories {
                id: row.get(0)?,
                word: row.get(1)?,
                translation: row.get(2)?,
                contains_count,
                percentage,
                categories: Vec::new(),
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    // 获取每个词根的分类
    let mut roots_with_categories = roots;
    for root in &mut roots_with_categories {
        let mut cat_stmt = conn.prepare(
            "SELECT category_id FROM root_categories WHERE root_id = ?1",
        )?;
        let cat_ids = cat_stmt
            .query_map([root.id], |row| row.get(0))?
            .collect::<Result<Vec<i64>>>()?;
        root.categories = cat_ids;
    }

    // 获取总数（按产品筛选）
    let total: i64 = if let Some(pid) = product_id {
        conn.query_row(
            "SELECT COUNT(DISTINCT id) FROM roots WHERE product_id = ?1",
            [pid],
            |row| row.get(0),
        )?
    } else {
        conn.query_row("SELECT COUNT(DISTINCT id) FROM roots", [], |row| row.get(0))?
    };

    Ok((roots_with_categories, total))
}

// 更新词根翻译
pub fn update_root_translation(id: i64, translation: String) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE roots SET translation = ?1 WHERE id = ?2",
        rusqlite::params![translation, id],
    )?;
    Ok(())
}

// 为词根添加分类
pub fn add_root_category(root_id: i64, category_id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT OR IGNORE INTO root_categories (root_id, category_id) VALUES (?1, ?2)",
        [root_id, category_id],
    )?;
    Ok(())
}

// 移除词根分类
pub fn remove_root_category(root_id: i64, category_id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "DELETE FROM root_categories WHERE root_id = ?1 AND category_id = ?2",
        [root_id, category_id],
    )?;
    Ok(())
}

// 获取统计信息（按产品筛选）
pub fn get_stats(product_id: Option<i64>) -> Result<(i64, i64)> {
    let conn = get_db().lock();
    let (keyword_count, root_count) = if let Some(pid) = product_id {
        let kw: i64 = conn.query_row(
            "SELECT COUNT(*) FROM keywords WHERE product_id = ?1",
            [pid],
            |row| row.get(0),
        )?;
        let rt: i64 = conn.query_row(
            "SELECT COUNT(*) FROM roots WHERE product_id = ?1",
            [pid],
            |row| row.get(0),
        )?;
        (kw, rt)
    } else {
        let kw: i64 = conn.query_row("SELECT COUNT(*) FROM keywords", [], |row| row.get(0))?;
        let rt: i64 = conn.query_row("SELECT COUNT(*) FROM roots", [], |row| row.get(0))?;
        (kw, rt)
    };
    Ok((keyword_count, root_count))
}

// 获取各分类的词根数量（按产品筛选）
pub fn get_category_counts(product_id: i64) -> Result<Vec<(i64, i64)>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT c.id, COUNT(DISTINCT r.id) as count
         FROM categories c
         LEFT JOIN root_categories rc ON c.id = rc.category_id
         LEFT JOIN roots r ON rc.root_id = r.id AND r.product_id = ?1
         GROUP BY c.id
         ORDER BY c.id",
    )?;
    let counts = stmt
        .query_map([product_id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<(i64, i64)>>>()?;
    Ok(counts)
}

// 清空产品数据（只删除指定产品的关键词和词根）
pub fn clear_product_data(product_id: i64) -> Result<()> {
    let conn = get_db().lock();

    // 显式禁用外键约束（解决 Windows 兼容性问题）
    conn.execute("PRAGMA foreign_keys = OFF", [])?;

    // 先删除关联表数据
    conn.execute(
        "DELETE FROM keyword_roots WHERE keyword_id IN (SELECT id FROM keywords WHERE product_id = ?1)",
        [product_id],
    )?;
    conn.execute(
        "DELETE FROM root_categories WHERE root_id IN (SELECT id FROM roots WHERE product_id = ?1)",
        [product_id],
    )?;
    // 再删除关键词和词根
    conn.execute("DELETE FROM keywords WHERE product_id = ?1", [product_id])?;
    conn.execute("DELETE FROM roots WHERE product_id = ?1", [product_id])?;
    // 删除关键词完整数据
    conn.execute("DELETE FROM keyword_data WHERE product_id = ?1", [product_id])?;
    Ok(())
}

// 获取未翻译的词根（按产品筛选）
pub fn get_untranslated_roots(product_id: i64) -> Result<Vec<String>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT word FROM roots WHERE product_id = ?1 AND (translation IS NULL OR translation = '') ORDER BY id",
    )?;
    let words = stmt
        .query_map([product_id], |row| row.get(0))?
        .collect::<Result<Vec<String>>>()?;
    Ok(words)
}

// 批量更新词根翻译和分类（按产品筛选，使用事务提升性能）
pub fn batch_update_root_analysis(
    product_id: i64,
    updates: Vec<(String, String, Vec<String>)>, // (word, translation, category_names)
) -> Result<()> {
    let conn = get_db().lock();

    // 显式禁用外键约束（解决 Windows 兼容性问题）
    conn.execute("PRAGMA foreign_keys = OFF", [])?;

    // 使用事务提升批量更新性能
    conn.execute("BEGIN TRANSACTION", [])?;

    let result = (|| {
        for (word, translation, category_names) in updates {
            // 使用 LOWER() 进行大小写不敏感匹配（DeepSeek 可能改变大小写）
            let word_lower = word.to_lowercase();

            // 先查询词根ID（大小写不敏感）
            let root_id: Option<i64> = conn
                .query_row(
                    "SELECT id FROM roots WHERE LOWER(word) = ?1 AND product_id = ?2",
                    rusqlite::params![&word_lower, product_id],
                    |row| row.get(0),
                )
                .ok();

            // 如果找不到词根，跳过
            let Some(root_id) = root_id else {
                continue;
            };

            // 更新翻译（使用ID更可靠）
            conn.execute(
                "UPDATE roots SET translation = ?1 WHERE id = ?2",
                rusqlite::params![&translation, root_id],
            )?;

            // 清除现有分类
            conn.execute("DELETE FROM root_categories WHERE root_id = ?1", [root_id])?;

            // 添加新分类（只添加存在的分类）
            for cat_name in category_names {
                // 查询分类ID
                let cat_id: Option<i64> = conn
                    .query_row(
                        "SELECT id FROM categories WHERE name = ?1",
                        [&cat_name],
                        |row| row.get(0),
                    )
                    .ok();

                // 只有分类存在时才插入（避免外键约束错误）
                if let Some(cat_id) = cat_id {
                    // 再次确认 root 存在（防止并发问题）
                    let root_exists: bool = conn
                        .query_row("SELECT 1 FROM roots WHERE id = ?1", [root_id], |_| Ok(true))
                        .unwrap_or(false);

                    let cat_exists: bool = conn
                        .query_row("SELECT 1 FROM categories WHERE id = ?1", [cat_id], |_| Ok(true))
                        .unwrap_or(false);

                    if root_exists && cat_exists {
                        conn.execute(
                            "INSERT OR IGNORE INTO root_categories (root_id, category_id) VALUES (?1, ?2)",
                            [root_id, cat_id],
                        )?;
                    }
                }
            }
        }
        Ok::<(), rusqlite::Error>(())
    })();

    match result {
        Ok(_) => {
            conn.execute("COMMIT", [])?;
            Ok(())
        }
        Err(e) => {
            conn.execute("ROLLBACK", []).ok();
            Err(e)
        }
    }
}

// ==================== 关键词完整数据管理 ====================

// 导入关键词完整数据
pub fn import_keyword_data(product_id: i64, data_list: Vec<KeywordData>) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("PRAGMA foreign_keys = OFF", [])?;
    conn.execute("BEGIN TRANSACTION", [])?;

    let result = (|| {
        for data in data_list {
            conn.execute(
                "INSERT OR REPLACE INTO keyword_data (
                    product_id, keyword, translation, relevance_score, relevance_level,
                    traffic_total, avg_keyword_rank, avg_search_volume, cpc_bid, bid_range,
                    click_rate, conversion_competition, competition_level, natural_position_flow,
                    top3_click_share, avg_conversion_share, asin_count, traffic_level, negative_word, orderliness,
                    phrase_tag, primary_category, secondary_category, search_intent, traffic_share, asin_data
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26)",
                rusqlite::params![
                    product_id,
                    data.keyword,
                    data.translation,
                    data.relevance_score,
                    data.relevance_level,
                    data.traffic_total,
                    data.avg_keyword_rank,
                    data.avg_search_volume,
                    data.cpc_bid,
                    data.bid_range,
                    data.click_rate,
                    data.conversion_competition,
                    data.competition_level,
                    data.natural_position_flow,
                    data.top3_click_share,
                    data.avg_conversion_share,
                    data.asin_count,
                    data.traffic_level,
                    data.negative_word,
                    data.orderliness,
                    data.phrase_tag,
                    data.primary_category,
                    data.secondary_category,
                    data.search_intent,
                    data.traffic_share,
                    data.asin_data,
                ],
            )?;
        }
        Ok::<(), rusqlite::Error>(())
    })();

    match result {
        Ok(_) => {
            conn.execute("COMMIT", [])?;
            Ok(())
        }
        Err(e) => {
            conn.execute("ROLLBACK", []).ok();
            Err(e)
        }
    }
}

// 获取关键词数据（分页）
pub fn get_keyword_data(
    product_id: i64,
    search: Option<String>,
    traffic_levels: Option<Vec<String>>,
    relevance_levels: Option<Vec<String>>,
    primary_categories: Option<Vec<String>>,
    orderliness_values: Option<Vec<String>>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<KeywordData>, i64)> {
    let conn = get_db().lock();

    let mut sql = String::from(
        "SELECT id, product_id, keyword, translation, relevance_score, relevance_level,
                traffic_total, avg_keyword_rank, avg_search_volume, cpc_bid, bid_range,
                click_rate, conversion_competition, competition_level, natural_position_flow,
                top3_click_share, avg_conversion_share, asin_count, traffic_level, negative_word, orderliness,
                phrase_tag, primary_category, secondary_category, search_intent, traffic_share, asin_data
         FROM keyword_data WHERE product_id = ?1",
    );

    let mut count_sql = String::from("SELECT COUNT(*) FROM keyword_data WHERE product_id = ?1");

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(product_id)];
    let mut param_index = 2;

    // 搜索条件
    if let Some(ref search_term) = search {
        if !search_term.is_empty() {
            let condition = format!(" AND keyword LIKE ?{}", param_index);
            sql.push_str(&condition);
            count_sql.push_str(&condition);
            params.push(Box::new(format!("%{}%", search_term)));
            param_index += 1;
        }
    }

    // 流量级别筛选
    if let Some(ref levels) = traffic_levels {
        if !levels.is_empty() {
            let placeholders: Vec<String> = levels.iter().enumerate()
                .map(|(i, _)| format!("?{}", param_index + i))
                .collect();
            let condition = format!(" AND traffic_level IN ({})", placeholders.join(","));
            sql.push_str(&condition);
            count_sql.push_str(&condition);
            for level in levels {
                params.push(Box::new(level.clone()));
            }
            param_index += levels.len();
        }
    }

    // 相关性档位筛选
    if let Some(ref levels) = relevance_levels {
        if !levels.is_empty() {
            let placeholders: Vec<String> = levels.iter().enumerate()
                .map(|(i, _)| format!("?{}", param_index + i))
                .collect();
            let condition = format!(" AND relevance_level IN ({})", placeholders.join(","));
            sql.push_str(&condition);
            count_sql.push_str(&condition);
            for level in levels {
                params.push(Box::new(level.clone()));
            }
            param_index += levels.len();
        }
    }

    // 一级分类筛选
    if let Some(ref categories) = primary_categories {
        if !categories.is_empty() {
            let placeholders: Vec<String> = categories.iter().enumerate()
                .map(|(i, _)| format!("?{}", param_index + i))
                .collect();
            let condition = format!(" AND primary_category IN ({})", placeholders.join(","));
            sql.push_str(&condition);
            count_sql.push_str(&condition);
            for cat in categories {
                params.push(Box::new(cat.clone()));
            }
            param_index += categories.len();
        }
    }

    // 有序性筛选
    if let Some(ref values) = orderliness_values {
        if !values.is_empty() {
            let placeholders: Vec<String> = values.iter().enumerate()
                .map(|(i, _)| format!("?{}", param_index + i))
                .collect();
            let condition = format!(" AND orderliness IN ({})", placeholders.join(","));
            sql.push_str(&condition);
            count_sql.push_str(&condition);
            for val in values {
                params.push(Box::new(val.clone()));
            }
        }
    }

    // 排序 - 空值始终排在最后
    let (sort_expr, null_check) = match sort_by.as_deref() {
        Some("keyword") => ("keyword", "keyword IS NULL OR keyword = ''"),
        Some("traffic_total") => ("traffic_total", "traffic_total IS NULL"),
        Some("avg_keyword_rank") => ("CAST(avg_keyword_rank AS REAL)", "avg_keyword_rank IS NULL OR avg_keyword_rank = ''"),
        Some("avg_search_volume") => ("avg_search_volume", "avg_search_volume IS NULL"),
        Some("traffic_level") => ("traffic_level", "traffic_level IS NULL OR traffic_level = ''"),
        Some("asin_count") => ("asin_count", "asin_count IS NULL"),
        _ => ("id", ""),
    };
    let order = match sort_order.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };
    if null_check.is_empty() {
        sql.push_str(&format!(" ORDER BY {} {}", sort_expr, order));
    } else {
        // 空值始终排在最后：先按是否为空排序，再按实际值排序
        sql.push_str(&format!(" ORDER BY CASE WHEN {} THEN 1 ELSE 0 END, {} {}", null_check, sort_expr, order));
    }

    // 分页
    sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, (page - 1) * page_size));

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let data = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(KeywordData {
                id: row.get(0)?,
                product_id: row.get(1)?,
                keyword: row.get(2)?,
                translation: row.get(3)?,
                relevance_score: row.get(4)?,
                relevance_level: row.get(5)?,
                traffic_total: row.get(6)?,
                avg_keyword_rank: row.get(7)?,
                avg_search_volume: row.get(8)?,
                cpc_bid: row.get(9)?,
                bid_range: row.get(10)?,
                click_rate: row.get(11)?,
                conversion_competition: row.get(12)?,
                competition_level: row.get(13)?,
                natural_position_flow: row.get(14)?,
                top3_click_share: row.get(15)?,
                avg_conversion_share: row.get(16)?,
                asin_count: row.get(17)?,
                traffic_level: row.get(18)?,
                negative_word: row.get(19)?,
                orderliness: row.get(20)?,
                phrase_tag: row.get(21)?,
                primary_category: row.get(22)?,
                secondary_category: row.get(23)?,
                search_intent: row.get(24)?,
                traffic_share: row.get(25)?,
                asin_data: row.get(26)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    // 获取总数（使用相同的筛选条件）
    let count_params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let total: i64 = conn.query_row(
        &count_sql,
        count_params_refs.as_slice(),
        |row| row.get(0),
    )?;

    Ok((data, total))
}

// 更新关键词数据的单个字段
pub fn update_keyword_field(id: i64, field: &str, value: &str) -> Result<()> {
    let conn = get_db().lock();

    // 只允许更新特定字段（安全考虑）
    let allowed_fields = [
        "traffic_level", "negative_word", "orderliness", "phrase_tag",
        "primary_category", "secondary_category", "search_intent", "traffic_share"
    ];

    if !allowed_fields.contains(&field) {
        return Err(rusqlite::Error::InvalidParameterName(format!("Field '{}' is not allowed", field)));
    }

    let sql = format!("UPDATE keyword_data SET {} = ?1 WHERE id = ?2", field);
    conn.execute(&sql, rusqlite::params![value, id])?;
    Ok(())
}

// 清空产品的关键词数据
pub fn clear_keyword_data(product_id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("DELETE FROM keyword_data WHERE product_id = ?1", [product_id])?;
    Ok(())
}

// 获取关键词数据统计
pub fn get_keyword_data_stats(product_id: i64) -> Result<i64> {
    let conn = get_db().lock();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM keyword_data WHERE product_id = ?1",
        [product_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

// ==================== 流量级别管理 ====================

// 更新产品阈值
pub fn update_product_thresholds(id: i64, big_word_threshold: i64, medium_word_threshold: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE products SET big_word_threshold = ?1, medium_word_threshold = ?2 WHERE id = ?3",
        rusqlite::params![big_word_threshold, medium_word_threshold, id],
    )?;
    Ok(())
}

// 根据阈值计算并更新流量级别
pub fn calculate_traffic_levels(product_id: i64, big_threshold: i64, medium_threshold: i64) -> Result<()> {
    let conn = get_db().lock();

    // 使用 CASE WHEN 批量更新流量级别（基于周平均排名）
    // 大词: avg_keyword_rank <= big_threshold
    // 中词: big_threshold < avg_keyword_rank <= medium_threshold
    // 小词: avg_keyword_rank > medium_threshold
    conn.execute(
        "UPDATE keyword_data SET traffic_level = CASE
            WHEN avg_keyword_rank IS NULL OR avg_keyword_rank = '' THEN NULL
            WHEN CAST(avg_keyword_rank AS REAL) <= ?1 THEN '大词'
            WHEN CAST(avg_keyword_rank AS REAL) <= ?2 THEN '中词'
            ELSE '小词'
        END
        WHERE product_id = ?3",
        rusqlite::params![big_threshold, medium_threshold, product_id],
    )?;

    Ok(())
}

// 获取流量级别统计
pub fn get_traffic_level_stats(product_id: i64) -> Result<TrafficLevelStats> {
    let conn = get_db().lock();

    let big_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM keyword_data WHERE product_id = ?1 AND traffic_level = '大词'",
        [product_id],
        |row| row.get(0),
    ).unwrap_or(0);

    let medium_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM keyword_data WHERE product_id = ?1 AND traffic_level = '中词'",
        [product_id],
        |row| row.get(0),
    ).unwrap_or(0);

    let small_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM keyword_data WHERE product_id = ?1 AND traffic_level = '小词'",
        [product_id],
        |row| row.get(0),
    ).unwrap_or(0);

    Ok(TrafficLevelStats {
        big_count,
        medium_count,
        small_count,
    })
}

// 智能推荐阈值：让大词数量接近目标数量（默认20）
pub fn recommend_threshold(product_id: i64, target_big_count: i64) -> Result<i64> {
    let conn = get_db().lock();

    // 获取所有非空排名值，按升序排列（基于周平均排名）
    let mut stmt = conn.prepare(
        "SELECT CAST(avg_keyword_rank AS REAL) FROM keyword_data
         WHERE product_id = ?1 AND avg_keyword_rank IS NOT NULL AND avg_keyword_rank != ''
         ORDER BY CAST(avg_keyword_rank AS REAL) ASC"
    )?;

    let ranks: Vec<i64> = stmt
        .query_map([product_id], |row| row.get::<_, f64>(0))?
        .filter_map(|r| r.ok())
        .map(|v| v as i64)
        .collect();

    if ranks.is_empty() {
        return Ok(20000); // 默认值
    }

    // 取第 target_big_count 个值作为大词阈值（索引从0开始，所以用 target_big_count - 1）
    let index = std::cmp::min(target_big_count as usize, ranks.len()) - 1;
    let threshold = ranks.get(index).copied().unwrap_or(20000);

    // 向上取整到千位，使阈值更整齐
    let rounded = ((threshold + 999) / 1000) * 1000;

    Ok(rounded)
}

// ==================== 流量占比计算 ====================

// 计算并更新流量占比
pub fn calculate_traffic_share(product_id: i64) -> Result<()> {
    let conn = get_db().lock();

    // 1. 计算该产品所有关键词的流量总和
    let total_traffic: f64 = conn.query_row(
        "SELECT COALESCE(SUM(traffic_total), 0) FROM keyword_data WHERE product_id = ?1",
        [product_id],
        |row| row.get(0),
    )?;

    if total_traffic <= 0.0 {
        // 没有流量数据，清空流量占比
        conn.execute(
            "UPDATE keyword_data SET traffic_share = NULL WHERE product_id = ?1",
            [product_id],
        )?;
        return Ok(());
    }

    // 2. 批量更新每行的流量占比 = (traffic_total / total_traffic) * 100
    conn.execute(
        "UPDATE keyword_data SET traffic_share =
            CASE
                WHEN traffic_total IS NOT NULL THEN ROUND((traffic_total / ?1) * 100, 2)
                ELSE NULL
            END
         WHERE product_id = ?2",
        rusqlite::params![total_traffic, product_id],
    )?;

    Ok(())
}

// ==================== 关键词分类管理 ====================

// 未分类关键词结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct UncategorizedKeyword {
    pub id: i64,
    pub keyword: String,
    pub translation: Option<String>,
}

// 获取未分类的关键词（primary_category为空）
pub fn get_uncategorized_keywords(product_id: i64) -> Result<Vec<UncategorizedKeyword>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, keyword, translation FROM keyword_data
         WHERE product_id = ?1 AND (primary_category IS NULL OR primary_category = '')
         ORDER BY id",
    )?;
    let keywords = stmt
        .query_map([product_id], |row| {
            Ok(UncategorizedKeyword {
                id: row.get(0)?,
                keyword: row.get(1)?,
                translation: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(keywords)
}

// 批量更新关键词分类（使用事务提升性能）
pub fn batch_update_keyword_categories(
    product_id: i64,
    updates: Vec<(String, String, String, String)>, // (keyword, primary_category, secondary_category, search_intent)
) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("BEGIN TRANSACTION", [])?;

    let result = (|| {
        for (keyword, primary_category, secondary_category, search_intent) in updates {
            conn.execute(
                "UPDATE keyword_data SET primary_category = ?1, secondary_category = ?2, search_intent = ?3
                 WHERE keyword = ?4 AND product_id = ?5",
                rusqlite::params![primary_category, secondary_category, search_intent, keyword, product_id],
            )?;
        }
        Ok::<(), rusqlite::Error>(())
    })();

    match result {
        Ok(_) => {
            conn.execute("COMMIT", [])?;
            Ok(())
        }
        Err(e) => {
            conn.execute("ROLLBACK", []).ok();
            Err(e)
        }
    }
}

// 将单词转换为单数形式（根据语言）
fn to_singular(word: &str, country: &str) -> String {
    let word_lower = word.to_lowercase();

    // 单词太短不处理
    if word_lower.len() < 3 {
        return word.to_string();
    }

    match country {
        // 英语: US, UK, CA, AU
        "US" | "UK" | "CA" | "AU" => {
            if word_lower.ends_with("es") && word_lower.len() > 3 {
                // boxes -> box, brushes -> brush
                word[..word.len() - 2].to_string()
            } else if word_lower.ends_with("s") {
                // files -> file
                word[..word.len() - 1].to_string()
            } else {
                word.to_string()
            }
        }
        // 德语: DE
        "DE" => {
            if word_lower.ends_with("en") && word_lower.len() > 3 {
                word[..word.len() - 2].to_string()
            } else if word_lower.ends_with("e") && word_lower.len() > 2 {
                word[..word.len() - 1].to_string()
            } else if word_lower.ends_with("s") {
                word[..word.len() - 1].to_string()
            } else {
                word.to_string()
            }
        }
        // 法语: FR
        "FR" => {
            if word_lower.ends_with("x") {
                word[..word.len() - 1].to_string()
            } else if word_lower.ends_with("s") {
                word[..word.len() - 1].to_string()
            } else {
                word.to_string()
            }
        }
        // 西班牙语: ES, MX
        "ES" | "MX" => {
            if word_lower.ends_with("es") && word_lower.len() > 3 {
                word[..word.len() - 2].to_string()
            } else if word_lower.ends_with("s") {
                word[..word.len() - 1].to_string()
            } else {
                word.to_string()
            }
        }
        // 意大利语或其他语言不处理
        _ => word.to_string(),
    }
}

// 将词组中的每个单词转换为单数形式
fn phrase_to_singular(phrase: &str, country: &str) -> String {
    phrase
        .split_whitespace()
        .map(|word| to_singular(word, country))
        .collect::<Vec<_>>()
        .join(" ")
}

// 词组打标：自动为关键词打上匹配的词组标签
pub fn calculate_phrase_tags(product_id: i64) -> Result<()> {
    let conn = get_db().lock();

    // 获取产品的国家代码
    let country: Option<String> = conn.query_row(
        "SELECT country FROM products WHERE id = ?1",
        [product_id],
        |row| row.get(0),
    ).unwrap_or(None);
    let country_code = country.as_deref().unwrap_or("US");

    // 1. 查询候选词组：(大词 OR 中词) AND (强相关 OR 高相关) AND 单词数 ≤ 5
    let mut stmt = conn.prepare(
        "SELECT keyword FROM keyword_data
         WHERE product_id = ?1
           AND traffic_level IN ('大词', '中词')
           AND relevance_level IN ('强相关', '高相关')
           AND (LENGTH(keyword) - LENGTH(REPLACE(keyword, ' ', '')) + 1) <= 5
         ORDER BY (LENGTH(keyword) - LENGTH(REPLACE(keyword, ' ', '')) + 1) DESC",
    )?;

    let candidates: Vec<String> = stmt
        .query_map([product_id], |row| row.get(0))?
        .collect::<Result<Vec<_>>>()?;

    // 2. 使用事务批量更新
    conn.execute("BEGIN TRANSACTION", [])?;

    let result = (|| {
        for candidate in &candidates {
            // 将候选词组转换为单数形式用于匹配
            let singular_candidate = phrase_to_singular(candidate, country_code);
            // 使用 LIKE 匹配包含该词组的关键词，且 phrase_tag 为空
            let pattern = format!("%{}%", singular_candidate);
            conn.execute(
                "UPDATE keyword_data
                 SET phrase_tag = ?1
                 WHERE product_id = ?2
                   AND keyword LIKE ?3
                   AND (phrase_tag IS NULL OR phrase_tag = '')",
                rusqlite::params![singular_candidate, product_id, pattern],
            )?;
        }
        Ok::<(), rusqlite::Error>(())
    })();

    match result {
        Ok(_) => {
            conn.execute("COMMIT", [])?;
            Ok(())
        }
        Err(e) => {
            conn.execute("ROLLBACK", []).ok();
            Err(e)
        }
    }
}

// 计算有序性：根据 phrase_tag 出现次数判断
pub fn calculate_orderliness(product_id: i64) -> Result<()> {
    let conn = get_db().lock();

    // 使用子查询统计每个 phrase_tag 的出现次数，然后更新 orderliness
    // - phrase_tag 为空 → orderliness = NULL
    // - phrase_tag 出现次数 < 4 → orderliness = '无序'
    // - phrase_tag 出现次数 >= 4 → orderliness = '有序'
    conn.execute(
        "UPDATE keyword_data
         SET orderliness =
           CASE
             WHEN phrase_tag IS NULL OR phrase_tag = '' THEN NULL
             WHEN (SELECT COUNT(*) FROM keyword_data k2
                   WHERE k2.product_id = keyword_data.product_id
                   AND k2.phrase_tag = keyword_data.phrase_tag) >= 4
             THEN '有序'
             ELSE '无序'
           END
         WHERE product_id = ?1",
        [product_id],
    )?;

    Ok(())
}

// ==================== 流程状态 ====================

// 流程状态结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub has_data: bool,           // 是否有关键词数据
    pub has_traffic_level: bool,  // 是否有流量级别
    pub has_category: bool,       // 是否有AI分类
    pub has_phrase_tag: bool,     // 是否有词组打标
    pub has_orderliness: bool,    // 是否有有序性
}

// 获取流程状态
pub fn get_workflow_status(product_id: i64) -> Result<WorkflowStatus> {
    let conn = get_db().lock();

    // 检查是否有数据
    let has_data: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM keyword_data WHERE product_id = ?1)",
        [product_id],
        |row| row.get(0),
    )?;

    // 检查是否有流量级别
    let has_traffic_level: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM keyword_data WHERE product_id = ?1 AND traffic_level IS NOT NULL AND traffic_level != '')",
        [product_id],
        |row| row.get(0),
    )?;

    // 检查是否有AI分类
    let has_category: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM keyword_data WHERE product_id = ?1 AND primary_category IS NOT NULL AND primary_category != '')",
        [product_id],
        |row| row.get(0),
    )?;

    // 检查是否有词组打标
    let has_phrase_tag: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM keyword_data WHERE product_id = ?1 AND phrase_tag IS NOT NULL AND phrase_tag != '')",
        [product_id],
        |row| row.get(0),
    )?;

    // 检查是否有有序性
    let has_orderliness: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM keyword_data WHERE product_id = ?1 AND orderliness IS NOT NULL AND orderliness != '')",
        [product_id],
        |row| row.get(0),
    )?;

    Ok(WorkflowStatus {
        has_data,
        has_traffic_level,
        has_category,
        has_phrase_tag,
        has_orderliness,
    })
}

// ==================== 备份功能 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupInfo {
    pub id: i64,
    pub product_id: i64,
    pub backup_name: Option<String>,
    pub created_at: String,
    pub keyword_data_count: i64,
}

// 创建备份
pub fn create_backup(product_id: i64, backup_name: Option<String>) -> Result<i64> {
    let conn = get_db().lock();

    // 开始事务
    conn.execute("BEGIN TRANSACTION", [])?;

    let result = (|| -> Result<i64> {
        // 1. 插入备份元数据
        conn.execute(
            "INSERT INTO backups (product_id, backup_name) VALUES (?1, ?2)",
            rusqlite::params![product_id, backup_name],
        )?;
        let backup_id = conn.last_insert_rowid();

        // 2. 复制 keyword_data 到 backup_keyword_data
        conn.execute(
            "INSERT INTO backup_keyword_data (
                backup_id, product_id, keyword, translation, relevance_score, relevance_level,
                traffic_total, avg_keyword_rank, avg_search_volume, cpc_bid, bid_range,
                click_rate, conversion_competition, competition_level, natural_position_flow,
                top3_click_share, avg_conversion_share, asin_count, traffic_level,
                negative_word, orderliness, phrase_tag, primary_category, secondary_category,
                search_intent, traffic_share, asin_data
            )
            SELECT
                ?1, product_id, keyword, translation, relevance_score, relevance_level,
                traffic_total, avg_keyword_rank, avg_search_volume, cpc_bid, bid_range,
                click_rate, conversion_competition, competition_level, natural_position_flow,
                top3_click_share, avg_conversion_share, asin_count, traffic_level,
                negative_word, orderliness, phrase_tag, primary_category, secondary_category,
                search_intent, traffic_share, asin_data
            FROM keyword_data WHERE product_id = ?2",
            rusqlite::params![backup_id, product_id],
        )?;

        // 3. 更新备份元数据中的数据量
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM backup_keyword_data WHERE backup_id = ?1",
            [backup_id],
            |row| row.get(0),
        )?;
        conn.execute(
            "UPDATE backups SET keyword_data_count = ?1 WHERE id = ?2",
            rusqlite::params![count, backup_id],
        )?;

        // 4. 清理旧备份（保留最近3个）
        cleanup_old_backups(&conn, product_id, 3)?;

        Ok(backup_id)
    })();

    match result {
        Ok(backup_id) => {
            conn.execute("COMMIT", [])?;
            Ok(backup_id)
        }
        Err(e) => {
            conn.execute("ROLLBACK", []).ok();
            Err(e)
        }
    }
}

// 清理旧备份（保留最近 max_backups 个）
fn cleanup_old_backups(conn: &Connection, product_id: i64, max_backups: i64) -> Result<()> {
    // 获取需要删除的备份ID
    let mut stmt = conn.prepare(
        "SELECT id FROM backups WHERE product_id = ?1 ORDER BY created_at DESC LIMIT -1 OFFSET ?2",
    )?;
    let backup_ids: Vec<i64> = stmt
        .query_map(rusqlite::params![product_id, max_backups], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    // 删除旧备份（CASCADE 会自动删除 backup_keyword_data 中的数据）
    for backup_id in backup_ids {
        conn.execute("DELETE FROM backups WHERE id = ?1", [backup_id])?;
    }

    Ok(())
}

// 获取产品的所有备份
pub fn get_backups(product_id: i64) -> Result<Vec<BackupInfo>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, product_id, backup_name, created_at, keyword_data_count
         FROM backups WHERE product_id = ?1 ORDER BY created_at DESC",
    )?;

    let backups = stmt
        .query_map([product_id], |row| {
            Ok(BackupInfo {
                id: row.get(0)?,
                product_id: row.get(1)?,
                backup_name: row.get(2)?,
                created_at: row.get(3)?,
                keyword_data_count: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(backups)
}

// 恢复备份
pub fn restore_backup(backup_id: i64) -> Result<()> {
    let conn = get_db().lock();

    // 开始事务
    conn.execute("BEGIN TRANSACTION", [])?;

    let result = (|| -> Result<()> {
        // 1. 获取备份对应的产品ID
        let product_id: i64 = conn.query_row(
            "SELECT product_id FROM backups WHERE id = ?1",
            [backup_id],
            |row| row.get(0),
        )?;

        // 2. 删除当前产品的 keyword_data
        conn.execute("DELETE FROM keyword_data WHERE product_id = ?1", [product_id])?;

        // 3. 从备份恢复数据
        conn.execute(
            "INSERT INTO keyword_data (
                product_id, keyword, translation, relevance_score, relevance_level,
                traffic_total, avg_keyword_rank, avg_search_volume, cpc_bid, bid_range,
                click_rate, conversion_competition, competition_level, natural_position_flow,
                top3_click_share, avg_conversion_share, asin_count, traffic_level,
                negative_word, orderliness, phrase_tag, primary_category, secondary_category,
                search_intent, traffic_share, asin_data
            )
            SELECT
                product_id, keyword, translation, relevance_score, relevance_level,
                traffic_total, avg_keyword_rank, avg_search_volume, cpc_bid, bid_range,
                click_rate, conversion_competition, competition_level, natural_position_flow,
                top3_click_share, avg_conversion_share, asin_count, traffic_level,
                negative_word, orderliness, phrase_tag, primary_category, secondary_category,
                search_intent, traffic_share, asin_data
            FROM backup_keyword_data WHERE backup_id = ?1",
            [backup_id],
        )?;

        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute("COMMIT", [])?;
            Ok(())
        }
        Err(e) => {
            conn.execute("ROLLBACK", []).ok();
            Err(e)
        }
    }
}

// 删除备份
pub fn delete_backup(backup_id: i64) -> Result<()> {
    let conn = get_db().lock();
    // CASCADE 会自动删除 backup_keyword_data 中的数据
    conn.execute("DELETE FROM backups WHERE id = ?1", [backup_id])?;
    Ok(())
}

// ==================== 设置管理 ====================

// 获取设置值
pub fn get_setting(key: &str) -> Result<Option<String>> {
    let conn = get_db().lock();
    let result = conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [key],
        |row| row.get(0),
    );
    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

// 设置值
pub fn set_setting(key: &str, value: &str) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

// 删除设置
pub fn delete_setting(key: &str) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("DELETE FROM settings WHERE key = ?1", [key])?;
    Ok(())
}

// ==================== 关键词排名监控 ====================

// 关键词监控结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeywordMonitoring {
    pub id: i64,
    pub product_id: i64,
    pub keyword: String,
    pub asin: String,
    pub country: String,           // US/UK/DE/FR/IT/ES
    pub priority: String,          // high/medium/low
    pub is_active: bool,

    // 最新排名
    pub latest_organic_rank: Option<i64>,
    pub latest_organic_page: Option<i64>,
    pub latest_sponsored_rank: Option<i64>,
    pub latest_sponsored_page: Option<i64>,

    // 产品信息
    pub image_url: Option<String>,
    pub price: Option<String>,
    pub reviews_count: Option<i64>,
    pub rating: Option<f64>,

    pub last_checked: Option<String>,
    pub created_at: String,
    pub tags: Option<String>,  // JSON array: ["high_traffic", "high_conversion"]
}

// 排名历史结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RankingHistory {
    pub id: i64,
    pub monitoring_id: i64,
    pub check_date: String,

    pub organic_rank: Option<i64>,
    pub organic_page: Option<i64>,
    pub sponsored_rank: Option<i64>,
    pub sponsored_page: Option<i64>,

    pub checked_at: String,
}

// 监控项的迷你图数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringSparkline {
    pub monitoring_id: i64,
    pub organic_ranks: Vec<Option<i64>>,
    pub sponsored_ranks: Vec<Option<i64>>,
}

// 竞品快照结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RankingSnapshot {
    pub id: i64,
    pub keyword: String,
    pub country: String,
    pub snapshot_date: String,

    pub organic_top_50: Option<String>,    // JSON: ["ASIN1", "ASIN2", ...]
    pub sponsored_top_20: Option<String>,  // JSON

    pub created_at: String,
}

// 监控统计结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringStats {
    pub total: i64,
    pub active: i64,
    pub top10_organic: i64,
    pub top30_organic: i64,
    pub with_sponsored: i64,
}

// 初始化关键词监控相关表
pub fn init_monitoring_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- 关键词监控表
        CREATE TABLE IF NOT EXISTS keyword_monitoring (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_id INTEGER NOT NULL,
            keyword TEXT NOT NULL,
            asin TEXT NOT NULL,
            country TEXT NOT NULL DEFAULT 'US',
            priority TEXT DEFAULT 'medium',
            is_active INTEGER DEFAULT 1,

            -- 最新排名
            latest_organic_rank INTEGER,
            latest_organic_page INTEGER,
            latest_sponsored_rank INTEGER,
            latest_sponsored_page INTEGER,

            -- 产品信息
            image_url TEXT,
            price TEXT,
            reviews_count INTEGER,
            rating REAL,

            last_checked TIMESTAMP,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(keyword, asin, country, product_id),
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
        );

        -- 排名历史表
        CREATE TABLE IF NOT EXISTS keyword_ranking_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            monitoring_id INTEGER NOT NULL,
            check_date DATE NOT NULL,

            organic_rank INTEGER,
            organic_page INTEGER,
            sponsored_rank INTEGER,
            sponsored_page INTEGER,

            checked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (monitoring_id) REFERENCES keyword_monitoring(id) ON DELETE CASCADE
        );

        -- 竞品快照表
        CREATE TABLE IF NOT EXISTS ranking_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            keyword TEXT NOT NULL,
            country TEXT NOT NULL,
            snapshot_date DATE NOT NULL,

            organic_top_50 TEXT,
            sponsored_top_20 TEXT,

            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(keyword, country, snapshot_date)
        );

        -- 定时任务记录表
        CREATE TABLE IF NOT EXISTS scheduler_task_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            started_at TIMESTAMP NOT NULL,
            ended_at TIMESTAMP,
            status TEXT NOT NULL DEFAULT 'running',  -- running, completed, failed
            total_keywords INTEGER DEFAULT 0,
            success_count INTEGER DEFAULT 0,
            failed_count INTEGER DEFAULT 0,
            trigger_type TEXT DEFAULT 'auto',  -- auto(定时), manual(手动)
            error_message TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 优化事件表
        CREATE TABLE IF NOT EXISTS optimization_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_id INTEGER NOT NULL,
            event_date TEXT NOT NULL,
            event_type TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            affected_keywords TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
        );

        -- 索引
        CREATE INDEX IF NOT EXISTS idx_keyword_monitoring_product ON keyword_monitoring(product_id);
        CREATE INDEX IF NOT EXISTS idx_keyword_monitoring_active ON keyword_monitoring(is_active);
        CREATE INDEX IF NOT EXISTS idx_keyword_monitoring_country ON keyword_monitoring(country);
        CREATE INDEX IF NOT EXISTS idx_ranking_history_monitoring ON keyword_ranking_history(monitoring_id);
        CREATE INDEX IF NOT EXISTS idx_ranking_history_date ON keyword_ranking_history(check_date);
        CREATE INDEX IF NOT EXISTS idx_ranking_snapshots_keyword ON ranking_snapshots(keyword, country);
        CREATE INDEX IF NOT EXISTS idx_scheduler_task_logs_started ON scheduler_task_logs(started_at);
        CREATE INDEX IF NOT EXISTS idx_events_product ON optimization_events(product_id);
        CREATE INDEX IF NOT EXISTS idx_events_date ON optimization_events(event_date);
        "
    )?;
    Ok(())
}

// 添加关键词监控
pub fn add_keyword_monitoring(
    product_id: i64,
    keyword: String,
    asin: String,
    country: String,
    priority: Option<String>,
) -> Result<i64> {
    let conn = get_db().lock();
    let priority = priority.unwrap_or_else(|| "medium".to_string());

    conn.execute(
        "INSERT OR IGNORE INTO keyword_monitoring (product_id, keyword, asin, country, priority)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![product_id, keyword, asin, country, priority],
    )?;

    // 获取插入或已存在的ID
    let id: i64 = conn.query_row(
        "SELECT id FROM keyword_monitoring WHERE keyword = ?1 AND asin = ?2 AND country = ?3 AND product_id = ?4",
        rusqlite::params![keyword, asin, country, product_id],
        |row| row.get(0),
    )?;

    Ok(id)
}

// 获取关键词监控列表
pub fn get_keyword_monitoring_list(
    product_id: i64,
    country: Option<String>,
    priority: Option<String>,
    is_active: Option<bool>,
    search: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<KeywordMonitoring>, i64)> {
    let conn = get_db().lock();

    let mut sql = String::from(
        "SELECT id, product_id, keyword, asin, country, priority, is_active,
                latest_organic_rank, latest_organic_page, latest_sponsored_rank, latest_sponsored_page,
                image_url, price, reviews_count, rating, last_checked, created_at, tags
         FROM keyword_monitoring WHERE product_id = ?1"
    );

    let mut count_sql = String::from("SELECT COUNT(*) FROM keyword_monitoring WHERE product_id = ?1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(product_id)];
    let mut param_index = 2;

    // 筛选条件
    if let Some(ref c) = country {
        let condition = format!(" AND country = ?{}", param_index);
        sql.push_str(&condition);
        count_sql.push_str(&condition);
        params.push(Box::new(c.clone()));
        param_index += 1;
    }

    if let Some(ref p) = priority {
        let condition = format!(" AND priority = ?{}", param_index);
        sql.push_str(&condition);
        count_sql.push_str(&condition);
        params.push(Box::new(p.clone()));
        param_index += 1;
    }

    if let Some(active) = is_active {
        let condition = format!(" AND is_active = ?{}", param_index);
        sql.push_str(&condition);
        count_sql.push_str(&condition);
        params.push(Box::new(if active { 1i64 } else { 0i64 }));
        param_index += 1;
    }

    if let Some(ref s) = search {
        if !s.is_empty() {
            let condition = format!(" AND (keyword LIKE ?{} OR asin LIKE ?{})", param_index, param_index);
            sql.push_str(&condition);
            count_sql.push_str(&condition);
            params.push(Box::new(format!("%{}%", s)));
            // param_index += 1;  // 同一个参数用两次
        }
    }

    // 排序
    let sort_column = match sort_by.as_deref() {
        Some("keyword") => "keyword",
        Some("latest_organic_rank") => "latest_organic_rank",
        Some("latest_sponsored_rank") => "latest_sponsored_rank",
        Some("last_checked") => "last_checked",
        Some("priority") => "priority",
        _ => "created_at",
    };
    let order = match sort_order.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };
    sql.push_str(&format!(" ORDER BY {} {}", sort_column, order));

    // 分页
    sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, (page - 1) * page_size));

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let data = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(KeywordMonitoring {
                id: row.get(0)?,
                product_id: row.get(1)?,
                keyword: row.get(2)?,
                asin: row.get(3)?,
                country: row.get(4)?,
                priority: row.get(5)?,
                is_active: row.get::<_, i64>(6)? == 1,
                latest_organic_rank: row.get(7)?,
                latest_organic_page: row.get(8)?,
                latest_sponsored_rank: row.get(9)?,
                latest_sponsored_page: row.get(10)?,
                image_url: row.get(11)?,
                price: row.get(12)?,
                reviews_count: row.get(13)?,
                rating: row.get(14)?,
                last_checked: row.get(15)?,
                created_at: row.get(16)?,
                tags: row.get(17)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    // 获取总数
    let count_params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let total: i64 = conn.query_row(&count_sql, count_params_refs.as_slice(), |row| row.get(0))?;

    Ok((data, total))
}

// 更新关键词监控
pub fn update_keyword_monitoring(
    id: i64,
    priority: Option<String>,
    is_active: Option<bool>,
) -> Result<()> {
    let conn = get_db().lock();

    if let Some(p) = priority {
        conn.execute(
            "UPDATE keyword_monitoring SET priority = ?1 WHERE id = ?2",
            rusqlite::params![p, id],
        )?;
    }

    if let Some(active) = is_active {
        conn.execute(
            "UPDATE keyword_monitoring SET is_active = ?1 WHERE id = ?2",
            rusqlite::params![if active { 1 } else { 0 }, id],
        )?;
    }

    Ok(())
}

// 删除关键词监控
pub fn delete_keyword_monitoring(id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("PRAGMA foreign_keys = OFF", [])?;
    // 先删除历史记录
    conn.execute("DELETE FROM keyword_ranking_history WHERE monitoring_id = ?1", [id])?;
    // 再删除监控记录
    conn.execute("DELETE FROM keyword_monitoring WHERE id = ?1", [id])?;
    Ok(())
}

// 批量删除关键词监控
pub fn batch_delete_keyword_monitoring(ids: Vec<i64>) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("PRAGMA foreign_keys = OFF", [])?;
    conn.execute("BEGIN TRANSACTION", [])?;

    let result = (|| {
        for id in ids {
            conn.execute("DELETE FROM keyword_ranking_history WHERE monitoring_id = ?1", [id])?;
            conn.execute("DELETE FROM keyword_monitoring WHERE id = ?1", [id])?;
        }
        Ok::<(), rusqlite::Error>(())
    })();

    match result {
        Ok(_) => {
            conn.execute("COMMIT", [])?;
            Ok(())
        }
        Err(e) => {
            conn.execute("ROLLBACK", []).ok();
            Err(e)
        }
    }
}

// 更新排名结果
pub fn update_ranking_result(
    monitoring_id: i64,
    organic_rank: Option<i64>,
    organic_page: Option<i64>,
    sponsored_rank: Option<i64>,
    sponsored_page: Option<i64>,
    image_url: Option<String>,
    price: Option<String>,
    reviews_count: Option<i64>,
    rating: Option<f64>,
) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("BEGIN TRANSACTION", [])?;

    let result = (|| {
        // 如果价格为空，尝试从同 ASIN+country 的其他记录中获取
        let final_price = if price.is_none() {
            // 先获取当前记录的 asin 和 country
            let asin_country: Option<(String, String)> = conn.query_row(
                "SELECT asin, country FROM keyword_monitoring WHERE id = ?1",
                [monitoring_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            ).ok();

            if let Some((asin, country)) = asin_country {
                // 从同 asin+country 的其他记录中查找最近有价格的
                conn.query_row(
                    "SELECT price FROM keyword_monitoring
                     WHERE asin = ?1 AND country = ?2 AND price IS NOT NULL AND price != ''
                     ORDER BY last_checked DESC LIMIT 1",
                    rusqlite::params![asin, country],
                    |row| row.get(0),
                ).ok()
            } else {
                None
            }
        } else {
            price
        };

        // 同样处理 image_url, reviews_count, rating
        let (final_image_url, final_reviews_count, final_rating) = if image_url.is_none() || reviews_count.is_none() || rating.is_none() {
            let asin_country: Option<(String, String)> = conn.query_row(
                "SELECT asin, country FROM keyword_monitoring WHERE id = ?1",
                [monitoring_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            ).ok();

            if let Some((asin, country)) = asin_country {
                let fallback: Option<(Option<String>, Option<i64>, Option<f64>)> = conn.query_row(
                    "SELECT image_url, reviews_count, rating FROM keyword_monitoring
                     WHERE asin = ?1 AND country = ?2 AND (image_url IS NOT NULL OR reviews_count IS NOT NULL OR rating IS NOT NULL)
                     ORDER BY last_checked DESC LIMIT 1",
                    rusqlite::params![asin, country],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                ).ok();

                if let Some((fb_img, fb_reviews, fb_rating)) = fallback {
                    (
                        image_url.or(fb_img),
                        reviews_count.or(fb_reviews),
                        rating.or(fb_rating),
                    )
                } else {
                    (image_url, reviews_count, rating)
                }
            } else {
                (image_url, reviews_count, rating)
            }
        } else {
            (image_url, reviews_count, rating)
        };

        // 更新最新排名
        conn.execute(
            "UPDATE keyword_monitoring SET
                latest_organic_rank = ?1, latest_organic_page = ?2,
                latest_sponsored_rank = ?3, latest_sponsored_page = ?4,
                image_url = ?5, price = ?6, reviews_count = ?7, rating = ?8,
                last_checked = datetime('now')
             WHERE id = ?9",
            rusqlite::params![
                organic_rank, organic_page,
                sponsored_rank, sponsored_page,
                final_image_url, final_price, final_reviews_count, final_rating,
                monitoring_id
            ],
        )?;

        // 插入历史记录（使用北京时间的日期）
        conn.execute(
            "INSERT INTO keyword_ranking_history
                (monitoring_id, check_date, organic_rank, organic_page, sponsored_rank, sponsored_page)
             VALUES (?1, date('now', '+8 hours'), ?2, ?3, ?4, ?5)",
            rusqlite::params![monitoring_id, organic_rank, organic_page, sponsored_rank, sponsored_page],
        )?;

        Ok::<(), rusqlite::Error>(())
    })();

    match result {
        Ok(_) => {
            conn.execute("COMMIT", [])?;
            Ok(())
        }
        Err(e) => {
            conn.execute("ROLLBACK", []).ok();
            Err(e)
        }
    }
}

// 获取排名历史（按日期聚合，每天只取最佳排名）
pub fn get_ranking_history(monitoring_id: i64, days: i64) -> Result<Vec<RankingHistory>> {
    let conn = get_db().lock();
    let days_str = format!("-{} days", days);

    // 按日期分组，取每天的最佳排名（最小值）
    // 页码取最小排名对应的页码
    let mut stmt = conn.prepare(
        "SELECT
            MIN(id) as id,
            monitoring_id,
            check_date,
            MIN(organic_rank) as organic_rank,
            MIN(organic_page) as organic_page,
            MIN(sponsored_rank) as sponsored_rank,
            MIN(sponsored_page) as sponsored_page,
            MIN(checked_at) as checked_at
         FROM keyword_ranking_history
         WHERE monitoring_id = ?1 AND check_date >= date('now', '+8 hours', ?2)
         GROUP BY check_date
         ORDER BY check_date ASC"
    )?;

    let history = stmt
        .query_map(rusqlite::params![monitoring_id, days_str], |row| {
            Ok(RankingHistory {
                id: row.get(0)?,
                monitoring_id: row.get(1)?,
                check_date: row.get(2)?,
                organic_rank: row.get(3)?,
                organic_page: row.get(4)?,
                sponsored_rank: row.get(5)?,
                sponsored_page: row.get(6)?,
                checked_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(history)
}

// 获取上一次的自然排名（用于计算排名变化）
pub fn get_previous_ranking(monitoring_id: i64) -> Result<Option<i64>> {
    let conn = get_db().lock();

    let result = conn.query_row(
        "SELECT organic_rank FROM keyword_ranking_history
         WHERE monitoring_id = ?1
         ORDER BY checked_at DESC
         LIMIT 1",
        [monitoring_id],
        |row| row.get(0),
    );

    match result {
        Ok(rank) => Ok(rank),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

// 保存竞品快照
pub fn save_ranking_snapshot(
    keyword: &str,
    country: &str,
    organic_top_50: Option<String>,
    sponsored_top_20: Option<String>,
) -> Result<()> {
    let conn = get_db().lock();

    conn.execute(
        "INSERT OR REPLACE INTO ranking_snapshots
            (keyword, country, snapshot_date, organic_top_50, sponsored_top_20)
         VALUES (?1, ?2, date('now', '+8 hours'), ?3, ?4)",
        rusqlite::params![keyword, country, organic_top_50, sponsored_top_20],
    )?;

    Ok(())
}

// 获取竞品快照
pub fn get_ranking_snapshots(keyword: &str, country: &str, days: i64) -> Result<Vec<RankingSnapshot>> {
    let conn = get_db().lock();

    let mut stmt = conn.prepare(
        "SELECT id, keyword, country, snapshot_date, organic_top_50, sponsored_top_20, created_at
         FROM ranking_snapshots
         WHERE keyword = ?1 AND country = ?2 AND snapshot_date >= date('now', '+8 hours', ?3)
         ORDER BY snapshot_date DESC"
    )?;

    let days_str = format!("-{} days", days);
    let snapshots = stmt
        .query_map(rusqlite::params![keyword, country, days_str], |row| {
            Ok(RankingSnapshot {
                id: row.get(0)?,
                keyword: row.get(1)?,
                country: row.get(2)?,
                snapshot_date: row.get(3)?,
                organic_top_50: row.get(4)?,
                sponsored_top_20: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(snapshots)
}

// 获取监控统计
pub fn get_monitoring_stats(product_id: i64) -> Result<MonitoringStats> {
    let conn = get_db().lock();

    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM keyword_monitoring WHERE product_id = ?1",
        [product_id],
        |row| row.get(0),
    )?;

    let active: i64 = conn.query_row(
        "SELECT COUNT(*) FROM keyword_monitoring WHERE product_id = ?1 AND is_active = 1",
        [product_id],
        |row| row.get(0),
    )?;

    let top10_organic: i64 = conn.query_row(
        "SELECT COUNT(*) FROM keyword_monitoring
         WHERE product_id = ?1 AND latest_organic_rank IS NOT NULL AND latest_organic_rank <= 10",
        [product_id],
        |row| row.get(0),
    )?;

    let top30_organic: i64 = conn.query_row(
        "SELECT COUNT(*) FROM keyword_monitoring
         WHERE product_id = ?1 AND latest_organic_rank IS NOT NULL AND latest_organic_rank <= 30",
        [product_id],
        |row| row.get(0),
    )?;

    let with_sponsored: i64 = conn.query_row(
        "SELECT COUNT(*) FROM keyword_monitoring
         WHERE product_id = ?1 AND latest_sponsored_rank IS NOT NULL",
        [product_id],
        |row| row.get(0),
    )?;

    Ok(MonitoringStats {
        total,
        active,
        top10_organic,
        top30_organic,
        with_sponsored,
    })
}

// 获取单个监控记录
pub fn get_keyword_monitoring_by_id(id: i64) -> Result<Option<KeywordMonitoring>> {
    let conn = get_db().lock();

    let result = conn.query_row(
        "SELECT id, product_id, keyword, asin, country, priority, is_active,
                latest_organic_rank, latest_organic_page, latest_sponsored_rank, latest_sponsored_page,
                image_url, price, reviews_count, rating, last_checked, created_at, tags
         FROM keyword_monitoring WHERE id = ?1",
        [id],
        |row| {
            Ok(KeywordMonitoring {
                id: row.get(0)?,
                product_id: row.get(1)?,
                keyword: row.get(2)?,
                asin: row.get(3)?,
                country: row.get(4)?,
                priority: row.get(5)?,
                is_active: row.get::<_, i64>(6)? == 1,
                latest_organic_rank: row.get(7)?,
                latest_organic_page: row.get(8)?,
                latest_sponsored_rank: row.get(9)?,
                latest_sponsored_page: row.get(10)?,
                image_url: row.get(11)?,
                price: row.get(12)?,
                reviews_count: row.get(13)?,
                rating: row.get(14)?,
                last_checked: row.get(15)?,
                created_at: row.get(16)?,
                tags: row.get(17)?,
            })
        },
    );

    match result {
        Ok(m) => Ok(Some(m)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

// 获取待检测的监控记录（活跃且未检测或超过指定时间未检测）
// hours_since_last_check = 0 表示无时间限制，返回所有活跃的监控项
pub fn get_pending_monitoring_checks(product_id: i64, hours_since_last_check: i64) -> Result<Vec<KeywordMonitoring>> {
    let conn = get_db().lock();

    // 如果 hours_since_last_check 为 0，则不限制时间，返回所有活跃监控项
    let sql = if hours_since_last_check == 0 {
        "SELECT id, product_id, keyword, asin, country, priority, is_active,
                latest_organic_rank, latest_organic_page, latest_sponsored_rank, latest_sponsored_page,
                image_url, price, reviews_count, rating, last_checked, created_at, tags
         FROM keyword_monitoring
         WHERE product_id = ?1 AND is_active = 1
         ORDER BY
           CASE priority
             WHEN 'high' THEN 1
             WHEN 'medium' THEN 2
             ELSE 3
           END,
           last_checked ASC NULLS FIRST".to_string()
    } else {
        let hours_str = format!("-{} hours", hours_since_last_check);
        format!(
            "SELECT id, product_id, keyword, asin, country, priority, is_active,
                    latest_organic_rank, latest_organic_page, latest_sponsored_rank, latest_sponsored_page,
                    image_url, price, reviews_count, rating, last_checked, created_at, tags
             FROM keyword_monitoring
             WHERE product_id = ?1 AND is_active = 1
               AND (last_checked IS NULL OR last_checked < datetime('now', '{}'))
             ORDER BY
               CASE priority
                 WHEN 'high' THEN 1
                 WHEN 'medium' THEN 2
                 ELSE 3
               END,
               last_checked ASC NULLS FIRST",
            hours_str
        )
    };

    let mut stmt = conn.prepare(&sql)?;

    let data = stmt
        .query_map(rusqlite::params![product_id], |row| {
            Ok(KeywordMonitoring {
                id: row.get(0)?,
                product_id: row.get(1)?,
                keyword: row.get(2)?,
                asin: row.get(3)?,
                country: row.get(4)?,
                priority: row.get(5)?,
                is_active: row.get::<_, i64>(6)? == 1,
                latest_organic_rank: row.get(7)?,
                latest_organic_page: row.get(8)?,
                latest_sponsored_rank: row.get(9)?,
                latest_sponsored_page: row.get(10)?,
                image_url: row.get(11)?,
                price: row.get(12)?,
                reviews_count: row.get(13)?,
                rating: row.get(14)?,
                last_checked: row.get(15)?,
                created_at: row.get(16)?,
                tags: row.get(17)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(data)
}

// 根据ID列表获取监控记录（用于选中检测）
pub fn get_monitoring_by_ids(ids: &[i64]) -> Result<Vec<KeywordMonitoring>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let conn = get_db().lock();
    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let sql = format!(
        "SELECT id, product_id, keyword, asin, country, priority, is_active,
                latest_organic_rank, latest_organic_page, latest_sponsored_rank, latest_sponsored_page,
                image_url, price, reviews_count, rating, last_checked, created_at, tags
         FROM keyword_monitoring
         WHERE id IN ({})
         ORDER BY
           CASE priority
             WHEN 'high' THEN 1
             WHEN 'medium' THEN 2
             ELSE 3
           END",
        placeholders.join(",")
    );

    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();

    let data = stmt
        .query_map(params.as_slice(), |row| {
            Ok(KeywordMonitoring {
                id: row.get(0)?,
                product_id: row.get(1)?,
                keyword: row.get(2)?,
                asin: row.get(3)?,
                country: row.get(4)?,
                priority: row.get(5)?,
                is_active: row.get::<_, i64>(6)? == 1,
                latest_organic_rank: row.get(7)?,
                latest_organic_page: row.get(8)?,
                latest_sponsored_rank: row.get(9)?,
                latest_sponsored_page: row.get(10)?,
                image_url: row.get(11)?,
                price: row.get(12)?,
                reviews_count: row.get(13)?,
                rating: row.get(14)?,
                last_checked: row.get(15)?,
                created_at: row.get(16)?,
                tags: row.get(17)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(data)
}

// 更新关键词监控标签
pub fn update_keyword_monitoring_tags(id: i64, tags: Option<String>) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE keyword_monitoring SET tags = ?1 WHERE id = ?2",
        rusqlite::params![tags, id],
    )?;
    Ok(())
}

// 批量获取监控项的迷你图数据（最近N天的排名，按日期聚合取最佳）
pub fn get_monitoring_sparklines(product_id: i64, days: i64) -> Result<Vec<MonitoringSparkline>> {
    let conn = get_db().lock();
    let days_str = format!("-{} days", days);

    // 按日期分组，每天取最佳排名（最小值）
    let mut stmt = conn.prepare(
        "SELECT h.monitoring_id, h.check_date,
                MIN(h.organic_rank) as best_organic_rank,
                MIN(h.sponsored_rank) as best_sponsored_rank
         FROM keyword_ranking_history h
         JOIN keyword_monitoring m ON h.monitoring_id = m.id
         WHERE m.product_id = ?1 AND h.check_date >= date('now', '+8 hours', ?2)
         GROUP BY h.monitoring_id, h.check_date
         ORDER BY h.monitoring_id, h.check_date ASC"
    )?;

    let rows = stmt.query_map(rusqlite::params![product_id, days_str], |row| {
        Ok((
            row.get::<_, i64>(0)?,         // monitoring_id
            row.get::<_, String>(1)?,      // check_date
            row.get::<_, Option<i64>>(2)?, // organic_rank (best)
            row.get::<_, Option<i64>>(3)?, // sponsored_rank (best)
        ))
    })?;

    // 按 monitoring_id 分组，同时存储 organic 和 sponsored
    let mut sparklines_map: std::collections::HashMap<i64, (Vec<Option<i64>>, Vec<Option<i64>>)> =
        std::collections::HashMap::new();

    for row in rows {
        let (monitoring_id, _check_date, organic_rank, sponsored_rank) = row?;
        let entry = sparklines_map
            .entry(monitoring_id)
            .or_insert_with(|| (Vec::new(), Vec::new()));
        entry.0.push(organic_rank);
        entry.1.push(sponsored_rank);
    }

    // 转换为 Vec<MonitoringSparkline>
    let sparklines: Vec<MonitoringSparkline> = sparklines_map
        .into_iter()
        .map(|(monitoring_id, (organic_ranks, sponsored_ranks))| MonitoringSparkline {
            monitoring_id,
            organic_ranks,
            sponsored_ranks,
        })
        .collect();

    Ok(sparklines)
}

// ============ 市场调研监控相关 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BsrSnapshot {
    pub id: i64,
    pub marketplace: String,
    pub category_id: String,
    pub category_name: Option<String>,
    pub snapshot_date: String,
    pub products_json: String,
    pub product_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketResearchTask {
    pub id: i64,
    pub name: String,
    pub marketplace: String,
    pub category_id: String,
    pub category_name: Option<String>,
    pub ai_provider: String,
    pub ai_model: Option<String>,   // 具体模型名称
    pub schedule_type: String,      // daily / weekly
    pub schedule_days: Option<String>,  // JSON array like [1,3,5]
    pub schedule_time: String,      // HH:MM
    pub is_enabled: bool,
    pub last_run_at: Option<String>,
    pub last_run_status: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketResearchRun {
    pub id: i64,
    pub task_id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,             // running / completed / failed
    pub report_summary: Option<String>,
    pub report_content: Option<String>,
    pub snapshot_id: Option<i64>,
    pub error_message: Option<String>,
    pub created_at: String,
}

// ============ 定时任务记录相关 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerTaskLog {
    pub id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,
    pub total_keywords: i64,
    pub success_count: i64,
    pub failed_count: i64,
    pub trigger_type: String,
    pub error_message: Option<String>,
}

// 创建新的任务记录（开始任务时调用）
pub fn create_task_log(trigger_type: &str, total_keywords: i64) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO scheduler_task_logs (started_at, status, total_keywords, trigger_type)
         VALUES (datetime('now'), 'running', ?1, ?2)",
        rusqlite::params![total_keywords, trigger_type],
    )?;
    Ok(conn.last_insert_rowid())
}

// 更新任务进度
pub fn update_task_progress(task_id: i64, success_count: i64, failed_count: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE scheduler_task_logs SET success_count = ?1, failed_count = ?2 WHERE id = ?3",
        rusqlite::params![success_count, failed_count, task_id],
    )?;
    Ok(())
}

// 完成任务记录
pub fn complete_task_log(task_id: i64, success_count: i64, failed_count: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE scheduler_task_logs
         SET ended_at = datetime('now'), status = 'completed',
             success_count = ?1, failed_count = ?2
         WHERE id = ?3",
        rusqlite::params![success_count, failed_count, task_id],
    )?;
    Ok(())
}

// 标记任务失败
pub fn fail_task_log(task_id: i64, error_message: &str) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE scheduler_task_logs
         SET ended_at = datetime('now'), status = 'failed', error_message = ?1
         WHERE id = ?2",
        rusqlite::params![error_message, task_id],
    )?;
    Ok(())
}

// 获取任务记录列表（最近N条）
pub fn get_task_logs(limit: i64) -> Result<Vec<SchedulerTaskLog>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, started_at, ended_at, status, total_keywords,
                success_count, failed_count, trigger_type, error_message
         FROM scheduler_task_logs
         ORDER BY started_at DESC
         LIMIT ?1"
    )?;

    let logs = stmt.query_map([limit], |row| {
        Ok(SchedulerTaskLog {
            id: row.get(0)?,
            started_at: row.get(1)?,
            ended_at: row.get(2)?,
            status: row.get(3)?,
            total_keywords: row.get(4)?,
            success_count: row.get(5)?,
            failed_count: row.get(6)?,
            trigger_type: row.get(7)?,
            error_message: row.get(8)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(logs)
}

// 获取正在运行的任务
pub fn get_running_task() -> Result<Option<SchedulerTaskLog>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, started_at, ended_at, status, total_keywords,
                success_count, failed_count, trigger_type, error_message
         FROM scheduler_task_logs
         WHERE status = 'running'
         ORDER BY started_at DESC
         LIMIT 1"
    )?;

    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(SchedulerTaskLog {
            id: row.get(0)?,
            started_at: row.get(1)?,
            ended_at: row.get(2)?,
            status: row.get(3)?,
            total_keywords: row.get(4)?,
            success_count: row.get(5)?,
            failed_count: row.get(6)?,
            trigger_type: row.get(7)?,
            error_message: row.get(8)?,
        }))
    } else {
        Ok(None)
    }
}

// 清空任务记录
pub fn clear_task_logs() -> Result<()> {
    let conn = get_db().lock();
    conn.execute("DELETE FROM scheduler_task_logs", [])?;
    Ok(())
}

// ============ 优化事件相关 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationEvent {
    pub id: i64,
    pub product_id: i64,
    pub event_date: String,
    pub event_type: String,          // 主类型: listing, ad
    pub event_sub_type: String,      // 子类型
    pub title: String,
    pub description: Option<String>,
    pub target_asin: Option<String>,         // 目标 ASIN（可选）
    pub affected_keywords: Option<String>,   // 关联关键词 JSON 数组（可选）
    pub created_at: String,
}

// 添加优化事件
pub fn add_optimization_event(
    product_id: i64,
    event_date: String,
    event_type: String,
    event_sub_type: String,
    title: String,
    description: Option<String>,
    target_asin: Option<String>,
    affected_keywords: Option<String>,
) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO optimization_events (product_id, event_date, event_type, event_sub_type, title, description, target_asin, affected_keywords)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![product_id, event_date, event_type, event_sub_type, title, description, target_asin, affected_keywords],
    )?;
    Ok(conn.last_insert_rowid())
}

// 获取优化事件列表
pub fn get_optimization_events(
    product_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<OptimizationEvent>> {
    let conn = get_db().lock();

    let mut sql = String::from(
        "SELECT id, product_id, event_date, event_type, COALESCE(event_sub_type, 'title') as event_sub_type, title, description, target_asin, affected_keywords, created_at
         FROM optimization_events WHERE product_id = ?1"
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(product_id)];
    let mut param_index = 2;

    if let Some(ref start) = start_date {
        sql.push_str(&format!(" AND event_date >= ?{}", param_index));
        params.push(Box::new(start.clone()));
        param_index += 1;
    }

    if let Some(ref end) = end_date {
        sql.push_str(&format!(" AND event_date <= ?{}", param_index));
        params.push(Box::new(end.clone()));
    }

    sql.push_str(" ORDER BY event_date DESC, created_at DESC");

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let events = stmt.query_map(params_refs.as_slice(), |row| {
        Ok(OptimizationEvent {
            id: row.get(0)?,
            product_id: row.get(1)?,
            event_date: row.get(2)?,
            event_type: row.get(3)?,
            event_sub_type: row.get(4)?,
            title: row.get(5)?,
            description: row.get(6)?,
            target_asin: row.get(7)?,
            affected_keywords: row.get(8)?,
            created_at: row.get(9)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(events)
}

// 更新优化事件
pub fn update_optimization_event(
    id: i64,
    event_date: String,
    event_type: String,
    event_sub_type: String,
    title: String,
    description: Option<String>,
    target_asin: Option<String>,
    affected_keywords: Option<String>,
) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE optimization_events SET event_date = ?1, event_type = ?2, event_sub_type = ?3, title = ?4,
         description = ?5, target_asin = ?6, affected_keywords = ?7 WHERE id = ?8",
        rusqlite::params![event_date, event_type, event_sub_type, title, description, target_asin, affected_keywords, id],
    )?;
    Ok(())
}

// 删除优化事件
pub fn delete_optimization_event(id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("DELETE FROM optimization_events WHERE id = ?1", [id])?;
    Ok(())
}

// ==================== 知识库模块 ====================

// 知识库分类结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KbCategory {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub sort_order: i64,
    pub color: String,
    pub created_at: String,
}

// 知识库文档结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KbDocument {
    pub id: i64,
    pub category_id: Option<i64>,
    pub title: String,
    pub file_name: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: Option<i64>,
    pub status: String,
    pub chunk_count: i64,
    pub created_at: String,
}

// 文档分块结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KbChunk {
    pub id: i64,
    pub document_id: i64,
    pub chunk_index: i64,
    pub content: String,
    pub page_number: Option<i64>,
    pub image_path: Option<String>,  // 关联的图片路径（用于图文问答）
}

// 搜索结果结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KbSearchResult {
    pub chunk_id: i64,
    pub document_id: i64,
    pub document_title: String,
    pub content: String,
    pub page_number: Option<i64>,
    pub score: f64,
    pub image_path: Option<String>,  // 关联的图片路径（用于图文问答）
}

// AI 对话结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KbConversation {
    pub id: i64,
    pub title: Option<String>,
    pub ai_provider: String,
    pub ai_model: Option<String>,
    pub created_at: String,
}

// AI 消息结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KbMessage {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    pub sources: Option<String>,
    pub created_at: String,
}

// 文档链接结构（双向链接）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KbDocumentLink {
    pub id: i64,
    pub source_doc_id: i64,
    pub target_doc_id: i64,
    pub source_title: String,
    pub target_title: String,
    pub link_text: Option<String>,
    pub created_at: String,
}

// 文档分类关联结构（多对多）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KbDocumentCategory {
    pub document_id: i64,
    pub category_id: i64,
    pub category_name: String,
    pub category_color: String,
}

// 初始化知识库表
pub fn init_knowledge_base_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- 知识库分类表
        CREATE TABLE IF NOT EXISTS kb_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            parent_id INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (parent_id) REFERENCES kb_categories(id) ON DELETE SET NULL
        );

        -- 知识库文档表
        CREATE TABLE IF NOT EXISTS kb_documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id INTEGER,
            title TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            file_size INTEGER,
            status TEXT DEFAULT 'pending',
            chunk_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (category_id) REFERENCES kb_categories(id) ON DELETE SET NULL
        );

        -- 文档分块表（用于 RAG 检索）
        CREATE TABLE IF NOT EXISTS kb_chunks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            document_id INTEGER NOT NULL,
            chunk_index INTEGER NOT NULL,
            content TEXT NOT NULL,
            page_number INTEGER,
            embedding BLOB,
            FOREIGN KEY (document_id) REFERENCES kb_documents(id) ON DELETE CASCADE
        );

        -- AI 对话表
        CREATE TABLE IF NOT EXISTS kb_conversations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT,
            ai_provider TEXT NOT NULL,
            ai_model TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- AI 消息表
        CREATE TABLE IF NOT EXISTS kb_messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            conversation_id INTEGER NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            sources TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (conversation_id) REFERENCES kb_conversations(id) ON DELETE CASCADE
        );

        -- 索引
        CREATE INDEX IF NOT EXISTS idx_kb_documents_category ON kb_documents(category_id);
        CREATE INDEX IF NOT EXISTS idx_kb_documents_status ON kb_documents(status);
        CREATE INDEX IF NOT EXISTS idx_kb_chunks_document ON kb_chunks(document_id);
        CREATE INDEX IF NOT EXISTS idx_kb_messages_conversation ON kb_messages(conversation_id);

        -- 文档链接关系表（双向链接）
        CREATE TABLE IF NOT EXISTS kb_document_links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_doc_id INTEGER NOT NULL,
            target_doc_id INTEGER NOT NULL,
            link_text TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (source_doc_id) REFERENCES kb_documents(id) ON DELETE CASCADE,
            FOREIGN KEY (target_doc_id) REFERENCES kb_documents(id) ON DELETE CASCADE,
            UNIQUE(source_doc_id, target_doc_id)
        );

        -- 文档分类关联表（多对多，一个文档可以属于多个分类）
        CREATE TABLE IF NOT EXISTS kb_document_categories (
            document_id INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            PRIMARY KEY (document_id, category_id),
            FOREIGN KEY (document_id) REFERENCES kb_documents(id) ON DELETE CASCADE,
            FOREIGN KEY (category_id) REFERENCES kb_categories(id) ON DELETE CASCADE
        );

        -- 索引
        CREATE INDEX IF NOT EXISTS idx_kb_document_links_source ON kb_document_links(source_doc_id);
        CREATE INDEX IF NOT EXISTS idx_kb_document_links_target ON kb_document_links(target_doc_id);
        CREATE INDEX IF NOT EXISTS idx_kb_document_categories_document ON kb_document_categories(document_id);
        CREATE INDEX IF NOT EXISTS idx_kb_document_categories_category ON kb_document_categories(category_id);
        "
    )?;

    // 创建 FTS5 全文搜索虚拟表
    // 注意：FTS5 表需要单独创建，不能在 execute_batch 中创建
    let fts_exists: bool = conn
        .prepare("SELECT 1 FROM kb_chunks_fts LIMIT 1")
        .is_ok();

    if !fts_exists {
        conn.execute(
            "CREATE VIRTUAL TABLE kb_chunks_fts USING fts5(content, content=kb_chunks, content_rowid=id)",
            [],
        )?;

        // 创建触发器以保持 FTS 索引同步
        conn.execute_batch(
            "
            -- 插入触发器
            CREATE TRIGGER IF NOT EXISTS kb_chunks_fts_insert AFTER INSERT ON kb_chunks BEGIN
                INSERT INTO kb_chunks_fts(rowid, content) VALUES (new.id, new.content);
            END;

            -- 删除触发器
            CREATE TRIGGER IF NOT EXISTS kb_chunks_fts_delete AFTER DELETE ON kb_chunks BEGIN
                INSERT INTO kb_chunks_fts(kb_chunks_fts, rowid, content) VALUES('delete', old.id, old.content);
            END;

            -- 更新触发器
            CREATE TRIGGER IF NOT EXISTS kb_chunks_fts_update AFTER UPDATE ON kb_chunks BEGIN
                INSERT INTO kb_chunks_fts(kb_chunks_fts, rowid, content) VALUES('delete', old.id, old.content);
                INSERT INTO kb_chunks_fts(rowid, content) VALUES (new.id, new.content);
            END;
            "
        )?;
    }

    // 数据库迁移：为 kb_chunks 表添加 embedding 字段（兼容旧版本）
    let has_embedding_column: bool = conn
        .prepare("SELECT embedding FROM kb_chunks LIMIT 1")
        .is_ok();

    if !has_embedding_column {
        conn.execute("ALTER TABLE kb_chunks ADD COLUMN embedding BLOB", [])?;
        println!("[DB Migration] Added embedding column to kb_chunks table");
    }

    // 数据库迁移：为 kb_chunks 表添加 image_path 字段（用于图文问答）
    let has_image_path_column: bool = conn
        .prepare("SELECT image_path FROM kb_chunks LIMIT 1")
        .is_ok();

    if !has_image_path_column {
        conn.execute("ALTER TABLE kb_chunks ADD COLUMN image_path TEXT", [])?;
        println!("[DB Migration] Added image_path column to kb_chunks table");
    }

    // 数据库迁移：为 kb_categories 表添加 sort_order 和 color 字段
    let has_sort_order_column: bool = conn
        .prepare("SELECT sort_order FROM kb_categories LIMIT 1")
        .is_ok();

    if !has_sort_order_column {
        conn.execute("ALTER TABLE kb_categories ADD COLUMN sort_order INTEGER DEFAULT 0", [])?;
        conn.execute("ALTER TABLE kb_categories ADD COLUMN color TEXT DEFAULT '#409EFF'", [])?;
        println!("[DB Migration] Added sort_order and color columns to kb_categories table");
    }

    // 数据库迁移：将旧的 category_id 数据迁移到新的多对多关联表
    // 检查是否已经迁移过（通过检查 kb_document_categories 表是否有数据）
    let migration_needed: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM kb_documents WHERE category_id IS NOT NULL",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) > 0;

    let already_migrated: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM kb_document_categories",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) > 0;

    if migration_needed && !already_migrated {
        conn.execute(
            "INSERT INTO kb_document_categories (document_id, category_id)
             SELECT id, category_id FROM kb_documents WHERE category_id IS NOT NULL",
            [],
        )?;
        println!("[DB Migration] Migrated category_id data to kb_document_categories table");
    }

    Ok(())
}

// ==================== 知识库分类 CRUD ====================

// 创建知识库分类
pub fn kb_create_category(name: String, parent_id: Option<i64>) -> Result<i64> {
    let conn = get_db().lock();
    // 获取当前最大的 sort_order
    let max_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM kb_categories",
            [],
            |row| row.get(0),
        )
        .unwrap_or(-1);

    conn.execute(
        "INSERT INTO kb_categories (name, parent_id, sort_order, color) VALUES (?1, ?2, ?3, '#409EFF')",
        rusqlite::params![name, parent_id, max_order + 1],
    )?;
    Ok(conn.last_insert_rowid())
}

// 获取所有知识库分类
pub fn kb_get_categories() -> Result<Vec<KbCategory>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, parent_id, COALESCE(sort_order, 0), COALESCE(color, '#409EFF'), created_at FROM kb_categories ORDER BY sort_order ASC, id ASC"
    )?;

    let categories = stmt.query_map([], |row| {
        Ok(KbCategory {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            sort_order: row.get(3)?,
            color: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(categories)
}

// 删除知识库分类
pub fn kb_delete_category(id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("DELETE FROM kb_categories WHERE id = ?1", [id])?;
    Ok(())
}

// 更新知识库分类名称
pub fn kb_update_category(id: i64, name: String) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("UPDATE kb_categories SET name = ?1 WHERE id = ?2", rusqlite::params![name, id])?;
    Ok(())
}

// 更新知识库分类颜色
pub fn kb_update_category_color(id: i64, color: String) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("UPDATE kb_categories SET color = ?1 WHERE id = ?2", rusqlite::params![color, id])?;
    Ok(())
}

// 批量更新知识库分类排序
pub fn kb_update_categories_order(ids: Vec<i64>) -> Result<()> {
    let conn = get_db().lock();
    for (index, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE kb_categories SET sort_order = ?1 WHERE id = ?2",
            rusqlite::params![index as i64, id],
        )?;
    }
    Ok(())
}

// ==================== 知识库文档 CRUD ====================

// 添加文档记录
pub fn kb_add_document(
    category_id: Option<i64>,
    title: String,
    file_name: String,
    file_path: String,
    file_type: String,
    file_size: Option<i64>,
) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO kb_documents (category_id, title, file_name, file_path, file_type, file_size, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending')",
        rusqlite::params![category_id, title, file_name, file_path, file_type, file_size],
    )?;
    Ok(conn.last_insert_rowid())
}

// 更新文档状态
pub fn kb_update_document_status(id: i64, status: String, chunk_count: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE kb_documents SET status = ?1, chunk_count = ?2 WHERE id = ?3",
        rusqlite::params![status, chunk_count, id],
    )?;
    Ok(())
}

// 更新文档分类
pub fn kb_update_document_category(id: i64, category_id: Option<i64>) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE kb_documents SET category_id = ?1 WHERE id = ?2",
        rusqlite::params![category_id, id],
    )?;
    Ok(())
}

// 获取文档列表
pub fn kb_get_documents(category_id: Option<i64>) -> Result<Vec<KbDocument>> {
    let conn = get_db().lock();

    let sql = if category_id.is_some() {
        "SELECT id, category_id, title, file_name, file_path, file_type, file_size, status, chunk_count, created_at
         FROM kb_documents WHERE category_id = ?1 ORDER BY created_at DESC"
    } else {
        "SELECT id, category_id, title, file_name, file_path, file_type, file_size, status, chunk_count, created_at
         FROM kb_documents ORDER BY created_at DESC"
    };

    let mut stmt = conn.prepare(sql)?;

    let documents = if let Some(cat_id) = category_id {
        stmt.query_map([cat_id], |row| {
            Ok(KbDocument {
                id: row.get(0)?,
                category_id: row.get(1)?,
                title: row.get(2)?,
                file_name: row.get(3)?,
                file_path: row.get(4)?,
                file_type: row.get(5)?,
                file_size: row.get(6)?,
                status: row.get(7)?,
                chunk_count: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect()
    } else {
        stmt.query_map([], |row| {
            Ok(KbDocument {
                id: row.get(0)?,
                category_id: row.get(1)?,
                title: row.get(2)?,
                file_name: row.get(3)?,
                file_path: row.get(4)?,
                file_type: row.get(5)?,
                file_size: row.get(6)?,
                status: row.get(7)?,
                chunk_count: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect()
    };

    Ok(documents)
}

// 删除文档
pub fn kb_delete_document(id: i64) -> Result<()> {
    let conn = get_db().lock();
    // 级联删除会自动删除相关的 chunks
    conn.execute("DELETE FROM kb_documents WHERE id = ?1", [id])?;
    Ok(())
}

// ==================== 文档分块 CRUD ====================

// 添加文档分块
pub fn kb_add_chunk(document_id: i64, chunk_index: i64, content: String, page_number: Option<i64>) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO kb_chunks (document_id, chunk_index, content, page_number) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![document_id, chunk_index, content, page_number],
    )?;
    Ok(conn.last_insert_rowid())
}

// 添加带图片路径的文档分块（用于图文问答）
pub fn kb_add_chunk_with_image(document_id: i64, chunk_index: i64, content: String, page_number: Option<i64>, image_path: String) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO kb_chunks (document_id, chunk_index, content, page_number, image_path) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![document_id, chunk_index, content, page_number, image_path],
    )?;
    Ok(conn.last_insert_rowid())
}

// 批量添加文档分块
pub fn kb_add_chunks_batch(document_id: i64, chunks: Vec<(String, Option<i64>)>) -> Result<i64> {
    let conn = get_db().lock();

    for (index, (content, page_number)) in chunks.iter().enumerate() {
        conn.execute(
            "INSERT INTO kb_chunks (document_id, chunk_index, content, page_number) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![document_id, index as i64, content, page_number],
        )?;
    }

    Ok(chunks.len() as i64)
}

// 获取文档的所有分块
pub fn kb_get_chunks(document_id: i64) -> Result<Vec<KbChunk>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, document_id, chunk_index, content, page_number, image_path
         FROM kb_chunks WHERE document_id = ?1 ORDER BY chunk_index"
    )?;

    let chunks = stmt.query_map([document_id], |row| {
        Ok(KbChunk {
            id: row.get(0)?,
            document_id: row.get(1)?,
            chunk_index: row.get(2)?,
            content: row.get(3)?,
            page_number: row.get(4)?,
            image_path: row.get(5)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(chunks)
}

// 更新分块的 embedding 向量
pub fn kb_update_chunk_embedding(chunk_id: i64, embedding: Vec<f32>) -> Result<()> {
    let conn = get_db().lock();
    // 将 f32 向量转换为字节数组存储
    let embedding_bytes: Vec<u8> = embedding
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();

    conn.execute(
        "UPDATE kb_chunks SET embedding = ?1 WHERE id = ?2",
        rusqlite::params![embedding_bytes, chunk_id],
    )?;
    Ok(())
}

// 清除所有 embedding（用于迁移到新的 embedding 模型）
pub fn kb_clear_all_embeddings() -> Result<i64> {
    let conn = get_db().lock();
    let count = conn.execute("UPDATE kb_chunks SET embedding = NULL WHERE embedding IS NOT NULL", [])?;
    Ok(count as i64)
}

// 获取所有没有 embedding 的分块
pub fn kb_get_chunks_without_embedding(document_id: i64) -> Result<Vec<KbChunk>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, document_id, chunk_index, content, page_number, image_path
         FROM kb_chunks WHERE document_id = ?1 AND embedding IS NULL ORDER BY chunk_index"
    )?;

    let chunks = stmt.query_map([document_id], |row| {
        Ok(KbChunk {
            id: row.get(0)?,
            document_id: row.get(1)?,
            chunk_index: row.get(2)?,
            content: row.get(3)?,
            page_number: row.get(4)?,
            image_path: row.get(5)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(chunks)
}

/// 获取文档的向量化统计（总分块数，已向量化数）
pub fn kb_get_document_embedding_stats(document_id: i64) -> Result<(i64, i64)> {
    let conn = get_db().lock();
    let (total, embedded): (i64, i64) = conn.query_row(
        "SELECT
            COUNT(*) as total,
            COUNT(embedding) as embedded
         FROM kb_chunks WHERE document_id = ?1",
        [document_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok((total, embedded))
}

// 向量相似度搜索（支持相关度阈值过滤）
pub fn kb_vector_search(query_embedding: Vec<f32>, limit: i64, min_score: f64) -> Result<Vec<KbSearchResult>> {
    let conn = get_db().lock();

    // 获取所有有 embedding 的 chunks
    let mut stmt = conn.prepare(
        "SELECT c.id, c.document_id, d.title, c.content, c.page_number, c.embedding, c.image_path
         FROM kb_chunks c
         JOIN kb_documents d ON c.document_id = d.id
         WHERE c.embedding IS NOT NULL"
    )?;

    let mut results: Vec<(KbSearchResult, f64)> = stmt.query_map([], |row| {
        let chunk_id: i64 = row.get(0)?;
        let document_id: i64 = row.get(1)?;
        let document_title: String = row.get(2)?;
        let content: String = row.get(3)?;
        let page_number: Option<i64> = row.get(4)?;
        let embedding_bytes: Vec<u8> = row.get(5)?;
        let image_path: Option<String> = row.get(6)?;

        // 将字节数组转换回 f32 向量
        let embedding: Vec<f32> = embedding_bytes
            .chunks(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        // 计算余弦相似度
        let similarity = cosine_similarity(&query_embedding, &embedding);

        Ok((KbSearchResult {
            chunk_id,
            document_id,
            document_title,
            content,
            page_number,
            score: similarity,
            image_path,
        }, similarity))
    })?
    .filter_map(|r| r.ok())
    .collect();

    // 按相似度降序排序
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // 过滤：只保留相关度 >= min_score 的结果，然后取前 limit 个
    let results: Vec<KbSearchResult> = results
        .into_iter()
        .filter(|(_, score)| *score >= min_score)
        .take(limit as usize)
        .map(|(r, _)| r)
        .collect();

    Ok(results)
}

// 计算余弦相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
    let norm_a: f64 = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

// ==================== 知识库搜索 ====================

// 全文搜索（支持中文）
pub fn kb_search(query: String, limit: i64) -> Result<Vec<KbSearchResult>> {
    let conn = get_db().lock();

    // 先尝试 FTS5 搜索
    let fts_results = try_fts_search(&conn, &query, limit)?;

    // 如果 FTS5 没有结果，使用 LIKE 搜索（对中文更友好）
    if fts_results.is_empty() {
        return like_search(&conn, &query, limit);
    }

    Ok(fts_results)
}

// FTS5 搜索
fn try_fts_search(conn: &rusqlite::Connection, query: &str, limit: i64) -> Result<Vec<KbSearchResult>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.document_id, d.title, c.content, c.page_number,
                bm25(kb_chunks_fts) as score, c.image_path
         FROM kb_chunks_fts
         JOIN kb_chunks c ON kb_chunks_fts.rowid = c.id
         JOIN kb_documents d ON c.document_id = d.id
         WHERE kb_chunks_fts MATCH ?1
         ORDER BY score
         LIMIT ?2"
    )?;

    let results = stmt.query_map(rusqlite::params![query, limit], |row| {
        Ok(KbSearchResult {
            chunk_id: row.get(0)?,
            document_id: row.get(1)?,
            document_title: row.get(2)?,
            content: row.get(3)?,
            page_number: row.get(4)?,
            score: row.get(5)?,
            image_path: row.get(6)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(results)
}

// LIKE 搜索（支持中文）- 智能分词版本
fn like_search(conn: &rusqlite::Connection, query: &str, limit: i64) -> Result<Vec<KbSearchResult>> {
    // 中文停用词列表
    const STOP_WORDS: &[&str] = &[
        "的", "是", "在", "了", "和", "与", "或", "我", "我们", "你", "你们",
        "他", "她", "它", "这", "那", "有", "没有", "不", "也", "都", "就",
        "要", "会", "可以", "能", "请", "帮", "帮我", "查看", "查找", "搜索",
        "知识库", "文档", "中", "里", "上", "下", "什么", "怎么", "如何",
        "哪里", "哪个", "为什么", "告诉", "一下", "一个",
    ];

    let mut keywords: Vec<String> = Vec::new();

    // 按空格和标点分割
    let parts: Vec<&str> = query
        .split(|c: char| c.is_whitespace() || c == ',' || c == '，' || c == '。' || c == '？' || c == '！' || c == '：' || c == ':')
        .filter(|s| !s.is_empty())
        .collect();

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // 如果是纯ASCII（英文），直接添加
        if part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            if part.len() >= 2 {
                keywords.push(part.to_lowercase());
            }
        } else {
            // 包含中文：使用滑动窗口提取2-4字符的词组
            let chars: Vec<char> = part.chars().collect();

            // 先尝试提取较长的词组（4字符），再提取短的（2字符）
            for window_size in [4, 3, 2] {
                if chars.len() >= window_size {
                    for i in 0..=(chars.len() - window_size) {
                        let word: String = chars[i..i + window_size].iter().collect();
                        // 过滤停用词
                        if !STOP_WORDS.contains(&word.as_str()) {
                            keywords.push(word);
                        }
                    }
                }
            }

            // 如果原始词组较短（<=4字符）且不是停用词，也添加
            if chars.len() <= 4 && chars.len() >= 2 {
                let word: String = chars.iter().collect();
                if !STOP_WORDS.contains(&word.as_str()) {
                    keywords.push(word);
                }
            }
        }
    }

    // 去重
    keywords.sort();
    keywords.dedup();

    if keywords.is_empty() {
        return Ok(vec![]);
    }

    // 限制关键词数量，避免SQL过长（优先保留较长的词）
    if keywords.len() > 15 {
        // 按长度降序排序，保留较长的词（更有意义）
        keywords.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
        keywords.truncate(15);
    }

    // 构建 LIKE 查询条件 (OR 逻辑)
    let like_conditions: Vec<String> = keywords
        .iter()
        .map(|k| format!("c.content LIKE '%{}%'", k.replace("'", "''")))
        .collect();

    let where_clause = like_conditions.join(" OR ");

    // 构建相关性评分：每匹配一个关键词加1分，匹配越多越相关
    let score_cases: Vec<String> = keywords
        .iter()
        .map(|k| format!("(CASE WHEN c.content LIKE '%{}%' THEN 1 ELSE 0 END)", k.replace("'", "''")))
        .collect();
    let score_expr = if score_cases.is_empty() {
        "0".to_string()
    } else {
        score_cases.join(" + ")
    };

    let sql = format!(
        "SELECT c.id, c.document_id, d.title, c.content, c.page_number, ({}) as score, c.image_path
         FROM kb_chunks c
         JOIN kb_documents d ON c.document_id = d.id
         WHERE {}
         ORDER BY score DESC
         LIMIT ?1",
        score_expr, where_clause
    );

    let mut stmt = conn.prepare(&sql)?;

    let results = stmt.query_map(rusqlite::params![limit], |row| {
        Ok(KbSearchResult {
            chunk_id: row.get(0)?,
            document_id: row.get(1)?,
            document_title: row.get(2)?,
            content: row.get(3)?,
            page_number: row.get(4)?,
            score: row.get(5)?,
            image_path: row.get(6)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(results)
}

// ==================== AI 对话 CRUD ====================

// 创建对话
pub fn kb_create_conversation(ai_provider: String, ai_model: Option<String>, title: Option<String>) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO kb_conversations (ai_provider, ai_model, title) VALUES (?1, ?2, ?3)",
        rusqlite::params![ai_provider, ai_model, title],
    )?;
    Ok(conn.last_insert_rowid())
}

// 获取对话列表
pub fn kb_get_conversations() -> Result<Vec<KbConversation>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, title, ai_provider, ai_model, created_at
         FROM kb_conversations ORDER BY created_at DESC"
    )?;

    let conversations = stmt.query_map([], |row| {
        Ok(KbConversation {
            id: row.get(0)?,
            title: row.get(1)?,
            ai_provider: row.get(2)?,
            ai_model: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(conversations)
}

// 更新对话标题
pub fn kb_update_conversation_title(id: i64, title: String) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("UPDATE kb_conversations SET title = ?1 WHERE id = ?2", rusqlite::params![title, id])?;
    Ok(())
}

// 删除对话
pub fn kb_delete_conversation(id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("DELETE FROM kb_conversations WHERE id = ?1", [id])?;
    Ok(())
}

// 添加消息
pub fn kb_add_message(
    conversation_id: i64,
    role: String,
    content: String,
    sources: Option<String>,
) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO kb_messages (conversation_id, role, content, sources) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![conversation_id, role, content, sources],
    )?;
    Ok(conn.last_insert_rowid())
}

// 获取对话消息
pub fn kb_get_messages(conversation_id: i64) -> Result<Vec<KbMessage>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, sources, created_at
         FROM kb_messages WHERE conversation_id = ?1 ORDER BY created_at"
    )?;

    let messages = stmt.query_map([conversation_id], |row| {
        Ok(KbMessage {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            sources: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(messages)
}

// ==================== 文档链接 CRUD ====================

// 添加文档链接
pub fn kb_add_document_link(source_id: i64, target_id: i64, link_text: Option<String>) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT OR IGNORE INTO kb_document_links (source_doc_id, target_doc_id, link_text) VALUES (?1, ?2, ?3)",
        rusqlite::params![source_id, target_id, link_text],
    )?;
    Ok(conn.last_insert_rowid())
}

// 移除文档链接
pub fn kb_remove_document_link(source_id: i64, target_id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "DELETE FROM kb_document_links WHERE source_doc_id = ?1 AND target_doc_id = ?2",
        rusqlite::params![source_id, target_id],
    )?;
    Ok(())
}

// 获取文档的出链（从当前文档链接到其他文档）
pub fn kb_get_document_links(doc_id: i64) -> Result<Vec<KbDocumentLink>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT l.id, l.source_doc_id, l.target_doc_id,
                s.title as source_title, t.title as target_title,
                l.link_text, l.created_at
         FROM kb_document_links l
         JOIN kb_documents s ON l.source_doc_id = s.id
         JOIN kb_documents t ON l.target_doc_id = t.id
         WHERE l.source_doc_id = ?1
         ORDER BY l.created_at DESC"
    )?;

    let links = stmt.query_map([doc_id], |row| {
        Ok(KbDocumentLink {
            id: row.get(0)?,
            source_doc_id: row.get(1)?,
            target_doc_id: row.get(2)?,
            source_title: row.get(3)?,
            target_title: row.get(4)?,
            link_text: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(links)
}

// 获取文档的反向链接（其他文档链接到当前文档）
pub fn kb_get_document_backlinks(doc_id: i64) -> Result<Vec<KbDocumentLink>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT l.id, l.source_doc_id, l.target_doc_id,
                s.title as source_title, t.title as target_title,
                l.link_text, l.created_at
         FROM kb_document_links l
         JOIN kb_documents s ON l.source_doc_id = s.id
         JOIN kb_documents t ON l.target_doc_id = t.id
         WHERE l.target_doc_id = ?1
         ORDER BY l.created_at DESC"
    )?;

    let links = stmt.query_map([doc_id], |row| {
        Ok(KbDocumentLink {
            id: row.get(0)?,
            source_doc_id: row.get(1)?,
            target_doc_id: row.get(2)?,
            source_title: row.get(3)?,
            target_title: row.get(4)?,
            link_text: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(links)
}

// 获取所有链接（用于知识图谱）
pub fn kb_get_all_links() -> Result<Vec<KbDocumentLink>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT l.id, l.source_doc_id, l.target_doc_id,
                s.title as source_title, t.title as target_title,
                l.link_text, l.created_at
         FROM kb_document_links l
         JOIN kb_documents s ON l.source_doc_id = s.id
         JOIN kb_documents t ON l.target_doc_id = t.id
         ORDER BY l.created_at DESC"
    )?;

    let links = stmt.query_map([], |row| {
        Ok(KbDocumentLink {
            id: row.get(0)?,
            source_doc_id: row.get(1)?,
            target_doc_id: row.get(2)?,
            source_title: row.get(3)?,
            target_title: row.get(4)?,
            link_text: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(links)
}

// ==================== 文档分类关联 CRUD（多对多）====================

// 给文档添加分类
pub fn kb_add_document_category(doc_id: i64, category_id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT OR IGNORE INTO kb_document_categories (document_id, category_id) VALUES (?1, ?2)",
        rusqlite::params![doc_id, category_id],
    )?;
    Ok(())
}

// 移除文档分类
pub fn kb_remove_document_category(doc_id: i64, category_id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "DELETE FROM kb_document_categories WHERE document_id = ?1 AND category_id = ?2",
        rusqlite::params![doc_id, category_id],
    )?;
    Ok(())
}

// 获取文档的所有分类
pub fn kb_get_document_categories(doc_id: i64) -> Result<Vec<KbDocumentCategory>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT dc.document_id, dc.category_id, c.name, c.color
         FROM kb_document_categories dc
         JOIN kb_categories c ON dc.category_id = c.id
         WHERE dc.document_id = ?1
         ORDER BY c.sort_order, c.name"
    )?;

    let categories = stmt.query_map([doc_id], |row| {
        Ok(KbDocumentCategory {
            document_id: row.get(0)?,
            category_id: row.get(1)?,
            category_name: row.get(2)?,
            category_color: row.get(3)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(categories)
}

// 按分类筛选文档（多对多版本）
pub fn kb_get_documents_by_categories(category_id: i64) -> Result<Vec<KbDocument>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT d.id, d.category_id, d.title, d.file_name, d.file_path,
                d.file_type, d.file_size, d.status, d.chunk_count, d.created_at
         FROM kb_documents d
         JOIN kb_document_categories dc ON d.id = dc.document_id
         WHERE dc.category_id = ?1
         ORDER BY d.created_at DESC"
    )?;

    let documents = stmt.query_map([category_id], |row| {
        Ok(KbDocument {
            id: row.get(0)?,
            category_id: row.get(1)?,
            title: row.get(2)?,
            file_name: row.get(3)?,
            file_path: row.get(4)?,
            file_type: row.get(5)?,
            file_size: row.get(6)?,
            status: row.get(7)?,
            chunk_count: row.get(8)?,
            created_at: row.get(9)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(documents)
}

// 设置文档的分类（替换所有现有分类）
pub fn kb_set_document_categories(doc_id: i64, category_ids: Vec<i64>) -> Result<()> {
    let conn = get_db().lock();
    // 先删除所有现有分类
    conn.execute(
        "DELETE FROM kb_document_categories WHERE document_id = ?1",
        [doc_id],
    )?;
    // 添加新分类
    for cat_id in category_ids {
        conn.execute(
            "INSERT INTO kb_document_categories (document_id, category_id) VALUES (?1, ?2)",
            rusqlite::params![doc_id, cat_id],
        )?;
    }
    Ok(())
}

// ==================== 智能文案模块 ====================

// 初始化智能文案表
pub fn init_smart_copy_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- 智能文案项目表
        CREATE TABLE IF NOT EXISTS sc_projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            scenario_type TEXT NOT NULL CHECK(scenario_type IN ('new', 'optimize')),
            marketplace TEXT NOT NULL DEFAULT 'US',
            my_asin TEXT,
            status TEXT DEFAULT 'draft' CHECK(status IN ('draft', 'fetching', 'analyzing', 'completed', 'failed')),
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- 竞品表
        CREATE TABLE IF NOT EXISTS sc_competitors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            asin TEXT NOT NULL,
            competitor_type TEXT DEFAULT 'direct' CHECK(competitor_type IN ('top', 'direct', 'rising')),
            title TEXT,
            price TEXT,
            rating TEXT,
            review_count INTEGER,
            bsr_rank TEXT,
            date_first_available TEXT,
            image_url TEXT,
            bullets TEXT,
            description TEXT,
            fetched_at DATETIME,
            FOREIGN KEY (project_id) REFERENCES sc_projects(id) ON DELETE CASCADE
        );

        -- 迁移：给 sc_competitors 添加 image_url 字段（如果不存在）
        -- SQLite 不支持 IF NOT EXISTS 对列，用 PRAGMA 检查会复杂，这里用简单方式

        -- 竞品评论表
        CREATE TABLE IF NOT EXISTS sc_reviews (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            competitor_id INTEGER NOT NULL,
            star_rating INTEGER NOT NULL,
            review_text TEXT,
            review_date TEXT,
            helpful_votes INTEGER DEFAULT 0,
            FOREIGN KEY (competitor_id) REFERENCES sc_competitors(id) ON DELETE CASCADE
        );

        -- 竞品 Q&A 表
        CREATE TABLE IF NOT EXISTS sc_qa (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            competitor_id INTEGER NOT NULL,
            question TEXT NOT NULL,
            answer TEXT,
            votes INTEGER DEFAULT 0,
            FOREIGN KEY (competitor_id) REFERENCES sc_competitors(id) ON DELETE CASCADE
        );

        -- AI 分析结果表
        CREATE TABLE IF NOT EXISTS sc_analysis_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            analysis_type TEXT NOT NULL,
            result_json TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (project_id) REFERENCES sc_projects(id) ON DELETE CASCADE
        );

        -- 索引
        CREATE INDEX IF NOT EXISTS idx_sc_competitors_project ON sc_competitors(project_id);
        CREATE INDEX IF NOT EXISTS idx_sc_reviews_competitor ON sc_reviews(competitor_id);
        CREATE INDEX IF NOT EXISTS idx_sc_qa_competitor ON sc_qa(competitor_id);
        CREATE INDEX IF NOT EXISTS idx_sc_analysis_project ON sc_analysis_results(project_id);
        "
    )?;

    // 迁移：给 sc_competitors 添加 image_url 字段（如果不存在）
    let _ = conn.execute("ALTER TABLE sc_competitors ADD COLUMN image_url TEXT", []);

    // 迁移：给 sc_competitors 添加 date_first_available 字段（上架时间）
    let _ = conn.execute("ALTER TABLE sc_competitors ADD COLUMN date_first_available TEXT", []);

    // 迁移：给 sc_projects 添加 product_id 字段（关联关键词数据）
    let _ = conn.execute("ALTER TABLE sc_projects ADD COLUMN product_id INTEGER REFERENCES products(id)", []);

    // 迁移：给 sc_analysis_results 添加 model 相关字段
    let _ = conn.execute("ALTER TABLE sc_analysis_results ADD COLUMN model_provider TEXT", []);
    let _ = conn.execute("ALTER TABLE sc_analysis_results ADD COLUMN model_name TEXT", []);

    // 迁移：给 sc_projects 添加 my_product_info 字段（存储用户产品信息 JSON）
    let _ = conn.execute("ALTER TABLE sc_projects ADD COLUMN my_product_info TEXT", []);

    // 迁移：给 sc_projects 添加用户 Listing 信息字段（老品优化时使用）
    let _ = conn.execute("ALTER TABLE sc_projects ADD COLUMN my_title TEXT", []);
    let _ = conn.execute("ALTER TABLE sc_projects ADD COLUMN my_bullets TEXT", []);
    let _ = conn.execute("ALTER TABLE sc_projects ADD COLUMN my_description TEXT", []);
    let _ = conn.execute("ALTER TABLE sc_projects ADD COLUMN my_listing_fetched_at DATETIME", []);

    // ==================== 智能广告（Smart Ads）表 ====================
    conn.execute_batch(
        "
        -- 广告项目表
        CREATE TABLE IF NOT EXISTS ad_projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_id INTEGER,
            name TEXT NOT NULL,
            marketplace TEXT DEFAULT 'US',
            target_acos REAL DEFAULT 30.0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (product_id) REFERENCES products(id)
        );

        -- 搜索词数据表（从报表导入）
        CREATE TABLE IF NOT EXISTS ad_search_terms (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            portfolio_name TEXT,
            campaign_name TEXT,
            ad_group_name TEXT,
            country TEXT,
            targeting TEXT,
            match_type TEXT,
            customer_search_term TEXT,
            impressions INTEGER DEFAULT 0,
            clicks INTEGER DEFAULT 0,
            ctr REAL DEFAULT 0,
            spend REAL DEFAULT 0,
            sales REAL DEFAULT 0,
            orders INTEGER DEFAULT 0,
            acos REAL DEFAULT 0,
            roas REAL DEFAULT 0,
            conversion_rate REAL DEFAULT 0,
            cpc REAL DEFAULT 0,
            report_date TEXT,
            sku TEXT,
            imported_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (project_id) REFERENCES ad_projects(id) ON DELETE CASCADE
        );

        -- 广告分析结果表
        CREATE TABLE IF NOT EXISTS ad_analysis_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            analysis_type TEXT NOT NULL,
            result_json TEXT NOT NULL,
            ai_provider TEXT,
            ai_model TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (project_id) REFERENCES ad_projects(id) ON DELETE CASCADE
        );

        -- 索引
        CREATE INDEX IF NOT EXISTS idx_ad_search_terms_project ON ad_search_terms(project_id);
        CREATE INDEX IF NOT EXISTS idx_ad_search_terms_acos ON ad_search_terms(acos);
        CREATE INDEX IF NOT EXISTS idx_ad_search_terms_spend ON ad_search_terms(spend);
        CREATE INDEX IF NOT EXISTS idx_ad_analysis_project ON ad_analysis_results(project_id);
        "
    )?;

    // 迁移：给 ad_search_terms 添加 portfolio_name、country、sku 字段（用于现有数据库）
    let _ = conn.execute("ALTER TABLE ad_search_terms ADD COLUMN portfolio_name TEXT", []);
    let _ = conn.execute("ALTER TABLE ad_search_terms ADD COLUMN country TEXT", []);
    let _ = conn.execute("ALTER TABLE ad_search_terms ADD COLUMN sku TEXT", []);

    Ok(())
}

// 项目相关结构体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScProject {
    pub id: i64,
    pub name: String,
    pub scenario_type: String,
    pub marketplace: String,
    pub my_asin: Option<String>,
    pub product_id: Option<i64>,  // 关联的产品ID（用于获取关键词数据）
    pub my_product_info: Option<String>,  // 我的产品信息（JSON）
    // 用户的 Listing 信息（老品优化时使用）
    pub my_title: Option<String>,
    pub my_bullets: Option<String>,  // JSON 数组
    pub my_description: Option<String>,
    pub my_listing_fetched_at: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub competitor_count: i64,
}

// 创建项目
pub fn sc_create_project(
    name: &str,
    scenario_type: &str,
    marketplace: &str,
    my_asin: Option<&str>,
    product_id: Option<i64>,
) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO sc_projects (name, scenario_type, marketplace, my_asin, product_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![name, scenario_type, marketplace, my_asin, product_id],
    )?;
    Ok(conn.last_insert_rowid())
}

// 获取项目列表
pub fn sc_get_projects(scenario_type: Option<&str>) -> Result<Vec<ScProject>> {
    let conn = get_db().lock();

    let base_sql = "
        SELECT p.id, p.name, p.scenario_type, p.marketplace, p.my_asin, p.product_id, p.my_product_info,
               p.my_title, p.my_bullets, p.my_description, p.my_listing_fetched_at,
               p.status, p.created_at, p.updated_at,
               (SELECT COUNT(*) FROM sc_competitors WHERE project_id = p.id) as competitor_count
        FROM sc_projects p
    ";

    let sql = match scenario_type {
        Some(_) => format!("{} WHERE p.scenario_type = ?1 ORDER BY p.updated_at DESC", base_sql),
        None => format!("{} ORDER BY p.updated_at DESC", base_sql),
    };

    let mut stmt = conn.prepare(&sql)?;

    let projects: Vec<ScProject> = if let Some(st) = scenario_type {
        let rows = stmt.query_map(rusqlite::params![st], |row| {
            Ok(ScProject {
                id: row.get(0)?,
                name: row.get(1)?,
                scenario_type: row.get(2)?,
                marketplace: row.get(3)?,
                my_asin: row.get(4)?,
                product_id: row.get(5)?,
                my_product_info: row.get(6)?,
                my_title: row.get(7)?,
                my_bullets: row.get(8)?,
                my_description: row.get(9)?,
                my_listing_fetched_at: row.get(10)?,
                status: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                competitor_count: row.get(14)?,
            })
        })?;
        rows.collect::<Result<Vec<_>>>()?
    } else {
        let rows = stmt.query_map([], |row| {
            Ok(ScProject {
                id: row.get(0)?,
                name: row.get(1)?,
                scenario_type: row.get(2)?,
                marketplace: row.get(3)?,
                my_asin: row.get(4)?,
                product_id: row.get(5)?,
                my_product_info: row.get(6)?,
                my_title: row.get(7)?,
                my_bullets: row.get(8)?,
                my_description: row.get(9)?,
                my_listing_fetched_at: row.get(10)?,
                status: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                competitor_count: row.get(14)?,
            })
        })?;
        rows.collect::<Result<Vec<_>>>()?
    };

    Ok(projects)
}

// 获取单个项目
pub fn sc_get_project(id: i64) -> Result<Option<ScProject>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.scenario_type, p.marketplace, p.my_asin, p.product_id, p.my_product_info,
                p.my_title, p.my_bullets, p.my_description, p.my_listing_fetched_at,
                p.status, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM sc_competitors WHERE project_id = p.id) as competitor_count
         FROM sc_projects p
         WHERE p.id = ?1"
    )?;

    let mut rows = stmt.query(rusqlite::params![id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(ScProject {
            id: row.get(0)?,
            name: row.get(1)?,
            scenario_type: row.get(2)?,
            marketplace: row.get(3)?,
            my_asin: row.get(4)?,
            product_id: row.get(5)?,
            my_product_info: row.get(6)?,
            my_title: row.get(7)?,
            my_bullets: row.get(8)?,
            my_description: row.get(9)?,
            my_listing_fetched_at: row.get(10)?,
            status: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
            competitor_count: row.get(14)?,
        }))
    } else {
        Ok(None)
    }
}

// 更新项目
pub fn sc_update_project(id: i64, name: &str, my_asin: Option<&str>) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE sc_projects SET name = ?1, my_asin = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        rusqlite::params![name, my_asin, id],
    )?;
    Ok(())
}

// 更新项目状态
pub fn sc_update_project_status(id: i64, status: &str) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE sc_projects SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        rusqlite::params![status, id],
    )?;
    Ok(())
}

// 更新我的产品信息
pub fn sc_update_my_product_info(id: i64, my_product_info: Option<&str>) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE sc_projects SET my_product_info = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        rusqlite::params![my_product_info, id],
    )?;
    Ok(())
}

// 更新用户的 Listing 信息（老品优化时使用）
pub fn sc_update_my_listing(
    id: i64,
    my_title: Option<&str>,
    my_bullets: Option<&str>,
    my_description: Option<&str>,
) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE sc_projects SET my_title = ?1, my_bullets = ?2, my_description = ?3,
         my_listing_fetched_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?4",
        rusqlite::params![my_title, my_bullets, my_description, id],
    )?;
    Ok(())
}

// 删除项目
pub fn sc_delete_project(id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("DELETE FROM sc_projects WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

// ==================== 竞品管理 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScCompetitor {
    pub id: i64,
    pub project_id: i64,
    pub asin: String,
    pub competitor_type: String,  // top, direct, rising
    pub title: Option<String>,
    pub price: Option<String>,
    pub rating: Option<String>,
    pub review_count: Option<i64>,
    pub bsr_rank: Option<String>,
    pub date_first_available: Option<String>,
    pub image_url: Option<String>,
    pub bullets: Option<String>,  // JSON array
    pub description: Option<String>,
    pub fetched_at: Option<String>,
}

// 添加竞品（仅 ASIN）
pub fn sc_add_competitor(project_id: i64, asin: &str, competitor_type: &str) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO sc_competitors (project_id, asin, competitor_type) VALUES (?1, ?2, ?3)",
        rusqlite::params![project_id, asin, competitor_type],
    )?;
    Ok(conn.last_insert_rowid())
}

// 获取项目的竞品列表
pub fn sc_get_competitors(project_id: i64) -> Result<Vec<ScCompetitor>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, asin, competitor_type, title, price, rating, review_count,
                bsr_rank, date_first_available, image_url, bullets, description, fetched_at
         FROM sc_competitors
         WHERE project_id = ?1
         ORDER BY id ASC"
    )?;

    let rows = stmt.query_map(rusqlite::params![project_id], |row| {
        Ok(ScCompetitor {
            id: row.get(0)?,
            project_id: row.get(1)?,
            asin: row.get(2)?,
            competitor_type: row.get(3)?,
            title: row.get(4)?,
            price: row.get(5)?,
            rating: row.get(6)?,
            review_count: row.get(7)?,
            bsr_rank: row.get(8)?,
            date_first_available: row.get(9)?,
            image_url: row.get(10)?,
            bullets: row.get(11)?,
            description: row.get(12)?,
            fetched_at: row.get(13)?,
        })
    })?;

    rows.collect::<Result<Vec<_>>>()
}

// 更新竞品信息（爬取后）
pub fn sc_update_competitor_info(
    id: i64,
    title: Option<&str>,
    price: Option<&str>,
    rating: Option<&str>,
    review_count: Option<i64>,
    bsr_rank: Option<&str>,
    date_first_available: Option<&str>,
    image_url: Option<&str>,
    bullets: Option<&str>,
    description: Option<&str>,
) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE sc_competitors SET
            title = ?1, price = ?2, rating = ?3, review_count = ?4,
            bsr_rank = ?5, date_first_available = ?6, image_url = ?7, bullets = ?8, description = ?9, fetched_at = CURRENT_TIMESTAMP
         WHERE id = ?10",
        rusqlite::params![title, price, rating, review_count, bsr_rank, date_first_available, image_url, bullets, description, id],
    )?;
    Ok(())
}

// 删除竞品
pub fn sc_delete_competitor(id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("DELETE FROM sc_competitors WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

// 更新竞品类型
pub fn sc_update_competitor_type(id: i64, competitor_type: &str) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE sc_competitors SET competitor_type = ?1 WHERE id = ?2",
        rusqlite::params![competitor_type, id],
    )?;
    Ok(())
}

// ==================== 评论管理 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScReview {
    pub id: i64,
    pub competitor_id: i64,
    pub star_rating: i32,
    pub review_text: Option<String>,
    pub review_title: Option<String>,
    pub review_date: Option<String>,
    pub helpful_votes: i32,
}

// 评论输入数据（用于批量添加）
#[derive(Debug, Clone)]
pub struct ReviewInput {
    pub star_rating: i32,
    pub review_text: String,
    pub review_title: Option<String>,
    pub review_date: Option<String>,
    pub helpful_votes: i32,
}

// 批量添加评论
pub fn sc_add_reviews_batch(competitor_id: i64, reviews: &[ReviewInput]) -> Result<i64> {
    let conn = get_db().lock();

    // 先删除该竞品的旧评论
    conn.execute(
        "DELETE FROM sc_reviews WHERE competitor_id = ?1",
        rusqlite::params![competitor_id],
    )?;

    // 批量插入新评论
    let mut stmt = conn.prepare(
        "INSERT INTO sc_reviews (competitor_id, star_rating, review_text, review_date, helpful_votes)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )?;

    let mut count = 0i64;
    for review in reviews {
        // 合并 title 和 text（如果有 title 的话）
        let full_text = if let Some(ref title) = review.review_title {
            if title.is_empty() {
                review.review_text.clone()
            } else {
                format!("{}\n\n{}", title, review.review_text)
            }
        } else {
            review.review_text.clone()
        };

        stmt.execute(rusqlite::params![
            competitor_id,
            review.star_rating,
            full_text,
            review.review_date,
            review.helpful_votes,
        ])?;
        count += 1;
    }

    Ok(count)
}

// 获取竞品的评论列表
pub fn sc_get_reviews(competitor_id: i64) -> Result<Vec<ScReview>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, competitor_id, star_rating, review_text, review_date, helpful_votes
         FROM sc_reviews
         WHERE competitor_id = ?1
         ORDER BY helpful_votes DESC, id ASC"
    )?;

    let rows = stmt.query_map(rusqlite::params![competitor_id], |row| {
        Ok(ScReview {
            id: row.get(0)?,
            competitor_id: row.get(1)?,
            star_rating: row.get(2)?,
            review_text: row.get(3)?,
            review_title: None, // 已合并到 review_text
            review_date: row.get(4)?,
            helpful_votes: row.get(5)?,
        })
    })?;

    let mut reviews = Vec::new();
    for row in rows {
        reviews.push(row?);
    }
    Ok(reviews)
}

// 获取评论统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScReviewSummary {
    pub total: i64,
    pub star_1: i64,
    pub star_2: i64,
    pub star_3: i64,
    pub star_4: i64,
    pub star_5: i64,
}

pub fn sc_get_reviews_summary(competitor_id: i64) -> Result<ScReviewSummary> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT
            COUNT(*) as total,
            SUM(CASE WHEN star_rating = 1 THEN 1 ELSE 0 END) as star_1,
            SUM(CASE WHEN star_rating = 2 THEN 1 ELSE 0 END) as star_2,
            SUM(CASE WHEN star_rating = 3 THEN 1 ELSE 0 END) as star_3,
            SUM(CASE WHEN star_rating = 4 THEN 1 ELSE 0 END) as star_4,
            SUM(CASE WHEN star_rating = 5 THEN 1 ELSE 0 END) as star_5
         FROM sc_reviews
         WHERE competitor_id = ?1"
    )?;

    let summary = stmt.query_row(rusqlite::params![competitor_id], |row| {
        Ok(ScReviewSummary {
            total: row.get(0)?,
            star_1: row.get(1)?,
            star_2: row.get(2)?,
            star_3: row.get(3)?,
            star_4: row.get(4)?,
            star_5: row.get(5)?,
        })
    })?;

    Ok(summary)
}

// 删除竞品的所有评论
pub fn sc_delete_reviews(competitor_id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "DELETE FROM sc_reviews WHERE competitor_id = ?1",
        rusqlite::params![competitor_id],
    )?;
    Ok(())
}

// ==================== AI 分析结果 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScAnalysis {
    pub id: i64,
    pub project_id: i64,
    pub analysis_type: String,  // 'review_insights' | 'listing_analysis' | 'optimization'
    pub result_json: String,
    pub model_provider: Option<String>,
    pub model_name: Option<String>,
    pub created_at: String,
}

// 保存或更新分析结果（如果同类型已存在则覆盖）
pub fn sc_save_analysis(
    project_id: i64,
    analysis_type: &str,
    result_json: &str,
    model_provider: Option<&str>,
    model_name: Option<&str>,
) -> Result<i64> {
    let conn = get_db().lock();

    // 先删除同类型的旧结果
    conn.execute(
        "DELETE FROM sc_analysis_results WHERE project_id = ?1 AND analysis_type = ?2",
        rusqlite::params![project_id, analysis_type],
    )?;

    // 插入新结果
    conn.execute(
        "INSERT INTO sc_analysis_results (project_id, analysis_type, result_json, model_provider, model_name)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![project_id, analysis_type, result_json, model_provider, model_name],
    )?;

    Ok(conn.last_insert_rowid())
}

// 获取指定类型的分析结果
pub fn sc_get_analysis(project_id: i64, analysis_type: &str) -> Result<Option<ScAnalysis>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, analysis_type, result_json, model_provider, model_name, created_at
         FROM sc_analysis_results
         WHERE project_id = ?1 AND analysis_type = ?2
         ORDER BY created_at DESC
         LIMIT 1"
    )?;

    let mut rows = stmt.query(rusqlite::params![project_id, analysis_type])?;

    if let Some(row) = rows.next()? {
        Ok(Some(ScAnalysis {
            id: row.get(0)?,
            project_id: row.get(1)?,
            analysis_type: row.get(2)?,
            result_json: row.get(3)?,
            model_provider: row.get(4)?,
            model_name: row.get(5)?,
            created_at: row.get(6)?,
        }))
    } else {
        Ok(None)
    }
}

// 获取项目的所有分析结果
pub fn sc_get_all_analysis(project_id: i64) -> Result<Vec<ScAnalysis>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, analysis_type, result_json, model_provider, model_name, created_at
         FROM sc_analysis_results
         WHERE project_id = ?1
         ORDER BY created_at DESC"
    )?;

    let rows = stmt.query_map(rusqlite::params![project_id], |row| {
        Ok(ScAnalysis {
            id: row.get(0)?,
            project_id: row.get(1)?,
            analysis_type: row.get(2)?,
            result_json: row.get(3)?,
            model_provider: row.get(4)?,
            model_name: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    rows.collect()
}

// 删除项目的所有分析结果
pub fn sc_delete_all_analysis(project_id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "DELETE FROM sc_analysis_results WHERE project_id = ?1",
        rusqlite::params![project_id],
    )?;
    Ok(())
}

// 获取项目关联的关键词数据（Top N 高搜索量）
pub fn sc_get_project_keywords(project_id: i64, limit: i64) -> Result<Vec<KeywordData>> {
    let conn = get_db().lock();

    // 先获取项目关联的 product_id
    let product_id: Option<i64> = conn.query_row(
        "SELECT product_id FROM sc_projects WHERE id = ?1",
        rusqlite::params![project_id],
        |row| row.get(0),
    )?;

    let product_id = match product_id {
        Some(id) => id,
        None => return Ok(vec![]),  // 未关联产品，返回空
    };

    // 获取关键词数据，按搜索量排序
    let mut stmt = conn.prepare(
        "SELECT id, product_id, keyword, translation, relevance_score, relevance_level,
                traffic_total, avg_keyword_rank, avg_search_volume, cpc_bid, bid_range,
                click_rate, conversion_competition, competition_level, natural_position_flow,
                top3_click_share, avg_conversion_share, asin_count,
                traffic_level, negative_word, orderliness, phrase_tag,
                primary_category, secondary_category, search_intent, traffic_share, asin_data
         FROM keyword_data
         WHERE product_id = ?1
         ORDER BY avg_search_volume DESC NULLS LAST
         LIMIT ?2"
    )?;

    let rows = stmt.query_map(rusqlite::params![product_id, limit], |row| {
        Ok(KeywordData {
            id: row.get(0)?,
            product_id: row.get(1)?,
            keyword: row.get(2)?,
            translation: row.get(3)?,
            relevance_score: row.get(4)?,
            relevance_level: row.get(5)?,
            traffic_total: row.get(6)?,
            avg_keyword_rank: row.get(7)?,
            avg_search_volume: row.get(8)?,
            cpc_bid: row.get(9)?,
            bid_range: row.get(10)?,
            click_rate: row.get(11)?,
            conversion_competition: row.get(12)?,
            competition_level: row.get(13)?,
            natural_position_flow: row.get(14)?,
            top3_click_share: row.get(15)?,
            avg_conversion_share: row.get(16)?,
            asin_count: row.get(17)?,
            traffic_level: row.get(18)?,
            negative_word: row.get(19)?,
            orderliness: row.get(20)?,
            phrase_tag: row.get(21)?,
            primary_category: row.get(22)?,
            secondary_category: row.get(23)?,
            search_intent: row.get(24)?,
            traffic_share: row.get(25)?,
            asin_data: row.get(26)?,
        })
    })?;

    rows.collect()
}

// ==================== 智能广告（Smart Ads）====================

// 广告项目结构体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdProject {
    pub id: i64,
    pub product_id: Option<i64>,
    pub name: String,
    pub marketplace: String,
    pub target_acos: f64,
    pub created_at: String,
    pub updated_at: String,
    pub search_term_count: i64,
}

// 搜索词数据结构体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdSearchTerm {
    pub id: i64,
    pub project_id: i64,
    pub portfolio_name: Option<String>,  // 广告组合名称
    pub campaign_name: Option<String>,   // 广告活动名称
    pub ad_group_name: Option<String>,   // 广告组名称
    pub country: Option<String>,         // 国家/地区
    pub targeting: Option<String>,       // 投放词
    pub match_type: Option<String>,      // 匹配类型
    pub customer_search_term: Option<String>,  // 客户搜索词
    pub impressions: i64,
    pub clicks: i64,
    pub ctr: f64,
    pub spend: f64,
    pub sales: f64,
    pub orders: i64,
    pub acos: f64,
    pub roas: f64,
    pub conversion_rate: f64,
    pub cpc: f64,
    pub report_date: Option<String>,
    pub sku: Option<String>,              // SKU
    pub imported_at: Option<String>,
}

// 广告分析结果结构体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdAnalysisResult {
    pub id: i64,
    pub project_id: i64,
    pub analysis_type: String,
    pub result_json: String,
    pub ai_provider: Option<String>,
    pub ai_model: Option<String>,
    pub created_at: String,
}

// 按国家分组的统计数据
#[derive(Debug, Serialize, Deserialize)]
pub struct CountryStats {
    pub country: String,
    pub total_spend: f64,
    pub total_sales: f64,
    pub avg_acos: f64,
    pub term_count: i64,
}

// 搜索词统计结果（包含总计和按国家分组）
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchTermsStatsResult {
    pub total_spend: f64,
    pub total_sales: f64,
    pub avg_acos: f64,
    pub count: i64,
    pub by_country: Vec<CountryStats>,
}

// 创建广告项目
pub fn ad_create_project(
    product_id: Option<i64>,
    name: &str,
    marketplace: &str,
    target_acos: f64,
) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO ad_projects (product_id, name, marketplace, target_acos) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![product_id, name, marketplace, target_acos],
    )?;
    Ok(conn.last_insert_rowid())
}

// 获取广告项目列表
pub fn ad_get_projects() -> Result<Vec<AdProject>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT p.id, p.product_id, p.name, p.marketplace, p.target_acos, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM ad_search_terms WHERE project_id = p.id) as search_term_count
         FROM ad_projects p
         ORDER BY p.updated_at DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(AdProject {
            id: row.get(0)?,
            product_id: row.get(1)?,
            name: row.get(2)?,
            marketplace: row.get(3)?,
            target_acos: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
            search_term_count: row.get(7)?,
        })
    })?;

    rows.collect()
}

// 获取单个广告项目
pub fn ad_get_project(id: i64) -> Result<Option<AdProject>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT p.id, p.product_id, p.name, p.marketplace, p.target_acos, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM ad_search_terms WHERE project_id = p.id) as search_term_count
         FROM ad_projects p
         WHERE p.id = ?1"
    )?;

    let mut rows = stmt.query_map([id], |row| {
        Ok(AdProject {
            id: row.get(0)?,
            product_id: row.get(1)?,
            name: row.get(2)?,
            marketplace: row.get(3)?,
            target_acos: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
            search_term_count: row.get(7)?,
        })
    })?;

    match rows.next() {
        Some(Ok(project)) => Ok(Some(project)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None),
    }
}

// 更新广告项目
pub fn ad_update_project(id: i64, name: &str, marketplace: &str, target_acos: f64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE ad_projects SET name = ?1, marketplace = ?2, target_acos = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4",
        rusqlite::params![name, marketplace, target_acos, id],
    )?;
    Ok(())
}

// 删除广告项目
pub fn ad_delete_project(id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("DELETE FROM ad_projects WHERE id = ?1", [id])?;
    Ok(())
}

// 导入搜索词数据（批量）
pub fn ad_import_search_terms(project_id: i64, search_terms: Vec<AdSearchTerm>) -> Result<i64> {
    let conn = get_db().lock();

    // 先删除该项目的旧数据
    conn.execute("DELETE FROM ad_search_terms WHERE project_id = ?1", [project_id])?;

    let mut count = 0i64;
    for term in search_terms {
        conn.execute(
            "INSERT INTO ad_search_terms (
                project_id, portfolio_name, campaign_name, ad_group_name, country, targeting, match_type,
                customer_search_term, impressions, clicks, ctr, spend, sales,
                orders, acos, roas, conversion_rate, cpc, report_date, sku
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
            rusqlite::params![
                project_id,
                term.portfolio_name,
                term.campaign_name,
                term.ad_group_name,
                term.country,
                term.targeting,
                term.match_type,
                term.customer_search_term,
                term.impressions,
                term.clicks,
                term.ctr,
                term.spend,
                term.sales,
                term.orders,
                term.acos,
                term.roas,
                term.conversion_rate,
                term.cpc,
                term.report_date,
                term.sku
            ],
        )?;
        count += 1;
    }

    // 更新项目的 updated_at
    conn.execute(
        "UPDATE ad_projects SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        [project_id],
    )?;

    Ok(count)
}

// 获取搜索词数据
pub fn ad_get_search_terms(project_id: i64) -> Result<Vec<AdSearchTerm>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, portfolio_name, campaign_name, ad_group_name, country, targeting, match_type,
                customer_search_term, impressions, clicks, ctr, spend, sales,
                orders, acos, roas, conversion_rate, cpc, report_date, sku, imported_at
         FROM ad_search_terms
         WHERE project_id = ?1
         ORDER BY spend DESC"
    )?;

    let rows = stmt.query_map([project_id], |row| {
        Ok(AdSearchTerm {
            id: row.get(0)?,
            project_id: row.get(1)?,
            portfolio_name: row.get(2)?,
            campaign_name: row.get(3)?,
            ad_group_name: row.get(4)?,
            country: row.get(5)?,
            targeting: row.get(6)?,
            match_type: row.get(7)?,
            customer_search_term: row.get(8)?,
            impressions: row.get(9)?,
            clicks: row.get(10)?,
            ctr: row.get(11)?,
            spend: row.get(12)?,
            sales: row.get(13)?,
            orders: row.get(14)?,
            acos: row.get(15)?,
            roas: row.get(16)?,
            conversion_rate: row.get(17)?,
            cpc: row.get(18)?,
            report_date: row.get(19)?,
            sku: row.get(20)?,
            imported_at: row.get(21)?,
        })
    })?;

    rows.collect()
}

// 获取搜索词统计（包含按国家分组）
pub fn ad_get_search_terms_stats(project_id: i64) -> Result<SearchTermsStatsResult> {
    let conn = get_db().lock();

    // 获取总计
    let mut total_stmt = conn.prepare(
        "SELECT COALESCE(SUM(spend), 0), COALESCE(SUM(sales), 0),
                CASE WHEN SUM(sales) > 0 THEN SUM(spend) / SUM(sales) * 100 ELSE 0 END,
                COUNT(*)
         FROM ad_search_terms WHERE project_id = ?1"
    )?;

    let (total_spend, total_sales, avg_acos, count) = total_stmt.query_row([project_id], |row| {
        Ok((
            row.get::<_, f64>(0)?,
            row.get::<_, f64>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, i64>(3)?,
        ))
    })?;

    // 按国家分组统计
    let mut country_stmt = conn.prepare(
        "SELECT COALESCE(country, 'Unknown') as country,
                COALESCE(SUM(spend), 0) as total_spend,
                COALESCE(SUM(sales), 0) as total_sales,
                CASE WHEN SUM(sales) > 0 THEN SUM(spend) / SUM(sales) * 100 ELSE 0 END as avg_acos,
                COUNT(*) as term_count
         FROM ad_search_terms
         WHERE project_id = ?1
         GROUP BY country
         ORDER BY total_spend DESC"
    )?;

    let by_country: Vec<CountryStats> = country_stmt
        .query_map([project_id], |row| {
            Ok(CountryStats {
                country: row.get(0)?,
                total_spend: row.get(1)?,
                total_sales: row.get(2)?,
                avg_acos: row.get(3)?,
                term_count: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(SearchTermsStatsResult {
        total_spend,
        total_sales,
        avg_acos,
        count,
        by_country,
    })
}

// 保存分析结果
pub fn ad_save_analysis(
    project_id: i64,
    analysis_type: &str,
    result_json: &str,
    ai_provider: &str,
    ai_model: &str,
) -> Result<i64> {
    let conn = get_db().lock();

    // 删除同类型的旧结果
    conn.execute(
        "DELETE FROM ad_analysis_results WHERE project_id = ?1 AND analysis_type = ?2",
        rusqlite::params![project_id, analysis_type],
    )?;

    conn.execute(
        "INSERT INTO ad_analysis_results (project_id, analysis_type, result_json, ai_provider, ai_model)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![project_id, analysis_type, result_json, ai_provider, ai_model],
    )?;

    Ok(conn.last_insert_rowid())
}

// 获取分析结果
pub fn ad_get_analysis(project_id: i64, analysis_type: &str) -> Result<Option<AdAnalysisResult>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, analysis_type, result_json, ai_provider, ai_model, created_at
         FROM ad_analysis_results
         WHERE project_id = ?1 AND analysis_type = ?2
         ORDER BY created_at DESC
         LIMIT 1"
    )?;

    let mut rows = stmt.query_map(rusqlite::params![project_id, analysis_type], |row| {
        Ok(AdAnalysisResult {
            id: row.get(0)?,
            project_id: row.get(1)?,
            analysis_type: row.get(2)?,
            result_json: row.get(3)?,
            ai_provider: row.get(4)?,
            ai_model: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    match rows.next() {
        Some(Ok(result)) => Ok(Some(result)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None),
    }
}

// 获取所有分析结果
pub fn ad_get_all_analysis(project_id: i64) -> Result<Vec<AdAnalysisResult>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, analysis_type, result_json, ai_provider, ai_model, created_at
         FROM ad_analysis_results
         WHERE project_id = ?1
         ORDER BY created_at DESC"
    )?;

    let rows = stmt.query_map([project_id], |row| {
        Ok(AdAnalysisResult {
            id: row.get(0)?,
            project_id: row.get(1)?,
            analysis_type: row.get(2)?,
            result_json: row.get(3)?,
            ai_provider: row.get(4)?,
            ai_model: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    rows.collect()
}

// ============ 市场调研监控表初始化 ============

fn init_market_research_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- BSR 历史快照表
        CREATE TABLE IF NOT EXISTS bsr_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            marketplace TEXT NOT NULL,
            category_id TEXT NOT NULL,
            category_name TEXT,
            snapshot_date DATE NOT NULL,
            products_json TEXT NOT NULL,
            product_count INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(marketplace, category_id, snapshot_date)
        );
        CREATE INDEX IF NOT EXISTS idx_bsr_snapshots_lookup
            ON bsr_snapshots(marketplace, category_id, snapshot_date);

        -- 市场调研监控任务表
        CREATE TABLE IF NOT EXISTS market_research_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            marketplace TEXT NOT NULL,
            category_id TEXT NOT NULL,
            category_name TEXT,
            ai_provider TEXT NOT NULL,
            ai_model TEXT,
            schedule_type TEXT NOT NULL,
            schedule_days TEXT,
            schedule_time TEXT NOT NULL,
            is_enabled INTEGER DEFAULT 1,
            last_run_at TIMESTAMP,
            last_run_status TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 任务执行记录表
        CREATE TABLE IF NOT EXISTS market_research_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            started_at TIMESTAMP NOT NULL,
            ended_at TIMESTAMP,
            status TEXT DEFAULT 'running',
            report_summary TEXT,
            report_content TEXT,
            snapshot_id INTEGER,
            error_message TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES market_research_tasks(id) ON DELETE CASCADE,
            FOREIGN KEY (snapshot_id) REFERENCES bsr_snapshots(id)
        );
        CREATE INDEX IF NOT EXISTS idx_research_runs_task ON market_research_runs(task_id);
        "
    )?;

    // Migration: add ai_model column if not exists (for existing databases)
    let has_ai_model = conn
        .prepare("SELECT ai_model FROM market_research_tasks LIMIT 1")
        .is_ok();
    if !has_ai_model {
        let _ = conn.execute(
            "ALTER TABLE market_research_tasks ADD COLUMN ai_model TEXT",
            [],
        );
    }

    Ok(())
}

// ============ BSR 快照 CRUD ============

pub fn save_bsr_snapshot(
    marketplace: &str,
    category_id: &str,
    category_name: Option<&str>,
    products_json: &str,
    product_count: i64,
) -> Result<i64> {
    let conn = get_db().lock();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    conn.execute(
        "INSERT OR REPLACE INTO bsr_snapshots
         (marketplace, category_id, category_name, snapshot_date, products_json, product_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![marketplace, category_id, category_name, today, products_json, product_count],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_bsr_snapshot(marketplace: &str, category_id: &str, date: &str) -> Result<Option<BsrSnapshot>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, marketplace, category_id, category_name, snapshot_date, products_json, product_count, created_at
         FROM bsr_snapshots
         WHERE marketplace = ?1 AND category_id = ?2 AND snapshot_date = ?3"
    )?;

    let mut rows = stmt.query_map(rusqlite::params![marketplace, category_id, date], |row| {
        Ok(BsrSnapshot {
            id: row.get(0)?,
            marketplace: row.get(1)?,
            category_id: row.get(2)?,
            category_name: row.get(3)?,
            snapshot_date: row.get(4)?,
            products_json: row.get(5)?,
            product_count: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;

    match rows.next() {
        Some(Ok(snapshot)) => Ok(Some(snapshot)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None),
    }
}

pub fn get_bsr_history(marketplace: &str, category_id: &str, days: i32) -> Result<Vec<BsrSnapshot>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, marketplace, category_id, category_name, snapshot_date, products_json, product_count, created_at
         FROM bsr_snapshots
         WHERE marketplace = ?1 AND category_id = ?2
         AND snapshot_date >= date('now', ?3)
         ORDER BY snapshot_date DESC"
    )?;

    let days_param = format!("-{} days", days);
    let rows = stmt.query_map(rusqlite::params![marketplace, category_id, days_param], |row| {
        Ok(BsrSnapshot {
            id: row.get(0)?,
            marketplace: row.get(1)?,
            category_id: row.get(2)?,
            category_name: row.get(3)?,
            snapshot_date: row.get(4)?,
            products_json: row.get(5)?,
            product_count: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;

    rows.collect()
}

// ============ 市场调研任务 CRUD ============

pub fn create_market_research_task(
    name: &str,
    marketplace: &str,
    category_id: &str,
    category_name: Option<&str>,
    ai_provider: &str,
    ai_model: Option<&str>,
    schedule_type: &str,
    schedule_days: Option<&str>,
    schedule_time: &str,
) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO market_research_tasks
         (name, marketplace, category_id, category_name, ai_provider, ai_model, schedule_type, schedule_days, schedule_time)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![name, marketplace, category_id, category_name, ai_provider, ai_model, schedule_type, schedule_days, schedule_time],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_market_research_tasks() -> Result<Vec<MarketResearchTask>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, marketplace, category_id, category_name, ai_provider, ai_model,
                schedule_type, schedule_days, schedule_time, is_enabled,
                last_run_at, last_run_status, created_at
         FROM market_research_tasks
         ORDER BY created_at DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(MarketResearchTask {
            id: row.get(0)?,
            name: row.get(1)?,
            marketplace: row.get(2)?,
            category_id: row.get(3)?,
            category_name: row.get(4)?,
            ai_provider: row.get(5)?,
            ai_model: row.get(6)?,
            schedule_type: row.get(7)?,
            schedule_days: row.get(8)?,
            schedule_time: row.get(9)?,
            is_enabled: row.get::<_, i64>(10)? != 0,
            last_run_at: row.get(11)?,
            last_run_status: row.get(12)?,
            created_at: row.get(13)?,
        })
    })?;

    rows.collect()
}

pub fn get_market_research_task(id: i64) -> Result<Option<MarketResearchTask>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, marketplace, category_id, category_name, ai_provider, ai_model,
                schedule_type, schedule_days, schedule_time, is_enabled,
                last_run_at, last_run_status, created_at
         FROM market_research_tasks
         WHERE id = ?1"
    )?;

    let mut rows = stmt.query_map([id], |row| {
        Ok(MarketResearchTask {
            id: row.get(0)?,
            name: row.get(1)?,
            marketplace: row.get(2)?,
            category_id: row.get(3)?,
            category_name: row.get(4)?,
            ai_provider: row.get(5)?,
            ai_model: row.get(6)?,
            schedule_type: row.get(7)?,
            schedule_days: row.get(8)?,
            schedule_time: row.get(9)?,
            is_enabled: row.get::<_, i64>(10)? != 0,
            last_run_at: row.get(11)?,
            last_run_status: row.get(12)?,
            created_at: row.get(13)?,
        })
    })?;

    match rows.next() {
        Some(Ok(task)) => Ok(Some(task)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None),
    }
}

pub fn update_market_research_task(
    id: i64,
    name: &str,
    marketplace: &str,
    category_id: &str,
    category_name: Option<&str>,
    ai_provider: &str,
    ai_model: Option<&str>,
    schedule_type: &str,
    schedule_days: Option<&str>,
    schedule_time: &str,
    is_enabled: bool,
) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE market_research_tasks
         SET name = ?1, marketplace = ?2, category_id = ?3, category_name = ?4,
             ai_provider = ?5, ai_model = ?6, schedule_type = ?7, schedule_days = ?8,
             schedule_time = ?9, is_enabled = ?10
         WHERE id = ?11",
        rusqlite::params![
            name, marketplace, category_id, category_name, ai_provider, ai_model,
            schedule_type, schedule_days, schedule_time, is_enabled as i64, id
        ],
    )?;
    Ok(())
}

pub fn delete_market_research_task(id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("DELETE FROM market_research_tasks WHERE id = ?1", [id])?;
    Ok(())
}

pub fn update_task_last_run(id: i64, status: &str) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE market_research_tasks
         SET last_run_at = datetime('now'), last_run_status = ?1
         WHERE id = ?2",
        rusqlite::params![status, id],
    )?;
    Ok(())
}

pub fn get_pending_research_tasks() -> Result<Vec<MarketResearchTask>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, marketplace, category_id, category_name, ai_provider, ai_model,
                schedule_type, schedule_days, schedule_time, is_enabled,
                last_run_at, last_run_status, created_at
         FROM market_research_tasks
         WHERE is_enabled = 1
         ORDER BY created_at DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(MarketResearchTask {
            id: row.get(0)?,
            name: row.get(1)?,
            marketplace: row.get(2)?,
            category_id: row.get(3)?,
            category_name: row.get(4)?,
            ai_provider: row.get(5)?,
            ai_model: row.get(6)?,
            schedule_type: row.get(7)?,
            schedule_days: row.get(8)?,
            schedule_time: row.get(9)?,
            is_enabled: row.get::<_, i64>(10)? != 0,
            last_run_at: row.get(11)?,
            last_run_status: row.get(12)?,
            created_at: row.get(13)?,
        })
    })?;

    rows.collect()
}

// ============ 执行记录 CRUD ============

pub fn create_research_run(task_id: i64) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO market_research_runs (task_id, started_at, status)
         VALUES (?1, datetime('now'), 'running')",
        [task_id],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_research_run(
    run_id: i64,
    status: &str,
    summary: Option<&str>,
    content: Option<&str>,
    snapshot_id: Option<i64>,
) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE market_research_runs
         SET ended_at = datetime('now'), status = ?1,
             report_summary = ?2, report_content = ?3, snapshot_id = ?4
         WHERE id = ?5",
        rusqlite::params![status, summary, content, snapshot_id, run_id],
    )?;
    Ok(())
}

pub fn fail_research_run(run_id: i64, error_message: &str) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE market_research_runs
         SET ended_at = datetime('now'), status = 'failed', error_message = ?1
         WHERE id = ?2",
        rusqlite::params![error_message, run_id],
    )?;
    Ok(())
}

pub fn get_latest_research_runs(limit: i32) -> Result<Vec<MarketResearchRun>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, task_id, started_at, ended_at, status,
                report_summary, report_content, snapshot_id, error_message, created_at
         FROM market_research_runs
         ORDER BY started_at DESC
         LIMIT ?1"
    )?;

    let rows = stmt.query_map([limit], |row| {
        Ok(MarketResearchRun {
            id: row.get(0)?,
            task_id: row.get(1)?,
            started_at: row.get(2)?,
            ended_at: row.get(3)?,
            status: row.get(4)?,
            report_summary: row.get(5)?,
            report_content: row.get(6)?,
            snapshot_id: row.get(7)?,
            error_message: row.get(8)?,
            created_at: row.get(9)?,
        })
    })?;

    rows.collect()
}

pub fn get_research_runs_by_task(task_id: i64, limit: i32) -> Result<Vec<MarketResearchRun>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, task_id, started_at, ended_at, status,
                report_summary, report_content, snapshot_id, error_message, created_at
         FROM market_research_runs
         WHERE task_id = ?1
         ORDER BY started_at DESC
         LIMIT ?2"
    )?;

    let rows = stmt.query_map(rusqlite::params![task_id, limit], |row| {
        Ok(MarketResearchRun {
            id: row.get(0)?,
            task_id: row.get(1)?,
            started_at: row.get(2)?,
            ended_at: row.get(3)?,
            status: row.get(4)?,
            report_summary: row.get(5)?,
            report_content: row.get(6)?,
            snapshot_id: row.get(7)?,
            error_message: row.get(8)?,
            created_at: row.get(9)?,
        })
    })?;

    rows.collect()
}

// ============ 竞品情报监控表初始化 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorTask {
    pub id: i64,
    pub name: String,
    pub marketplace: String,
    pub my_asin: Option<String>,
    pub ai_provider: String,
    pub ai_model: Option<String>,
    pub schedule_type: String,
    pub schedule_days: Option<String>,
    pub schedule_time: String,
    pub is_enabled: bool,
    pub last_run_at: Option<String>,
    pub last_run_status: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorAsin {
    pub id: i64,
    pub task_id: i64,
    pub asin: String,
    pub title: Option<String>,
    pub tags: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorSnapshot {
    pub id: i64,
    pub asin_id: i64,
    pub snapshot_date: String,
    pub price: Option<f64>,
    pub bsr_rank: Option<i64>,
    pub rating: Option<f64>,
    pub review_count: Option<i64>,
    pub availability: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorRun {
    pub id: i64,
    pub task_id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,
    pub report_summary: Option<String>,
    pub report_content: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
}

fn init_competitor_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- 竞品监控任务表
        CREATE TABLE IF NOT EXISTS competitor_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            marketplace TEXT NOT NULL,
            my_asin TEXT,
            ai_provider TEXT NOT NULL DEFAULT 'deepseek',
            ai_model TEXT,
            schedule_type TEXT NOT NULL DEFAULT 'daily',
            schedule_days TEXT,
            schedule_time TEXT NOT NULL DEFAULT '09:00',
            is_enabled INTEGER DEFAULT 1,
            last_run_at TIMESTAMP,
            last_run_status TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- 监控的竞品 ASIN 表
        CREATE TABLE IF NOT EXISTS competitor_asins (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            asin TEXT NOT NULL,
            title TEXT,
            tags TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES competitor_tasks(id) ON DELETE CASCADE,
            UNIQUE(task_id, asin)
        );
        CREATE INDEX IF NOT EXISTS idx_competitor_asins_task ON competitor_asins(task_id);

        -- 竞品快照表（每日数据）
        CREATE TABLE IF NOT EXISTS competitor_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            asin_id INTEGER NOT NULL,
            snapshot_date TEXT NOT NULL,
            price REAL,
            bsr_rank INTEGER,
            rating REAL,
            review_count INTEGER,
            availability TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (asin_id) REFERENCES competitor_asins(id) ON DELETE CASCADE,
            UNIQUE(asin_id, snapshot_date)
        );
        CREATE INDEX IF NOT EXISTS idx_competitor_snapshots_asin ON competitor_snapshots(asin_id);
        CREATE INDEX IF NOT EXISTS idx_competitor_snapshots_date ON competitor_snapshots(snapshot_date);

        -- 竞品监控执行记录表
        CREATE TABLE IF NOT EXISTS competitor_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            started_at TIMESTAMP NOT NULL,
            ended_at TIMESTAMP,
            status TEXT DEFAULT 'running',
            report_summary TEXT,
            report_content TEXT,
            error_message TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES competitor_tasks(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_competitor_runs_task ON competitor_runs(task_id);
        "
    )?;

    // 迁移：给 competitor_tasks 添加 schedule_days 字段（用于现有数据库）
    let _ = conn.execute("ALTER TABLE competitor_tasks ADD COLUMN schedule_days TEXT", []);

    Ok(())
}

// ============ 竞品监控任务 CRUD ============

pub fn create_competitor_task(
    name: &str,
    marketplace: &str,
    my_asin: Option<&str>,
    ai_provider: &str,
    ai_model: Option<&str>,
    schedule_type: &str,
    schedule_days: Option<&str>,
    schedule_time: &str,
) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO competitor_tasks (name, marketplace, my_asin, ai_provider, ai_model, schedule_type, schedule_days, schedule_time)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![name, marketplace, my_asin, ai_provider, ai_model, schedule_type, schedule_days, schedule_time],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_competitor_task(
    id: i64,
    name: &str,
    marketplace: &str,
    my_asin: Option<&str>,
    ai_provider: &str,
    ai_model: Option<&str>,
    schedule_type: &str,
    schedule_days: Option<&str>,
    schedule_time: &str,
    is_enabled: bool,
) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE competitor_tasks
         SET name = ?2, marketplace = ?3, my_asin = ?4, ai_provider = ?5, ai_model = ?6,
             schedule_type = ?7, schedule_days = ?8, schedule_time = ?9, is_enabled = ?10
         WHERE id = ?1",
        rusqlite::params![id, name, marketplace, my_asin, ai_provider, ai_model, schedule_type, schedule_days, schedule_time, is_enabled],
    )?;
    Ok(())
}

pub fn delete_competitor_task(id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("DELETE FROM competitor_tasks WHERE id = ?1", [id])?;
    Ok(())
}

pub fn get_competitor_tasks() -> Result<Vec<CompetitorTask>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, marketplace, my_asin, ai_provider, ai_model, schedule_type, schedule_days, schedule_time,
                is_enabled, last_run_at, last_run_status, created_at
         FROM competitor_tasks
         ORDER BY created_at DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(CompetitorTask {
            id: row.get(0)?,
            name: row.get(1)?,
            marketplace: row.get(2)?,
            my_asin: row.get(3)?,
            ai_provider: row.get(4)?,
            ai_model: row.get(5)?,
            schedule_type: row.get(6)?,
            schedule_days: row.get(7)?,
            schedule_time: row.get(8)?,
            is_enabled: row.get(9)?,
            last_run_at: row.get(10)?,
            last_run_status: row.get(11)?,
            created_at: row.get(12)?,
        })
    })?;

    rows.collect()
}

pub fn get_competitor_task(id: i64) -> Result<Option<CompetitorTask>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, marketplace, my_asin, ai_provider, ai_model, schedule_type, schedule_days, schedule_time,
                is_enabled, last_run_at, last_run_status, created_at
         FROM competitor_tasks WHERE id = ?1"
    )?;

    let mut rows = stmt.query_map([id], |row| {
        Ok(CompetitorTask {
            id: row.get(0)?,
            name: row.get(1)?,
            marketplace: row.get(2)?,
            my_asin: row.get(3)?,
            ai_provider: row.get(4)?,
            ai_model: row.get(5)?,
            schedule_type: row.get(6)?,
            schedule_days: row.get(7)?,
            schedule_time: row.get(8)?,
            is_enabled: row.get(9)?,
            last_run_at: row.get(10)?,
            last_run_status: row.get(11)?,
            created_at: row.get(12)?,
        })
    })?;

    rows.next().transpose()
}

pub fn get_enabled_competitor_tasks() -> Result<Vec<CompetitorTask>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, marketplace, my_asin, ai_provider, ai_model, schedule_type, schedule_days, schedule_time,
                is_enabled, last_run_at, last_run_status, created_at
         FROM competitor_tasks
         WHERE is_enabled = 1
         ORDER BY created_at DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(CompetitorTask {
            id: row.get(0)?,
            name: row.get(1)?,
            marketplace: row.get(2)?,
            my_asin: row.get(3)?,
            ai_provider: row.get(4)?,
            ai_model: row.get(5)?,
            schedule_type: row.get(6)?,
            schedule_days: row.get(7)?,
            schedule_time: row.get(8)?,
            is_enabled: row.get(9)?,
            last_run_at: row.get(10)?,
            last_run_status: row.get(11)?,
            created_at: row.get(12)?,
        })
    })?;

    rows.collect()
}

pub fn update_competitor_task_status(id: i64, status: &str) -> Result<()> {
    let conn = get_db().lock();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE competitor_tasks SET last_run_at = ?2, last_run_status = ?3 WHERE id = ?1",
        rusqlite::params![id, now, status],
    )?;
    Ok(())
}

// ============ 竞品 ASIN CRUD ============

pub fn add_competitor_asin(task_id: i64, asin: &str, title: Option<&str>, tags: Option<&str>) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT OR REPLACE INTO competitor_asins (task_id, asin, title, tags) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![task_id, asin, title, tags],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn remove_competitor_asin(task_id: i64, asin: &str) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "DELETE FROM competitor_asins WHERE task_id = ?1 AND asin = ?2",
        rusqlite::params![task_id, asin],
    )?;
    Ok(())
}

pub fn get_competitor_asins(task_id: i64) -> Result<Vec<CompetitorAsin>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, task_id, asin, title, tags, created_at
         FROM competitor_asins WHERE task_id = ?1 ORDER BY created_at"
    )?;

    let rows = stmt.query_map([task_id], |row| {
        Ok(CompetitorAsin {
            id: row.get(0)?,
            task_id: row.get(1)?,
            asin: row.get(2)?,
            title: row.get(3)?,
            tags: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    rows.collect()
}

// ============ 竞品快照 CRUD ============

pub fn save_competitor_snapshot(
    asin_id: i64,
    price: Option<f64>,
    bsr_rank: Option<i64>,
    rating: Option<f64>,
    review_count: Option<i64>,
    availability: Option<&str>,
) -> Result<i64> {
    let conn = get_db().lock();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    conn.execute(
        "INSERT OR REPLACE INTO competitor_snapshots
         (asin_id, snapshot_date, price, bsr_rank, rating, review_count, availability)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![asin_id, today, price, bsr_rank, rating, review_count, availability],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_competitor_snapshots(asin_id: i64, limit: i32) -> Result<Vec<CompetitorSnapshot>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, asin_id, snapshot_date, price, bsr_rank, rating, review_count, availability, created_at
         FROM competitor_snapshots
         WHERE asin_id = ?1
         ORDER BY snapshot_date DESC
         LIMIT ?2"
    )?;

    let rows = stmt.query_map(rusqlite::params![asin_id, limit], |row| {
        Ok(CompetitorSnapshot {
            id: row.get(0)?,
            asin_id: row.get(1)?,
            snapshot_date: row.get(2)?,
            price: row.get(3)?,
            bsr_rank: row.get(4)?,
            rating: row.get(5)?,
            review_count: row.get(6)?,
            availability: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;

    rows.collect()
}

// ============ 竞品执行记录 CRUD ============

pub fn create_competitor_run(task_id: i64) -> Result<i64> {
    let conn = get_db().lock();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO competitor_runs (task_id, started_at, status) VALUES (?1, ?2, 'running')",
        rusqlite::params![task_id, now],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_competitor_run(
    run_id: i64,
    status: &str,
    report_summary: Option<&str>,
    report_content: Option<&str>,
) -> Result<()> {
    let conn = get_db().lock();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // 获取 task_id
    let task_id: i64 = conn.query_row(
        "SELECT task_id FROM competitor_runs WHERE id = ?1",
        [run_id],
        |row| row.get(0),
    )?;

    // 更新 run 记录
    conn.execute(
        "UPDATE competitor_runs
         SET ended_at = ?2, status = ?3, report_summary = ?4, report_content = ?5
         WHERE id = ?1",
        rusqlite::params![run_id, now, status, report_summary, report_content],
    )?;

    // 同时更新 task 的 last_run_at 和 last_run_status
    conn.execute(
        "UPDATE competitor_tasks SET last_run_at = ?2, last_run_status = ?3 WHERE id = ?1",
        rusqlite::params![task_id, now, status],
    )?;

    Ok(())
}

pub fn fail_competitor_run(run_id: i64, error_message: &str) -> Result<()> {
    let conn = get_db().lock();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // 获取 task_id
    let task_id: i64 = conn.query_row(
        "SELECT task_id FROM competitor_runs WHERE id = ?1",
        [run_id],
        |row| row.get(0),
    )?;

    // 更新 run 记录
    conn.execute(
        "UPDATE competitor_runs SET ended_at = ?2, status = 'failed', error_message = ?3 WHERE id = ?1",
        rusqlite::params![run_id, now, error_message],
    )?;

    // 同时更新 task 的 last_run_at 和 last_run_status
    conn.execute(
        "UPDATE competitor_tasks SET last_run_at = ?2, last_run_status = 'failed' WHERE id = ?1",
        rusqlite::params![task_id, now],
    )?;

    Ok(())
}

pub fn get_latest_competitor_runs(limit: i32) -> Result<Vec<CompetitorRun>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, task_id, started_at, ended_at, status, report_summary, report_content, error_message, created_at
         FROM competitor_runs
         ORDER BY started_at DESC
         LIMIT ?1"
    )?;

    let rows = stmt.query_map([limit], |row| {
        Ok(CompetitorRun {
            id: row.get(0)?,
            task_id: row.get(1)?,
            started_at: row.get(2)?,
            ended_at: row.get(3)?,
            status: row.get(4)?,
            report_summary: row.get(5)?,
            report_content: row.get(6)?,
            error_message: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;

    rows.collect()
}

pub fn get_competitor_runs_by_task(task_id: i64, limit: i32) -> Result<Vec<CompetitorRun>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, task_id, started_at, ended_at, status, report_summary, report_content, error_message, created_at
         FROM competitor_runs
         WHERE task_id = ?1
         ORDER BY started_at DESC
         LIMIT ?2"
    )?;

    let rows = stmt.query_map(rusqlite::params![task_id, limit], |row| {
        Ok(CompetitorRun {
            id: row.get(0)?,
            task_id: row.get(1)?,
            started_at: row.get(2)?,
            ended_at: row.get(3)?,
            status: row.get(4)?,
            report_summary: row.get(5)?,
            report_content: row.get(6)?,
            error_message: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;

    rows.collect()
}

// ==================== 快捷备忘录 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuickNote {
    pub id: i64,
    pub content: String,
    pub completed: bool,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub due_date: Option<String>,
    pub sort_order: i64,
    pub repeat_type: Option<String>,    // 'daily' | 'weekly' | 'monthly' | null
    pub repeat_interval: i64,           // 间隔数，默认1
}

pub fn init_quick_notes_table(conn: &Connection) -> Result<()> {
    // 创建表（如果不存在）
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS quick_notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            completed INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            completed_at DATETIME,
            due_date TEXT,
            sort_order INTEGER DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_quick_notes_completed ON quick_notes(completed);
        CREATE INDEX IF NOT EXISTS idx_quick_notes_created ON quick_notes(created_at);
        "
    )?;

    // 迁移：为现有表添加新列
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(quick_notes)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !columns.contains(&"due_date".to_string()) {
        conn.execute("ALTER TABLE quick_notes ADD COLUMN due_date TEXT", [])?;
    }
    if !columns.contains(&"sort_order".to_string()) {
        conn.execute("ALTER TABLE quick_notes ADD COLUMN sort_order INTEGER DEFAULT 0", [])?;
    }
    // 重复任务字段
    if !columns.contains(&"repeat_type".to_string()) {
        conn.execute("ALTER TABLE quick_notes ADD COLUMN repeat_type TEXT", [])?;
    }
    if !columns.contains(&"repeat_interval".to_string()) {
        conn.execute("ALTER TABLE quick_notes ADD COLUMN repeat_interval INTEGER DEFAULT 1", [])?;
    }

    // 迁移后创建 sort_order 索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_quick_notes_sort_order ON quick_notes(sort_order)",
        [],
    )?;

    Ok(())
}

pub fn add_quick_note(content: String) -> Result<i64> {
    let conn = get_db().lock();
    // 获取当前最大的 sort_order
    let max_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM quick_notes",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // 使用本地时间（北京时间）而不是 UTC
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO quick_notes (content, sort_order, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![content, max_order + 1, now],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_quick_notes(filter: Option<String>) -> Result<Vec<QuickNote>> {
    let conn = get_db().lock();

    let sql = match filter.as_deref() {
        Some("pending") => "SELECT id, content, completed, created_at, completed_at, due_date, sort_order, repeat_type, repeat_interval FROM quick_notes WHERE completed = 0 ORDER BY sort_order ASC, created_at DESC",
        Some("completed") => "SELECT id, content, completed, created_at, completed_at, due_date, sort_order, repeat_type, repeat_interval FROM quick_notes WHERE completed = 1 ORDER BY completed_at DESC",
        _ => "SELECT id, content, completed, created_at, completed_at, due_date, sort_order, repeat_type, repeat_interval FROM quick_notes ORDER BY completed ASC, sort_order ASC, created_at DESC",
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(QuickNote {
            id: row.get(0)?,
            content: row.get(1)?,
            completed: row.get::<_, i32>(2)? == 1,
            created_at: row.get(3)?,
            completed_at: row.get(4)?,
            due_date: row.get(5)?,
            sort_order: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
            repeat_type: row.get(7)?,
            repeat_interval: row.get::<_, Option<i64>>(8)?.unwrap_or(1),
        })
    })?;

    rows.collect()
}

pub fn update_quick_note(id: i64, content: String) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE quick_notes SET content = ?2 WHERE id = ?1",
        rusqlite::params![id, content],
    )?;
    Ok(())
}

pub fn toggle_quick_note(id: i64) -> Result<bool> {
    let conn = get_db().lock();

    // 获取当前状态和重复设置
    let (current, content, due_date, repeat_type, repeat_interval, sort_order): (i32, String, Option<String>, Option<String>, Option<i64>, i64) = conn.query_row(
        "SELECT completed, content, due_date, repeat_type, repeat_interval, sort_order FROM quick_notes WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get::<_, Option<i64>>(5)?.unwrap_or(0))),
    )?;

    let new_completed = current == 0;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if new_completed {
        conn.execute(
            "UPDATE quick_notes SET completed = 1, completed_at = ?2 WHERE id = ?1",
            rusqlite::params![id, now],
        )?;

        // 如果是重复任务，自动创建下一个
        if let (Some(ref rtype), Some(ref ddate)) = (&repeat_type, &due_date) {
            let interval = repeat_interval.unwrap_or(1) as i64;
            if let Some(next_due) = calculate_next_due_date(ddate, rtype, interval) {
                let created_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                conn.execute(
                    "INSERT INTO quick_notes (content, due_date, repeat_type, repeat_interval, sort_order, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![content, next_due, rtype, interval, sort_order, created_at],
                )?;
            }
        }
    } else {
        conn.execute(
            "UPDATE quick_notes SET completed = 0, completed_at = NULL WHERE id = ?1",
            rusqlite::params![id],
        )?;
    }

    Ok(new_completed)
}

// 计算下一个截止日期
fn calculate_next_due_date(current_due: &str, repeat_type: &str, interval: i64) -> Option<String> {
    use chrono::{NaiveDate, Duration, Datelike, Weekday};

    let date = NaiveDate::parse_from_str(current_due, "%Y-%m-%d").ok()?;

    let next_date = match repeat_type {
        "daily" => date + Duration::days(interval),
        "weekly" => {
            // 找到下一个周一（至少 1 天后）
            let mut next = date + Duration::days(1);
            while next.weekday() != Weekday::Mon {
                next = next + Duration::days(1);
            }
            // 如果 interval > 1，再加 (interval-1) 周
            if interval > 1 {
                next = next + Duration::weeks(interval - 1);
            }
            next
        },
        "monthly" => {
            // 找到下一个月的 1 号
            let (year, month) = if date.month() == 12 {
                (date.year() + 1, 1)
            } else {
                (date.year(), date.month() + 1)
            };
            let mut next = NaiveDate::from_ymd_opt(year, month, 1)?;
            // 如果 interval > 1，再加 (interval-1) 月
            if interval > 1 {
                for _ in 1..interval {
                    let (y, m) = if next.month() == 12 {
                        (next.year() + 1, 1)
                    } else {
                        (next.year(), next.month() + 1)
                    };
                    next = NaiveDate::from_ymd_opt(y, m, 1)?;
                }
            }
            next
        },
        _ => return None,
    };

    Some(next_date.format("%Y-%m-%d").to_string())
}

pub fn delete_quick_note(id: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute("DELETE FROM quick_notes WHERE id = ?1", [id])?;
    Ok(())
}

pub fn get_quick_notes_count() -> Result<(i64, i64)> {
    let conn = get_db().lock();
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM quick_notes",
        [],
        |row| row.get(0),
    )?;
    let pending: i64 = conn.query_row(
        "SELECT COUNT(*) FROM quick_notes WHERE completed = 0",
        [],
        |row| row.get(0),
    )?;
    Ok((total, pending))
}

pub fn update_quick_note_due_date(id: i64, due_date: Option<String>) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE quick_notes SET due_date = ?2 WHERE id = ?1",
        rusqlite::params![id, due_date],
    )?;
    Ok(())
}

pub fn update_quick_note_repeat(id: i64, repeat_type: Option<String>, repeat_interval: i64) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE quick_notes SET repeat_type = ?2, repeat_interval = ?3 WHERE id = ?1",
        rusqlite::params![id, repeat_type, repeat_interval],
    )?;
    Ok(())
}

pub fn reorder_quick_notes(ids: Vec<i64>) -> Result<()> {
    let conn = get_db().lock();
    for (index, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE quick_notes SET sort_order = ?2 WHERE id = ?1",
            rusqlite::params![id, index as i64],
        )?;
    }
    Ok(())
}

// ==================== 汇率缓存 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExchangeRateCache {
    pub currency: String,
    pub rate: f64,
    pub updated_at: String,
}

pub fn init_exchange_rate_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS exchange_rates (
            currency TEXT PRIMARY KEY,
            rate REAL NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "
    )?;
    Ok(())
}

pub fn save_exchange_rates(rates: Vec<(String, f64)>) -> Result<()> {
    let conn = get_db().lock();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    for (currency, rate) in rates {
        conn.execute(
            "INSERT OR REPLACE INTO exchange_rates (currency, rate, updated_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![currency, rate, now],
        )?;
    }

    Ok(())
}

pub fn get_exchange_rates() -> Result<Vec<ExchangeRateCache>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT currency, rate, updated_at FROM exchange_rates"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(ExchangeRateCache {
            currency: row.get(0)?,
            rate: row.get(1)?,
            updated_at: row.get(2)?,
        })
    })?;

    rows.collect()
}
