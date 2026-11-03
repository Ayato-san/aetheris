//! Shared interpretation of direct and decompressed S2C messages.
use super::{
    analysis::{AnalysisRow, DecodedSection},
    hex,
    opcode::Opcode,
};

pub(super) fn interpret(row: &mut AnalysisRow, width: usize) {
    row.entity.clear();
    row.sections.clear();
    let body = &row.bytes[width..];
    let opcode = Opcode::from_wire([body[0], body[1]]);
    row.opcode = hex(&opcode.wire());
    let (name, confidence) = match opcode.info() {
        Some(info) => (
            info.name,
            format!("documentation communautaire · {}", info.confidence),
        ),
        None => (
            "Opcode inconnu",
            "non documenté dans le catalogue S2C".into(),
        ),
    };
    row.sections.push(DecodedSection {
        kind: "Opcode".into(),
        value: name.into(),
        confidence: confidence.clone(),
        offset: width,
        bytes: body[..2].to_vec(),
    });
    if let Some(info) = opcode.info() {
        row.sections.push(DecodedSection {
            kind: "Source de l’opcode".into(),
            value: info.source.into(),
            confidence: format!(
                "{} · phase {} · preuve externe à Aetheris",
                info.confidence, info.phase
            ),
            offset: width,
            bytes: Vec::new(),
        });
    }
    row.status = if opcode.info().is_some() {
        "identifié"
    } else {
        "inconnu"
    }
    .into();
    let interpretation = if opcode == Opcode::ServerTick {
        if body.len() == 10 {
            let time = u64::from_le_bytes(body[2..10].try_into().unwrap());
            row.sections.push(DecodedSection {
                kind: "Horodatage serveur (unix_ms, u64 LE)".into(),
                value: time.to_string(),
                confidence: confidence.clone(),
                offset: width + 2,
                bytes: body[2..10].to_vec(),
            });
            row.status = "décodé".into();
            format!("unix_ms = {time}")
        } else {
            row.status = "incomplet / erreur".into();
            format!(
                "corps SERVER_TICK invalide : {} octets, 10 attendus",
                body.len()
            )
        }
    } else {
        format!("corps conservé · {} octets non interprétés", body.len() - 2)
    };
    row.summary = format!("{} · {name} · {interpretation} · {confidence}", row.summary);
}
