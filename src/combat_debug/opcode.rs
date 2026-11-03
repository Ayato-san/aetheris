//! S2C names from the supplied community registry. Names do not define bodies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpcodeInfo {
    pub wire: [u8; 2],
    pub name: &'static str,
    pub translated_name: Option<&'static str>,
    pub phase: &'static str,
    pub confidence: &'static str,
    pub source: &'static str,
}

impl OpcodeInfo {
    pub fn display_name(&self) -> &'static str {
        self.translated_name
            .filter(|name| !name.is_empty())
            .unwrap_or(self.name)
    }
}

pub const SOURCE: &str = "https://github.com/lunixxx37/aion2-protocol-research/blob/fe2122f0e54e1de854c11e91e3c7bf51916a4f52/docs/OPCODES.md";

macro_rules! catalog {
    ($($variant:ident => ($wire:pat, $bytes:expr, $name:literal, $phase:literal, $confidence:literal)),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Opcode { $($variant,)+ Unknown([u8; 2]) }

        impl Opcode {
            pub const KNOWN: &'static [Self] = &[$(Self::$variant,)+];

            pub fn from_wire(wire: [u8; 2]) -> Self {
                match wire { $($wire => Self::$variant,)+ other => Self::Unknown(other) }
            }

            pub fn wire(self) -> [u8; 2] {
                match self { $(Self::$variant => $bytes,)+ Self::Unknown(bytes) => bytes }
            }

            pub fn info(self) -> Option<OpcodeInfo> {
                match self {
                    $(Self::$variant => Some(OpcodeInfo {
                        wire: $bytes, name: $name, translated_name: None,
                        phase: $phase, confidence: $confidence, source: SOURCE,
                    }),)+
                    Self::Unknown(_) => None,
                }
            }
        }
    }
}

catalog! {
    LobbyHandshake => ([0x01, 0x39], [0x01, 0x39], "LOBBY_HANDSHAKE", "lobby", "parser-based"),
    LobbyPing => ([0x03, 0x39], [0x03, 0x39], "LOBBY_PING", "lobby", "parser-based"),
    LobbyAccount => ([0x06, 0x39], [0x06, 0x39], "LOBBY_ACCOUNT", "lobby", "parser-based"),
    LobbyServerList => ([0x09, 0x39], [0x09, 0x39], "LOBBY_SERVER_LIST", "lobby", "parser-based"),
    LobbyCharacterList => ([0x0B, 0x39], [0x0B, 0x39], "LOBBY_CHARACTER_LIST", "lobby", "parser-based"),
    LobbyJoin => ([0x0D, 0x39], [0x0D, 0x39], "LOBBY_JOIN", "lobby", "parser-based"),
    LobbyWorldRedirect => ([0x0F, 0x39], [0x0F, 0x39], "LOBBY_WORLD_REDIRECT", "lobby", "publicly-confirmed"),
    ServerTick => ([0x00, 0x36], [0x00, 0x36], "SERVER_TICK", "world", "locally-confirmed"),
    WorldPing => ([0x03, 0x36], [0x03, 0x36], "WORLD_PING", "world", "parser-based"),
    ServerRsaHandshake => ([0x11, 0x36], [0x11, 0x36], "SERVER_RSA_HANDSHAKE", "world", "locally-confirmed"),
    LoginOrServerTransfer => ([0x15, 0x36], [0x15, 0x36], "LOGIN_OR_SERVER_TRANSFER", "world", "inferred"),
    GameServerInfo => ([0x16, 0x36], [0x16, 0x36], "GAME_SERVER_INFO", "world", "inferred"),
    NameCheck => ([0x1A, 0x36], [0x1A, 0x36], "NAME_CHECK", "world", "parser-based"),
    Zone => ([0x23, 0x36], [0x23, 0x36], "ZONE", "world", "parser-based"),
    LocalPlayer => ([0x33, 0x36], [0x33, 0x36], "LOCAL_PLAYER", "world", "publicly-confirmed"),
    SummonSpawn => ([0x35, 0x36], [0x35, 0x36], "SUMMON_SPAWN", "world", "parser-based"),
    SpawnVariant => ([0x40, 0x36], [0x40, 0x36], "SPAWN_VARIANT", "world", "parser-based"),
    SpawnOrSummon => ([0x41, 0x36], [0x41, 0x36], "SPAWN_OR_SUMMON", "world", "publicly-confirmed"),
    EntityRemoveOrDeath => ([0x42, 0x36], [0x42, 0x36], "ENTITY_REMOVE_OR_DEATH", "world", "publicly-confirmed"),
    PlayerInfoVariant => ([0x44, 0x36], [0x44, 0x36], "PLAYER_INFO_VARIANT", "world", "parser-based"),
    PlayerInfo => ([0x45, 0x36], [0x45, 0x36], "PLAYER_INFO", "world", "publicly-confirmed"),
    MovementA => ([0x1A, 0x37], [0x1A, 0x37], "MOVEMENT_A", "world", "parser-based"),
    MovementB => ([0x1B, 0x37], [0x1B, 0x37], "MOVEMENT_B", "world", "parser-based"),
    MovementC => ([0x1C, 0x37], [0x1C, 0x37], "MOVEMENT_C", "world", "parser-based"),
    MovementRelated => ([0x1D, 0x37], [0x1D, 0x37], "MOVEMENT_RELATED", "world", "parser-based"),
    ActionOrCast => ([0x02, 0x38], [0x02, 0x38], "ACTION_OR_CAST", "world", "parser-based"),
    NpcPosition => ([0x03, 0x38], [0x03, 0x38], "NPC_POSITION", "world", "parser-based"),
    DirectDamage => ([0x04, 0x38], [0x04, 0x38], "DIRECT_DAMAGE", "world", "publicly-confirmed"),
    DamageOverTime => ([0x05, 0x38], [0x05, 0x38], "DAMAGE_OVER_TIME", "world", "parser-based"),
    CastEndOrDefense => ([0x06, 0x38], [0x06, 0x38], "CAST_END_OR_DEFENSE", "world", "parser-based"),
    SkillCast => ([0x09, 0x38], [0x09, 0x38], "SKILL_CAST", "world", "parser-based"),
    TargetSelected => ([0x0E, 0x38], [0x0E, 0x38], "TARGET_SELECTED", "world", "parser-based"),
    StatusApply => ([0x2A, 0x38], [0x2A, 0x38], "STATUS_APPLY", "world", "parser-based"),
    StatusApplyVariant => ([0x2B, 0x38], [0x2B, 0x38], "STATUS_APPLY_VARIANT", "world", "parser-based"),
    StatusRemove => ([0x2C, 0x38], [0x2C, 0x38], "STATUS_REMOVE", "world", "parser-based"),
    SummonOrSkillSequence => ([0x35, 0x38], [0x35, 0x38], "SUMMON_OR_SKILL_SEQUENCE", "world", "parser-based"),
    CombatBatch => ([0x3D, 0x38], [0x3D, 0x38], "COMBAT_BATCH", "world", "parser-based"),
    FeatureRelatedUnknown => ([0x1B, 0x56], [0x1B, 0x56], "FEATURE_RELATED_UNKNOWN", "world", "parser-based"),
    DungeonRunState => ([0x00, 0x61], [0x00, 0x61], "DUNGEON_RUN_STATE", "world", "parser-based"),
    DungeonResult => ([0x01, 0x61], [0x01, 0x61], "DUNGEON_RESULT", "world", "parser-based"),
    HpUpdate => ([0x00, 0x8D], [0x00, 0x8D], "HP_UPDATE", "world", "parser-based"),
    NicknameOrOwner => ([0x04, 0x8D], [0x04, 0x8D], "NICKNAME_OR_OWNER", "world", "parser-based"),
    Notice => ([0x2F, 0x8D], [0x2F, 0x8D], "NOTICE", "world", "parser-based"),
    NpcBroadcast => ([0x01, 0x91], [0x01, 0x91], "NPC_BROADCAST", "world", "parser-based"),
    HpState => ([0x1B, 0x92], [0x1B, 0x92], "HP_STATE", "world", "parser-based"),
    PartyState => ([0x02, 0x97], [0x02, 0x97], "PARTY_STATE", "world", "parser-based"),
    GroggyOrGuard => ([0x05, 0xE0], [0x05, 0xE0], "GROGGY_OR_GUARD", "world", "parser-based"),
}
