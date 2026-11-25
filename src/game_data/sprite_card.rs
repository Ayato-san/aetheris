//! UI sprite inspection reads just the selected image, never the whole atlas.
use super::*;

#[derive(Debug)]
pub struct SpriteCard {
    pub id: i64,
    pub asset_path: String,
    pub width: u32,
    pub height: u32,
    pub image: Option<Vec<u8>>,
    pub mime: Option<String>,
    pub image_state: String,
    pub aliases: Vec<(String, String)>,
}

pub(super) fn load(db: &Connection, id: i64) -> Result<SpriteCard> {
    let mut card = db.query_row(
        "SELECT s.asset_path,s.width,s.height,i.image,i.mime_type,i.image_state FROM ui_sprites s JOIN catalogue_icons i ON i.id=s.icon_id WHERE s.id=?1",
        [id],
        |r| Ok(SpriteCard {
            id, asset_path: r.get(0)?, width: r.get(1)?, height: r.get(2)?,
            image: r.get(3)?, mime: r.get(4)?, image_state: r.get(5)?, aliases: vec![],
        }),
    )?;
    card.aliases = db.prepare("SELECT table_name,row_key FROM ui_sprite_aliases WHERE sprite_id=?1 ORDER BY table_name,row_key")?
        .query_map([id], |r| Ok((r.get(0)?,r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(card)
}
