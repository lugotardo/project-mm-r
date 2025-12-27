use common::*;
use std::collections::HashMap;

/// Gerenciador do mundo com sistema de camadas
pub struct World {
    /// Camada de terreno (base)
    terrain_layer: HashMap<Position, Tile>,
    /// Camada de entidades (criaturas, NPCs, jogadores)
    entity_layer: HashMap<u32, Entity>,
    /// Dimensões do mundo
    width: i32,
    height: i32,
    /// Próximo ID de entidade
    next_entity_id: u32,
}

impl World {
    /// Cria um novo mundo com tamanho especificado
    pub fn new(width: i32, height: i32) -> Self {
        let mut world = Self {
            terrain_layer: HashMap::new(),
            entity_layer: HashMap::new(),
            width,
            height,
            next_entity_id: 1,
        };
        
        world.generate_terrain();
        world
    }

    /// Gera terreno procedural básico
    fn generate_terrain(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                let pos = Position::new(x, y);
                
                // Geração simples: água no centro, grama ao redor
                let tile = if self.is_water_region(x, y) {
                    Tile::water()
                } else if self.is_stone_region(x, y) {
                    Tile::new(TerrainType::Stone, true)
                } else {
                    Tile::grass()
                };
                
                self.terrain_layer.insert(pos, tile);
            }
        }
    }

    /// Determina se posição deve ser água
    fn is_water_region(&self, x: i32, y: i32) -> bool {
        let center_x = self.width / 2;
        let center_y = self.height / 2;
        let dx = (x - center_x).abs();
        let dy = (y - center_y).abs();
        
        // Lago circular no centro
        dx * dx + dy * dy < 9
    }

    /// Determina se posição deve ser pedra
    fn is_stone_region(&self, x: i32, y: i32) -> bool {
        // Montanhas nas bordas
        x < 2 || x >= self.width - 2 || y < 2 || y >= self.height - 2
    }

    /// Adiciona uma entidade ao mundo
    pub fn spawn_entity(&mut self, name: String, pos: Position, entity_type: EntityType) -> Option<u32> {
        // Valida se posição está no mundo
        if !self.is_valid_position(pos) {
            return None;
        }

        // Valida se tile é transitável
        if let Some(tile) = self.terrain_layer.get(&pos) {
            if !tile.walkable {
                return None;
            }
        }

        let id = self.next_entity_id;
        self.next_entity_id += 1;
        
        let entity = Entity::new(id, name, pos, entity_type);
        self.entity_layer.insert(id, entity);
        Some(id)
    }

    /// Move uma entidade
    pub fn move_entity(&mut self, entity_id: u32, dx: i32, dy: i32) -> bool {
        if let Some(entity) = self.entity_layer.get(&entity_id) {
            let new_pos = entity.pos.moved(dx, dy);
            
            // Valida limites do mundo
            if !self.is_valid_position(new_pos) {
                return false;
            }

            // Valida se tile é transitável
            if let Some(tile) = self.terrain_layer.get(&new_pos) {
                if !tile.walkable {
                    return false;
                }
            } else {
                return false;
            }

            // Move a entidade
            if let Some(entity) = self.entity_layer.get_mut(&entity_id) {
                entity.pos = new_pos;
                return true;
            }
        }
        false
    }

    /// Remove uma entidade do mundo
    pub fn despawn_entity(&mut self, entity_id: u32) -> bool {
        self.entity_layer.remove(&entity_id).is_some()
    }

    /// Verifica se posição está dentro dos limites
    fn is_valid_position(&self, pos: Position) -> bool {
        pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
    }

    /// Retorna tile em uma posição
    pub fn get_tile(&self, pos: Position) -> Option<&Tile> {
        self.terrain_layer.get(&pos)
    }

    /// Retorna entidade por ID
    pub fn get_entity(&self, id: u32) -> Option<&Entity> {
        self.entity_layer.get(&id)
    }

    /// Retorna todas as entidades em uma região
    pub fn get_entities_in_region(&self, center: Position, radius: i32) -> Vec<&Entity> {
        self.entity_layer.values()
            .filter(|e| {
                let dx = (e.pos.x - center.x).abs();
                let dy = (e.pos.y - center.y).abs();
                dx <= radius && dy <= radius
            })
            .collect()
    }

    /// Retorna snapshot do mundo visível para um jogador
    pub fn get_visible_snapshot(&self, center: Position, view_radius: i32) -> WorldSnapshot {
        let mut tiles = Vec::new();
        let mut entities = Vec::new();

        // Coleta tiles visíveis
        for x in (center.x - view_radius)..=(center.x + view_radius) {
            for y in (center.y - view_radius)..=(center.y + view_radius) {
                let pos = Position::new(x, y);
                if let Some(tile) = self.terrain_layer.get(&pos) {
                    tiles.push((pos, tile.clone()));
                }
            }
        }

        // Coleta entidades visíveis
        entities = self.get_entities_in_region(center, view_radius)
            .into_iter()
            .cloned()
            .collect();

        WorldSnapshot { tiles, entities }
    }

    /// Retorna dimensões do mundo
    pub fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    /// Retorna número de entidades ativas
    pub fn entity_count(&self) -> usize {
        self.entity_layer.len()
    }
}

/// Snapshot do mundo visível para enviar ao cliente
pub struct WorldSnapshot {
    pub tiles: Vec<(Position, Tile)>,
    pub entities: Vec<Entity>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = World::new(20, 20);
        assert_eq!(world.dimensions(), (20, 20));
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_terrain_generation() {
        let world = World::new(10, 10);
        
        // Centro deve ter água
        let center_tile = world.get_tile(Position::new(5, 5)).unwrap();
        assert_eq!(center_tile.terrain, TerrainType::Water);
        assert!(!center_tile.walkable);
        
        // Cantos devem ter pedra (montanhas)
        let corner_tile = world.get_tile(Position::new(0, 0)).unwrap();
        assert_eq!(corner_tile.terrain, TerrainType::Stone);
    }

    #[test]
    fn test_spawn_entity() {
        let mut world = World::new(10, 10);
        
        let id = world.spawn_entity(
            "TestPlayer".to_string(),
            Position::new(3, 3),
            EntityType::Player
        );
        
        assert!(id.is_some());
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_spawn_on_water_fails() {
        let mut world = World::new(10, 10);
        
        // Tenta spawnar na água (centro)
        let id = world.spawn_entity(
            "Player".to_string(),
            Position::new(5, 5),
            EntityType::Player
        );
        
        assert!(id.is_none());
    }

    #[test]
    fn test_move_entity() {
        let mut world = World::new(10, 10);
        
        let id = world.spawn_entity(
            "Player".to_string(),
            Position::new(3, 3),
            EntityType::Player
        ).unwrap();
        
        let success = world.move_entity(id, 1, 0);
        assert!(success);
        
        let entity = world.get_entity(id).unwrap();
        assert_eq!(entity.pos, Position::new(4, 3));
    }

    #[test]
    fn test_move_into_water_fails() {
        let mut world = World::new(10, 10);
        
        let id = world.spawn_entity(
            "Player".to_string(),
            Position::new(4, 5),
            EntityType::Player
        ).unwrap();
        
        // Tenta mover para água
        let success = world.move_entity(id, 1, 0);
        assert!(!success);
        
        // Posição não mudou
        let entity = world.get_entity(id).unwrap();
        assert_eq!(entity.pos, Position::new(4, 5));
    }

    #[test]
    fn test_move_out_of_bounds() {
        let mut world = World::new(10, 10);
        
        let id = world.spawn_entity(
            "Player".to_string(),
            Position::new(0, 0),
            EntityType::Player
        ).unwrap();
        
        let success = world.move_entity(id, -1, 0);
        assert!(!success);
    }

    #[test]
    fn test_despawn_entity() {
        let mut world = World::new(10, 10);
        
        let id = world.spawn_entity(
            "Player".to_string(),
            Position::new(3, 3),
            EntityType::Player
        ).unwrap();
        
        assert!(world.despawn_entity(id));
        assert_eq!(world.entity_count(), 0);
        assert!(world.get_entity(id).is_none());
    }

    #[test]
    fn test_get_entities_in_region() {
        let mut world = World::new(20, 20);
        
        world.spawn_entity("P1".to_string(), Position::new(10, 10), EntityType::Player);
        world.spawn_entity("P2".to_string(), Position::new(11, 11), EntityType::Player);
        world.spawn_entity("P3".to_string(), Position::new(15, 15), EntityType::Player);
        
        let nearby = world.get_entities_in_region(Position::new(10, 10), 2);
        assert_eq!(nearby.len(), 2);
    }

    #[test]
    fn test_visible_snapshot() {
        let mut world = World::new(20, 20);
        
        world.spawn_entity("P1".to_string(), Position::new(10, 10), EntityType::Player);
        
        let snapshot = world.get_visible_snapshot(Position::new(10, 10), 3);
        
        assert!(snapshot.tiles.len() > 0);
        assert_eq!(snapshot.entities.len(), 1);
    }
}
