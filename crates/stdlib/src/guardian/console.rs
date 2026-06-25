//! Interactive Console for Guardian
//! Allows real-time interaction with the owner

use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

/// Custom error type for Guardian console
#[derive(Debug)]
pub enum ConsoleError {
    Io(std::io::Error),
    Dialoguer(dialoguer::Error),
}

impl std::fmt::Display for ConsoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsoleError::Io(e) => write!(f, "IO error: {}", e),
            ConsoleError::Dialoguer(e) => write!(f, "Dialoguer error: {}", e),
        }
    }
}

impl std::error::Error for ConsoleError {}

impl From<std::io::Error> for ConsoleError {
    fn from(err: std::io::Error) -> Self {
        ConsoleError::Io(err)
    }
}

impl From<dialoguer::Error> for ConsoleError {
    fn from(err: dialoguer::Error) -> Self {
        ConsoleError::Dialoguer(err)
    }
}

/// Interactive console for Guardian
pub struct GuardianConsole;

impl GuardianConsole {
    /// Create new console instance
    pub fn new() -> Self {
        Self
    }

    /// Ask yes/no question to owner
    pub fn ask_yes_no(&self, question: &str, default: bool) -> Result<bool, ConsoleError> {
        let result = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(question)
            .default(default)
            .interact()?;
        Ok(result)
    }

    /// Ask for text input from owner
    pub fn ask_text(&self, prompt: &str, default: &str) -> Result<String, ConsoleError> {
        let result = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(default.to_string())
            .interact_text()?;
        Ok(result)
    }

    /// Show selection menu
    pub fn select(&self, prompt: &str, items: &[&str]) -> Result<usize, ConsoleError> {
        let result = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(items)
            .default(0)
            .interact()?;
        Ok(result)
    }

    /// Display warning message
    pub fn warn(&self, message: &str) {
        println!("\n⚠️  GUARDIAN WARNING");
        println!("{}", "─".repeat(50));
        println!("{}", message);
        println!("{}\n", "─".repeat(50));
    }

    /// Display alert message
    pub fn alert(&self, message: &str) {
        println!("\n🚨 GUARDIAN ALERT");
        println!("{}", "█".repeat(50));
        println!("{}", message);
        println!("{}\n", "█".repeat(50));
    }

    /// Display info message
    pub fn info(&self, message: &str) {
        println!("\n🛡️  GUARDIAN INFO");
        println!("{}", "─".repeat(50));
        println!("{}", message);
        println!("{}\n", "─".repeat(50));
    }

    /// Ask for network permission
    pub fn ask_network_permission(
        &self,
        url: &str,
        domain: &str,
        reason: Option<&str>,
    ) -> Result<bool, ConsoleError> {
        println!("\n🌐 NETWORK ACCESS REQUEST");
        println!("{}", "═".repeat(60));
        println!("URL: {}", url);
        println!("Domain: {}", domain);

        if let Some(r) = reason {
            println!("Reason: {}", r);
        }

        println!("{}", "═".repeat(60));

        let choices = &[
            "✅ Allow this request",
            "❌ Block this request",
            "⚙️  Add domain to whitelist (always allow)",
            "🚫 Add domain to blacklist (always block)",
        ];

        let selection = self.select("What do you want to do?", choices)?;

        match selection {
            0 => Ok(true),  // Allow
            1 => Ok(false), // Block
            2 => Ok(true),  // Add to whitelist (allow for now)
            3 => Ok(false), // Add to blacklist (block for now)
            _ => Ok(false),
        }
    }
}

impl Default for GuardianConsole {
    fn default() -> Self {
        Self::new()
    }
}
