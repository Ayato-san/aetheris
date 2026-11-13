//! Bounded LZ4 transformation. Each derived byte range names its envelope.
use super::analysis::{AnalysisRow, ByteProvenance, read_varuint};

const MAX_PLAIN_BYTES: usize = 8 * 1024 * 1024;
const MAX_EXPANDED_BYTES: usize = 32 * 1024 * 1024;
const MAX_RATIO: usize = 256;
const MAX_DEPTH: usize = 4;
const MAX_DERIVED_ROWS: usize = 65_536;

/// Physical length and prefix width. None means a potentially incomplete prefix.
pub(crate) fn frame_length(bytes: &[u8]) -> Result<Option<(usize, usize)>, &'static str> {
    let Some((length, width)) = read_varuint(bytes) else {
        return if bytes.len() < 10 && bytes.iter().all(|b| b & 0x80 != 0) {
            Ok(None)
        } else {
            Err("ULEB128 invalide")
        };
    };
    if width > 1 && bytes[width - 1] == 0 {
        return Err("ULEB128 non minimal");
    }
    if !(6..=MAX_PLAIN_BYTES as u64).contains(&length) {
        return Err("taille hors limites");
    }
    Ok(Some((width + length as usize - 4, width)))
}

pub(super) fn expand(row: AnalysisRow, width: usize, rows: &mut Vec<AnalysisRow>) {
    let mut budget = Budget {
        bytes: MAX_EXPANDED_BYTES,
        rows: MAX_DERIVED_ROWS,
    };
    expand_inner(row, width, rows, 0, &mut budget);
}

struct Budget {
    bytes: usize,
    rows: usize,
}

fn expand_inner(
    mut row: AnalysisRow,
    width: usize,
    rows: &mut Vec<AnalysisRow>,
    depth: usize,
    budget: &mut Budget,
) {
    let body = &row.bytes[width..];
    let marker = if body.starts_with(&[0xff, 0xff]) {
        Some(2)
    } else if matches!(body.first(), Some(0xf0..=0xfe)) && body.get(1..3) == Some(&[0xff, 0xff]) {
        Some(3)
    } else {
        None
    };
    let Some(marker) = marker else {
        super::interpretation::interpret(&mut row, width);
        rows.push(row);
        return;
    };
    row.kind = "enveloppe LZ4".into();
    row.entity.clear();
    row.opcode.clear();
    row.sections.clear();
    let decoded = (|| {
        if depth >= MAX_DEPTH {
            return Err("profondeur LZ4 maximale atteinte");
        }
        let size = body
            .get(marker..marker + 4)
            .ok_or("en-tête LZ4 incomplet")?;
        let size = u32::from_le_bytes(size.try_into().unwrap()) as usize;
        let compressed = &body[marker + 4..];
        if size == 0
            || size > MAX_PLAIN_BYTES
            || size > budget.bytes
            || size > compressed.len().saturating_mul(MAX_RATIO)
        {
            return Err("taille, ratio ou budget LZ4 hors limites");
        }
        let plain =
            lz4_flex::block::decompress(compressed, size).map_err(|_| "bloc LZ4 invalide")?;
        if plain.len() != size {
            return Err("taille décompressée différente de la taille annoncée");
        }
        budget.bytes -= size;
        Ok(plain)
    })();
    let envelope = rows.len();
    let plain = match decoded {
        Ok(plain) => plain,
        Err(error) => {
            row.status = "incomplet / erreur".into();
            row.summary = format!("{} · Enveloppe LZ4 : {error}", row.summary);
            rows.push(row);
            return;
        }
    };
    row.status = "observé".into();
    row.summary = format!(
        "{} · Enveloppe LZ4 · {} octets décompressés (documentation communautaire)",
        row.summary,
        plain.len()
    );
    rows.push(row);
    let mut template = rows[envelope].clone();
    template.bytes.clear();
    template.kind = "message".into();
    template.status = "inconnu".into();
    let mut offset = 0;
    while offset < plain.len() {
        if plain[offset] == 0 {
            offset += 1;
            continue;
        }
        let frame = frame_length(&plain[offset..]);
        let mut child = template.clone();
        child.provenance = Some(ByteProvenance::Decompressed { envelope, offset });
        match frame {
            Ok(Some((length, width))) if length <= plain.len() - offset && budget.rows > 0 => {
                budget.rows -= 1;
                child.bytes = plain[offset..offset + length].to_vec();

                child.summary =
                    format!("Message interne · offset décompressé {offset} · framing expérimental");
                expand_inner(child, width, rows, depth + 1, budget);
                offset += length;
            }
            _ => {
                child.kind = "octets décompressés non interprétés".into();
                child.status = "incomplet / erreur".into();
                child.bytes = plain[offset..].to_vec();
                child.summary = "Branche décompressée arrêtée : longueur invalide, message incomplet ou limite de lignes atteinte".into();
                rows[envelope].status = "incomplet / erreur".into();
                rows[envelope]
                    .summary
                    .push_str(" · branche interne incomplète");
                rows.push(child);
                break;
            }
        }
    }
}
