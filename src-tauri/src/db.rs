use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

static DB: OnceCell<Mutex<Connection>> = OnceCell::new();

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

    // 创建表
    conn.execute_batch(
        "
        -- 搜索词表
        CREATE TABLE IF NOT EXISTS keywords (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            keyword TEXT NOT NULL UNIQUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- 词根表
        CREATE TABLE IF NOT EXISTS roots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            word TEXT NOT NULL UNIQUE,
            translation TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- 搜索词-词根关联表
        CREATE TABLE IF NOT EXISTS keyword_roots (
            keyword_id INTEGER NOT NULL,
            root_id INTEGER NOT NULL,
            PRIMARY KEY (keyword_id, root_id),
            FOREIGN KEY (keyword_id) REFERENCES keywords(id) ON DELETE CASCADE,
            FOREIGN KEY (root_id) REFERENCES roots(id) ON DELETE CASCADE
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

        -- 创建索引
        CREATE INDEX IF NOT EXISTS idx_roots_word ON roots(word);
        CREATE INDEX IF NOT EXISTS idx_keywords_keyword ON keywords(keyword);
        ",
    )?;

    // 初始化分类数据
    init_categories(&conn)?;

    DB.set(Mutex::new(conn))
        .map_err(|_| rusqlite::Error::InvalidQuery)?;

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

// 导入关键词并分析词根
pub fn import_keywords(keywords: Vec<String>) -> Result<()> {
    let conn = get_db().lock();

    for keyword in keywords {
        let keyword = keyword.trim().to_lowercase();
        if keyword.is_empty() {
            continue;
        }

        // 插入关键词（忽略重复）
        conn.execute(
            "INSERT OR IGNORE INTO keywords (keyword) VALUES (?1)",
            [&keyword],
        )?;

        // 获取关键词ID
        let keyword_id: i64 = conn.query_row(
            "SELECT id FROM keywords WHERE keyword = ?1",
            [&keyword],
            |row| row.get(0),
        )?;

        // 分词并插入词根
        let words: Vec<&str> = keyword.split_whitespace().collect();
        for word in words {
            let word = word.trim();
            if word.is_empty() {
                continue;
            }

            // 插入词根（忽略重复）
            conn.execute("INSERT OR IGNORE INTO roots (word) VALUES (?1)", [word])?;

            // 获取词根ID
            let root_id: i64 =
                conn.query_row("SELECT id FROM roots WHERE word = ?1", [word], |row| {
                    row.get(0)
                })?;

            // 建立关联（忽略重复）
            conn.execute(
                "INSERT OR IGNORE INTO keyword_roots (keyword_id, root_id) VALUES (?1, ?2)",
                [keyword_id, root_id],
            )?;
        }
    }

    Ok(())
}

// 获取词根列表（带统计信息）
pub fn get_roots(
    search: Option<String>,
    category_ids: Option<Vec<i64>>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<RootWithCategories>, i64)> {
    let conn = get_db().lock();

    // 获取总关键词数
    let total_keywords: i64 =
        conn.query_row("SELECT COUNT(*) FROM keywords", [], |row| row.get(0))?;

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

    // 获取总数
    let total: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT id) FROM roots",
        [],
        |row| row.get(0),
    )?;

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

// 获取统计信息
pub fn get_stats() -> Result<(i64, i64)> {
    let conn = get_db().lock();
    let keyword_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM keywords", [], |row| row.get(0))?;
    let root_count: i64 = conn.query_row("SELECT COUNT(*) FROM roots", [], |row| row.get(0))?;
    Ok((keyword_count, root_count))
}

// 清空所有数据
pub fn clear_all_data() -> Result<()> {
    let conn = get_db().lock();
    conn.execute_batch(
        "
        DELETE FROM keyword_roots;
        DELETE FROM root_categories;
        DELETE FROM keywords;
        DELETE FROM roots;
        ",
    )?;
    Ok(())
}
