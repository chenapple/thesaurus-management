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

        // 插入历史记录
        conn.execute(
            "INSERT INTO keyword_ranking_history
                (monitoring_id, check_date, organic_rank, organic_page, sponsored_rank, sponsored_page)
             VALUES (?1, date('now'), ?2, ?3, ?4, ?5)",
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

// 获取排名历史
pub fn get_ranking_history(monitoring_id: i64, days: i64) -> Result<Vec<RankingHistory>> {
    let conn = get_db().lock();

    let mut stmt = conn.prepare(
        "SELECT id, monitoring_id, check_date, organic_rank, organic_page,
                sponsored_rank, sponsored_page, checked_at
         FROM keyword_ranking_history
         WHERE monitoring_id = ?1 AND check_date >= date('now', ?2)
         ORDER BY check_date ASC"
    )?;

    let days_str = format!("-{} days", days);
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
         VALUES (?1, ?2, date('now'), ?3, ?4)",
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
         WHERE keyword = ?1 AND country = ?2 AND snapshot_date >= date('now', ?3)
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

// 批量获取监控项的迷你图数据（最近N天的排名）
pub fn get_monitoring_sparklines(product_id: i64, days: i64) -> Result<Vec<MonitoringSparkline>> {
    let conn = get_db().lock();
    let days_str = format!("-{} days", days);

    // 获取该产品所有监控项在最近N天的排名历史（包含自然排名和广告排名）
    let mut stmt = conn.prepare(
        "SELECT h.monitoring_id, h.check_date, h.organic_rank, h.sponsored_rank
         FROM keyword_ranking_history h
         JOIN keyword_monitoring m ON h.monitoring_id = m.id
         WHERE m.product_id = ?1 AND h.check_date >= date('now', ?2)
         ORDER BY h.monitoring_id, h.check_date ASC"
    )?;

    let rows = stmt.query_map(rusqlite::params![product_id, days_str], |row| {
        Ok((
            row.get::<_, i64>(0)?,         // monitoring_id
            row.get::<_, String>(1)?,      // check_date
            row.get::<_, Option<i64>>(2)?, // organic_rank
            row.get::<_, Option<i64>>(3)?, // sponsored_rank
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

    Ok(())
}

// ==================== 知识库分类 CRUD ====================

// 创建知识库分类
pub fn kb_create_category(name: String, parent_id: Option<i64>) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO kb_categories (name, parent_id) VALUES (?1, ?2)",
        rusqlite::params![name, parent_id],
    )?;
    Ok(conn.last_insert_rowid())
}

// 获取所有知识库分类
pub fn kb_get_categories() -> Result<Vec<KbCategory>> {
    let conn = get_db().lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, parent_id, created_at FROM kb_categories ORDER BY name"
    )?;

    let categories = stmt.query_map([], |row| {
        Ok(KbCategory {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            created_at: row.get(3)?,
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
        "SELECT id, document_id, chunk_index, content, page_number
         FROM kb_chunks WHERE document_id = ?1 ORDER BY chunk_index"
    )?;

    let chunks = stmt.query_map([document_id], |row| {
        Ok(KbChunk {
            id: row.get(0)?,
            document_id: row.get(1)?,
            chunk_index: row.get(2)?,
            content: row.get(3)?,
            page_number: row.get(4)?,
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(chunks)
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
                bm25(kb_chunks_fts) as score
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
        })
    })?
    .filter_map(|r| r.ok())
    .collect();

    Ok(results)
}

// LIKE 搜索（支持中文）
fn like_search(conn: &rusqlite::Connection, query: &str, limit: i64) -> Result<Vec<KbSearchResult>> {
    // 提取查询中的关键词（按空格、逗号分割，或者对中文逐字符匹配）
    let keywords: Vec<&str> = query
        .split(|c: char| c.is_whitespace() || c == ',' || c == '，')
        .filter(|s| !s.is_empty())
        .collect();

    if keywords.is_empty() {
        return Ok(vec![]);
    }

    // 构建 LIKE 查询条件
    let like_conditions: Vec<String> = keywords
        .iter()
        .map(|k| format!("c.content LIKE '%{}%'", k.replace("'", "''")))
        .collect();

    let where_clause = like_conditions.join(" OR ");

    let sql = format!(
        "SELECT c.id, c.document_id, d.title, c.content, c.page_number, 0.0 as score
         FROM kb_chunks c
         JOIN kb_documents d ON c.document_id = d.id
         WHERE {}
         LIMIT ?1",
        where_clause
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
