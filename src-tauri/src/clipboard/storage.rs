use std::path::Path;

use rusqlite::{params, Connection, Result};

use crate::{
    clipboard::models::{ClipboardContentType, ClipboardItem},
    processes::get_process_icon,
};

pub fn list_history(
    conn: &Connection,
    limit: i32,
    search: Option<&str>,
) -> Result<Vec<ClipboardItem>> {
    if let Some(term) = search.filter(|s| !s.trim().is_empty()) {
        let pattern = format!("%{}%", term.trim());
        let mut stmt = conn.prepare(
            "SELECT id, content_type, text_content, image_path, source_app, source_app_path, created_at
             FROM clipboard_history
             WHERE text_content LIKE ?1 OR source_app LIKE ?1
             ORDER BY created_at DESC, id DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![pattern, limit], map_row)?;
        return rows.collect();
    }

    let mut stmt = conn.prepare(
        "SELECT id, content_type, text_content, image_path, source_app, source_app_path, created_at
         FROM clipboard_history
         ORDER BY created_at DESC, id DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], map_row)?;
    rows.collect()
}

pub fn get_item(conn: &Connection, id: i64) -> Result<Option<ClipboardItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, content_type, text_content, image_path, source_app, source_app_path, created_at
         FROM clipboard_history
         WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map(params![id], map_row)?;

    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn insert_item(
    conn: &Connection,
    content_type: ClipboardContentType,
    text_content: Option<&str>,
    image_path: Option<&str>,
    source_app: Option<&str>,
    source_app_path: Option<&str>,
    created_at: i64,
) -> Result<i64> {
    let type_str = content_type.as_str();

    conn.execute(
        "INSERT INTO clipboard_history (content_type, text_content, image_path, source_app, source_app_path, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            type_str,
            text_content,
            image_path,
            source_app,
            source_app_path,
            created_at
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn delete_item(conn: &Connection, id: i64) -> Result<Option<String>> {
    let image_path: Option<String> = conn
        .query_row(
            "SELECT image_path FROM clipboard_history WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .ok();

    conn.execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])?;

    Ok(image_path)
}

pub fn clear_history(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT image_path FROM clipboard_history WHERE image_path IS NOT NULL")?;
    let paths = stmt
        .query_map([], |row| row.get::<_, Option<String>>(0))?
        .filter_map(|row| row.ok().flatten())
        .collect::<Vec<_>>();

    conn.execute("DELETE FROM clipboard_history", [])?;

    Ok(paths)
}

pub fn effective_history_limit(limit: i32) -> i32 {
    if limit <= 0 {
        i32::MAX / 2
    } else {
        limit
    }
}

pub fn trim_to_limit(conn: &Connection, max_items: i32) -> Result<Vec<String>> {
    if max_items <= 0 {
        return Ok(vec![]);
    }

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM clipboard_history", [], |row| {
        row.get(0)
    })?;

    if count <= max_items as i64 {
        return Ok(vec![]);
    }

    let mut stmt = conn.prepare(
        "SELECT image_path FROM clipboard_history
         WHERE id IN (
             SELECT id FROM clipboard_history
             ORDER BY created_at ASC, id ASC
             LIMIT ?1
         )
         AND image_path IS NOT NULL",
    )?;

    let excess = (count - max_items as i64) as i32;
    let removed_images = stmt
        .query_map(params![excess], |row| row.get::<_, Option<String>>(0))?
        .filter_map(|row| row.ok().flatten())
        .collect::<Vec<_>>();

    conn.execute(
        "DELETE FROM clipboard_history
         WHERE id IN (
             SELECT id FROM clipboard_history
             ORDER BY created_at ASC, id ASC
             LIMIT ?1
         )",
        params![excess],
    )?;

    Ok(removed_images)
}

pub fn latest_matches(
    conn: &Connection,
    content_type: ClipboardContentType,
    text_content: Option<&str>,
    image_path: Option<&str>,
) -> Result<bool> {
    let mut stmt = conn.prepare(
        "SELECT content_type, text_content, image_path
         FROM clipboard_history
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
    )?;

    let mut rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    })?;

    let Some(Ok((latest_type, latest_text, latest_image))) = rows.next() else {
        return Ok(false);
    };

    Ok(latest_type == content_type.as_str()
        && latest_text.as_deref() == text_content
        && latest_image.as_deref() == image_path)
}

impl ClipboardContentType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
        }
    }

    fn from_db(value: &str) -> Option<Self> {
        match value {
            "text" => Some(Self::Text),
            "image" => Some(Self::Image),
            _ => None,
        }
    }
}

fn map_row(row: &rusqlite::Row<'_>) -> Result<ClipboardItem> {
    let content_type =
        ClipboardContentType::from_db(&row.get::<_, String>(1)?).ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(
                1,
                "content_type".into(),
                rusqlite::types::Type::Text,
            )
        })?;

    let text_content: Option<String> = row.get(2)?;
    let image_path: Option<String> = row.get(3)?;
    let source_app: Option<String> = row.get(4)?;
    let source_app_path: Option<String> = row.get(5)?;
    let created_at: i64 = row.get(6)?;

    let source_app_icon = source_app
        .as_deref()
        .map(|name| {
            get_process_icon(
                name,
                source_app_path.as_deref().filter(|path| !path.is_empty()),
            )
        })
        .flatten();

    let (image_width, image_height, image_size, image_filename) =
        image_metadata(image_path.as_deref());

    Ok(ClipboardItem {
        id: row.get(0)?,
        content_type,
        text_content,
        image_path,
        source_app,
        source_app_icon,
        source_app_path,
        image_width,
        image_height,
        image_size,
        image_filename,
        created_at,
    })
}

fn image_metadata(path: Option<&str>) -> (Option<u32>, Option<u32>, Option<u64>, Option<String>) {
    let Some(path) = path else {
        return (None, None, None, None);
    };

    let file_path = Path::new(path);
    let image_filename = file_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string());

    let image_size = std::fs::metadata(path).ok().map(|meta| meta.len());

    if let Ok(reader) = image::ImageReader::open(path) {
        if let Ok((width, height)) = reader.into_dimensions() {
            return (Some(width), Some(height), image_size, image_filename);
        }
    }

    (None, None, image_size, image_filename)
}
