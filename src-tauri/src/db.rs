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
    pub sku: Option<String>,
    pub asin: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Root {
    pub id: i64,
    pub word: String,
    pub translation: Option<String>,
    pub contains_count: i64,
    pub percentage: f64,
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

pub fn init_db(app_data_dir: PathBuf) -> Result<()> {
    std::fs::create_dir_all(&app_data_dir).ok();
    let db_path = app_data_dir.join("thesaurus.db");
    let conn = Connection::open(db_path)?;

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

    // 创建索引（迁移完成后）
    conn.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_roots_word ON roots(word);
        CREATE INDEX IF NOT EXISTS idx_roots_product ON roots(product_id);
        CREATE INDEX IF NOT EXISTS idx_keywords_keyword ON keywords(keyword);
        CREATE INDEX IF NOT EXISTS idx_keywords_product ON keywords(product_id);
        ",
    )?;

    // 初始化分类数据
    init_categories(&conn)?;

    DB.set(Mutex::new(conn))
        .map_err(|_| rusqlite::Error::InvalidQuery)?;

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
    let mut stmt = conn.prepare("SELECT id, name, sku, asin FROM products ORDER BY id")?;
    let products = stmt
        .query_map([], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                sku: row.get(2)?,
                asin: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(products)
}

// 创建产品
pub fn create_product(name: String, sku: Option<String>, asin: Option<String>) -> Result<i64> {
    let conn = get_db().lock();
    conn.execute(
        "INSERT INTO products (name, sku, asin) VALUES (?1, ?2, ?3)",
        rusqlite::params![name, sku, asin],
    )?;
    Ok(conn.last_insert_rowid())
}

// 更新产品
pub fn update_product(id: i64, name: String, sku: Option<String>, asin: Option<String>) -> Result<()> {
    let conn = get_db().lock();
    conn.execute(
        "UPDATE products SET name = ?1, sku = ?2, asin = ?3 WHERE id = ?4",
        rusqlite::params![name, sku, asin, id],
    )?;
    Ok(())
}

// 删除产品（同时删除关联的关键词和词根）
pub fn delete_product(id: i64) -> Result<()> {
    let conn = get_db().lock();
    // 由于外键级联删除，只需要删除产品即可
    conn.execute("DELETE FROM products WHERE id = ?1", [id])?;
    Ok(())
}

// ==================== 关键词和词根 ====================

// 导入关键词并分析词根（关联到指定产品）
pub fn import_keywords(product_id: i64, keywords: Vec<String>) -> Result<()> {
    let conn = get_db().lock();

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

    Ok(())
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

    // 获取总数
    let count_sql = format!(
        "SELECT COUNT(*) FROM ({}) as t",
        sql.replace("SELECT DISTINCT r.id, r.word, r.translation,", "SELECT DISTINCT r.id FROM roots r")
            .split(" ORDER BY")
            .next()
            .unwrap_or("")
    );

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

// 清空产品数据（只删除指定产品的关键词和词根）
pub fn clear_product_data(product_id: i64) -> Result<()> {
    let conn = get_db().lock();
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

// 批量更新词根翻译和分类（按产品筛选）
pub fn batch_update_root_analysis(
    product_id: i64,
    updates: Vec<(String, String, Vec<String>)>, // (word, translation, category_names)
) -> Result<()> {
    let conn = get_db().lock();

    for (word, translation, category_names) in updates {
        // 更新翻译
        conn.execute(
            "UPDATE roots SET translation = ?1 WHERE word = ?2 AND product_id = ?3",
            rusqlite::params![translation, word, product_id],
        )?;

        // 获取词根ID
        let root_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM roots WHERE word = ?1 AND product_id = ?2",
                rusqlite::params![&word, product_id],
                |row| row.get(0),
            )
            .ok();

        if let Some(root_id) = root_id {
            // 清除现有分类
            conn.execute("DELETE FROM root_categories WHERE root_id = ?1", [root_id])?;

            // 添加新分类
            for cat_name in category_names {
                let cat_id: Option<i64> = conn
                    .query_row(
                        "SELECT id FROM categories WHERE name = ?1",
                        [&cat_name],
                        |row| row.get(0),
                    )
                    .ok();

                if let Some(cat_id) = cat_id {
                    conn.execute(
                        "INSERT OR IGNORE INTO root_categories (root_id, category_id) VALUES (?1, ?2)",
                        [root_id, cat_id],
                    )?;
                }
            }
        }
    }

    Ok(())
}
