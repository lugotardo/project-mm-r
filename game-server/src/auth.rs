use common::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};

pub type SharedAuthState = Arc<Mutex<AuthState>>;

pub struct AuthState {
    users: HashMap<uuid::Uuid, User>,
    sessions: HashMap<uuid::Uuid, Session>,
    username_index: HashMap<String, uuid::Uuid>,
}

impl AuthState {
    pub fn new() -> Self {
        let mut state = Self {
            users: HashMap::new(),
            sessions: HashMap::new(),
            username_index: HashMap::new(),
        };
        
        // Criar usuário admin padrão (MUDAR SENHA EM PRODUÇÃO!)
        state.create_default_admin();
        state
    }

    fn create_default_admin(&mut self) {
        let admin_id = uuid::Uuid::new_v4();
        let password_hash = hash("admin123", DEFAULT_COST).unwrap();
        
        let admin = User {
            id: admin_id,
            username: "admin".to_string(),
            password_hash,
            created_at: Utc::now(),
            role: UserRole::Admin,
        };
        
        self.users.insert(admin_id, admin.clone());
        self.username_index.insert("admin".to_string(), admin_id);
        
        println!("⚠️  Admin padrão criado: username=admin, password=admin123");
        println!("⚠️  MUDE A SENHA IMEDIATAMENTE EM PRODUÇÃO!");
    }

    pub fn register_user(&mut self, username: String, password: String) -> Result<uuid::Uuid, String> {
        if self.username_index.contains_key(&username) {
            return Err("Username já existe".to_string());
        }

        if username.len() < 3 || username.len() > 20 {
            return Err("Username deve ter entre 3 e 20 caracteres".to_string());
        }

        if password.len() < 6 {
            return Err("Senha deve ter pelo menos 6 caracteres".to_string());
        }

        let user_id = uuid::Uuid::new_v4();
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|_| "Erro ao criar hash da senha".to_string())?;

        let user = User {
            id: user_id,
            username: username.clone(),
            password_hash,
            created_at: Utc::now(),
            role: UserRole::Player,
        };

        self.users.insert(user_id, user);
        self.username_index.insert(username, user_id);

        Ok(user_id)
    }

    pub fn login(&mut self, username: String, password: String, ip: String) -> LoginResponse {
        let user_id = match self.username_index.get(&username) {
            Some(id) => *id,
            None => {
                return LoginResponse {
                    success: false,
                    token: None,
                    message: "Usuário ou senha inválidos".to_string(),
                    role: None,
                };
            }
        };

        let user = self.users.get(&user_id).unwrap();

        if !verify(&password, &user.password_hash).unwrap_or(false) {
            return LoginResponse {
                success: false,
                token: None,
                message: "Usuário ou senha inválidos".to_string(),
                role: None,
            };
        }

        // Criar sessão
        let token = uuid::Uuid::new_v4();
        let session = Session {
            token,
            user_id,
            expires_at: Utc::now() + Duration::hours(24),
            ip_address: ip,
        };

        self.sessions.insert(token, session);

        LoginResponse {
            success: true,
            token: Some(token),
            message: format!("Bem-vindo, {}!", username),
            role: Some(user.role.clone()),
        }
    }

    pub fn validate_session(&mut self, token: uuid::Uuid) -> Option<&User> {
        // Remove sessões expiradas
        self.sessions.retain(|_, session| session.expires_at > Utc::now());

        let session = self.sessions.get(&token)?;
        
        if session.expires_at < Utc::now() {
            return None;
        }

        self.users.get(&session.user_id)
    }

    pub fn logout(&mut self, token: uuid::Uuid) {
        self.sessions.remove(&token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_login() {
        let mut auth = AuthState::new();
        
        let user_id = auth.register_user("testuser".to_string(), "password123".to_string()).unwrap();
        assert!(user_id != uuid::Uuid::nil());

        let response = auth.login("testuser".to_string(), "password123".to_string(), "127.0.0.1".to_string());
        assert!(response.success);
        assert!(response.token.is_some());
    }

    #[test]
    fn test_invalid_login() {
        let mut auth = AuthState::new();
        let response = auth.login("fake".to_string(), "wrong".to_string(), "127.0.0.1".to_string());
        assert!(!response.success);
    }
}
