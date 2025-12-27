use serde::{Deserialize, Serialize};

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
