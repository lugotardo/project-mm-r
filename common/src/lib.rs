use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Representa uma posição 2D no mundo baseado em tiles
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn moved(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

/// Representa um tile do terreno (camada de terreno)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Tile {
    pub terrain: TerrainType,
    pub walkable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum TerrainType {
    Grass,
    Water,
    Stone,
    Sand,
}

impl Tile {
    pub fn new(terrain: TerrainType, walkable: bool) -> Self {
        Self { terrain, walkable }
    }

    pub fn grass() -> Self {
        Self::new(TerrainType::Grass, true)
    }

    pub fn water() -> Self {
        Self::new(TerrainType::Water, false)
    }
}

/// Representa uma entidade no mundo (camada de entidades)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Entity {
    pub id: u32,
    pub name: String,
    pub pos: Position,
    pub entity_type: EntityType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum EntityType {
    Player,
    NPC,
    Animal,
}

impl Entity {
    pub fn new(id: u32, name: String, pos: Position, entity_type: EntityType) -> Self {
        Self {
            id,
            name,
            pos,
            entity_type,
        }
    }
}

/// Sistema de Fações
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Faction {
    pub id: u32,
    pub name: String,
    pub faction_type: FactionType,
    pub territory: Vec<Position>,
    pub member_count: usize,
    pub relations: HashMap<u32, Relation>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum FactionType {
    Human,
    Dwarf,
    Elf,
    Goblin,
    Wildlife,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Relation {
    Allied,
    Friendly,
    Neutral,
    Hostile,
    War,
}

/// Comportamento de IA
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AIBehavior {
    pub current_goal: AIGoal,
    pub memory: Vec<Memory>,
    pub personality: Personality,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum AIGoal {
    Wander,
    Hunt,
    Flee,
    Patrol { start: Position, end: Position },
    Guard { pos: Position },
    Sleep,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Memory {
    pub event: String,
    pub tick: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Personality {
    pub aggression: f32,   // 0.0 - 1.0
    pub curiosity: f32,
    pub sociability: f32,
}

/// Evento histórico
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoricalEvent {
    pub id: u64,
    pub tick: u64,
    pub event_type: EventType,
    pub participants: Vec<u32>,
    pub location: Position,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum EventType {
    Birth,
    Death,
    Combat,
    FactionFounded,
    TerritoryConquered,
    Alliance,
    War,
}

/// Mensagens do servidor para o cliente
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    /// Atualização completa do mundo visível
    WorldUpdate {
        tiles: Vec<(Position, Tile)>,
        entities: Vec<Entity>,
    },
    /// Confirmação de ação
    ActionResult { success: bool, message: String },
}

/// Mensagens do cliente para o servidor
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    /// Tentativa de movimento
    Move { dx: i32, dy: i32 },
    /// Login inicial
    Login { player_name: String },
}

/// Sistema de Autenticação
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub role: UserRole,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UserRole {
    Player,
    Admin,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub token: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub ip_address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<uuid::Uuid>,
    pub message: String,
    pub role: Option<UserRole>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthenticatedMessage {
    pub token: uuid::Uuid,
    pub message: ClientMessage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn test_position_moved() {
        let pos = Position::new(0, 0);
        let new_pos = pos.moved(3, -2);
        assert_eq!(new_pos, Position::new(3, -2));
    }

    #[test]
    fn test_tile_creation() {
        let grass = Tile::grass();
        assert!(grass.walkable);
        assert_eq!(grass.terrain, TerrainType::Grass);

        let water = Tile::water();
        assert!(!water.walkable);
        assert_eq!(water.terrain, TerrainType::Water);
    }

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(
            1,
            "TestPlayer".to_string(),
            Position::new(0, 0),
            EntityType::Player,
        );
        assert_eq!(entity.id, 1);
        assert_eq!(entity.name, "TestPlayer");
        assert_eq!(entity.entity_type, EntityType::Player);
    }

    #[test]
    fn test_message_serialization() {
        let msg = ClientMessage::Move { dx: 1, dy: -1 };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
        
        if let ClientMessage::Move { dx, dy } = deserialized {
            assert_eq!(dx, 1);
            assert_eq!(dy, -1);
        } else {
            panic!("Wrong message type");
        }
    }
}
