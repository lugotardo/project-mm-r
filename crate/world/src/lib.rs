use common::*;
use std::collections::HashMap;

/// Gerenciador do mundo com sistema de camadas
pub struct World {
    /// Camada de terreno (base)
    terrain_layer: HashMap<Position, Tile>,
    /// Camada de entidades (criaturas, NPCs, jogadores)
    entity_layer: HashMap<u32, Entity>,
    /// Comportamentos de IA para entidades
    ai_behaviors: HashMap<u32, AIBehavior>,
    /// Fações no mundo
    #[allow(dead_code)]
    factions: HashMap<u32, Faction>,
    /// Eventos históricos
    historical_events: Vec<HistoricalEvent>,
    /// Dimensões do mundo
    width: i32,
    height: i32,
    /// Próximo ID de entidade
    next_entity_id: u32,
    /// Próximo ID de facção
    #[allow(dead_code)]
    next_faction_id: u32,
    /// Próximo ID de evento
    next_event_id: u64,
    /// Tick atual do mundo
    current_tick: u64,
}

impl World {
    /// Cria um novo mundo com tamanho especificado
    pub fn new(width: i32, height: i32) -> Self {
        let mut world = Self {
            terrain_layer: HashMap::new(),
            entity_layer: HashMap::new(),
            ai_behaviors: HashMap::new(),
            factions: HashMap::new(),
            historical_events: Vec::new(),
            width,
            height,
            next_entity_id: 1,
            next_faction_id: 1,
            next_event_id: 1,
            current_tick: 0,
        };
        
        world.generate_terrain();
        world.spawn_initial_npcs();
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

    /// Spawna NPCs iniciais com IA
    fn spawn_initial_npcs(&mut self) {
        // Spawna alguns NPCs com IA
        for i in 0..5 {
            let pos = Position::new(5 + i, 5 + i);
            if let Some(id) = self.spawn_entity(
                format!("NPC_{}", i),
                pos,
                EntityType::NPC,
            ) {
                self.ai_behaviors.insert(id, AIBehavior {
                    current_goal: AIGoal::Wander,
                    memory: Vec::new(),
                    personality: Personality {
                        aggression: 0.3,
                        curiosity: 0.7,
                        sociability: 0.5,
                    },
                });
            }
        }
    }

    /// Processa um tick do mundo
    pub fn tick(&mut self) {
        self.current_tick += 1;
        
        // Atualiza IA de todas as entidades
        self.update_ai();
        
        // Atualiza fações
        self.update_factions();
        
        // Gera eventos aleatórios
        if self.current_tick % 100 == 0 {
            self.generate_random_event();
        }
    }

    fn update_ai(&mut self) {
        let entity_ids: Vec<u32> = self.ai_behaviors.keys().copied().collect();
        
        for entity_id in entity_ids {
            if let Some(behavior) = self.ai_behaviors.get(&entity_id) {
                match behavior.current_goal {
                    AIGoal::Wander => {
                        // Movimento aleatório
                        let dx = (self.current_tick % 3) as i32 - 1;
                        let dy = ((self.current_tick / 3) % 3) as i32 - 1;
                        self.move_entity(entity_id, dx, dy);
                    }
                    AIGoal::Patrol { start, end } => {
                        // Patrulha entre dois pontos
                        if let Some(entity) = self.entity_layer.get(&entity_id) {
                            let target = if (self.current_tick / 50) % 2 == 0 { start } else { end };
                            let dx = (target.x - entity.pos.x).signum();
                            let dy = (target.y - entity.pos.y).signum();
                            self.move_entity(entity_id, dx, dy);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn update_factions(&mut self) {
        // Atualiza relações entre fações
        // TODO: Implementar lógica de diplomacia
    }

    fn generate_random_event(&mut self) {
        // Gera eventos históricos aleatórios
        if self.entity_layer.len() > 1 {
            let event = HistoricalEvent {
                id: self.next_event_id,
                tick: self.current_tick,
                event_type: EventType::Combat,
                participants: vec![1, 2],
                location: Position::new(10, 10),
                description: "A skirmish occurred in the grasslands".to_string(),
            };
            
            self.next_event_id += 1;
            self.historical_events.push(event);
        }
    }

    /// Retorna tick atual
    pub fn get_current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Retorna eventos históricos
    pub fn get_historical_events(&self, limit: usize) -> Vec<HistoricalEvent> {
        let start = if self.historical_events.len() > limit {
            self.historical_events.len() - limit
        } else {
            0
        };
        self.historical_events[start..].to_vec()
    }

    /// Retorna todas as entidades (para debug/admin)
    pub fn get_all_entities(&self) -> Vec<&Entity> {
        self.entity_layer.values().collect()
    }
    
    /// Retorna IDs de todas as entidades
    pub fn get_entity_ids(&self) -> Vec<u32> {
        self.entity_layer.keys().copied().collect()
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

    #[test]
    fn test_world_tick() {
        let mut world = World::new(20, 20);
        let initial_tick = world.get_current_tick();
        
        world.tick();
        
        assert_eq!(world.get_current_tick(), initial_tick + 1);
    }

    #[test]
    fn test_npc_spawning() {
        let world = World::new(20, 20);
        let npcs = world.entity_layer.values()
            .filter(|e| e.entity_type == EntityType::NPC)
            .count();
        
        assert_eq!(npcs, 5);
    }
}
