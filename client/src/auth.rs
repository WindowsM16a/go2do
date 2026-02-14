use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::error::Error;
use reqwest::blocking::Client; // Using blocking client for simplicity in Phase 1, can migrate to async if needed
// Note: We'll likely use async where possible, but keyring operations are blocking.

const SERVICE_NAME: &str = "go2do-sync";
const USER_KEY: &str = "auth-token";

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    pub message: String,
    pub user_id: String,
}

pub struct AuthManager {
    base_url: String,
    device_name: String,
}

impl AuthManager {
    pub fn new(base_url: String) -> Self {
        // Generate a random device name if not set
        // For V1, we just call it "Desktop"
        let device_name = format!("Linux-Desktop-{}", uuid::Uuid::new_v4().to_string().chars().take(4).collect::<String>());
        
        Self {
            base_url,
            device_name,
        }
    }

    pub fn set_base_url(&mut self, url: String) {
        self.base_url = url;
    }

    pub fn get_token(&self) -> Result<String, Box<dyn Error>> {
        let entry = Entry::new(SERVICE_NAME, USER_KEY)?;
        let password = entry.get_password()?;
        Ok(password)
    }

    pub fn set_token(&self, token: &str) -> Result<(), Box<dyn Error>> {
        let entry = Entry::new(SERVICE_NAME, USER_KEY)?;
        entry.set_password(token)?;
        Ok(())
    }

    pub fn logout(&self) -> Result<(), Box<dyn Error>> {
        let entry = Entry::new(SERVICE_NAME, USER_KEY)?;
        entry.delete_credential()?;
        Ok(())
    }

    // Since we use HttpOnly cookies, the server sets the cookie.
    // BUT desktop clients need to store something to persist the session if cookies aren't automatically handled by a pervasive cookie jar.
    // Our server returns a `Set-Cookie`. reqwest's `cookie_store` can handle this.
    // However, for a simple desktop app, it's often easier if the server ALSO returns a token in the body for the desktop client to save.
    // 
    // REVISION: The server code I wrote ONLY sets a cookie. It doesn't return the token in JSON.
    // I need to update the server or handle cookies in Rust properly.
    // Handling persistent cookies in Rust across restarts is annoying (need to serialize the cookie jar).
    // 
    // PREFERRED APPROACH:
    // I will update the server to ALSO return the `token` in the JSON body for non-browser clients (like this Rust app).
    // It's much safer to store a token in the OS Keyring than a serialized cookie jar on disk.
    
    pub fn login(&self, email: &str, password: &str) -> Result<String, Box<dyn Error>> {
        let client = Client::new();
        let res = client.post(format!("{}/auth/login", self.base_url))
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()?;

        if !res.status().is_success() {
            return Err("Login failed".into());
        }

        // We need the token. 
        // If server returns it in body -> great.
        // If not, we might need to parse `Set-Cookie` header manually?
        
        // Let's assume for now I will patch the server to return `{ token: "..." }`.
        #[derive(Deserialize)]
        struct LoginResponse {
            token: Option<String>, 
            // user_id: String
        }
        
        let body: LoginResponse = res.json()?;
        
        if let Some(token) = body.token {
            self.set_token(&token)?;
            Ok(token)
        } else {
            // Fallback: Parse cookie? 
            Err("Server didn't return a token".into())
        }
    }
}
