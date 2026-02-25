//! Backend selection logic
//!
//! Automatically detects the best available backend based on the environment
//! (TTY detection, available features, user preferences).

use super::BackendType;
use crate::{Result, TategakiError};

/// Backend selection preferences
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendPreference {
    /// Automatically detect the best backend
    Auto,
    /// Force a specific backend type
    Specific(BackendType),
}

/// Backend selector for choosing the appropriate rendering backend
pub struct BackendSelector {
    preference: BackendPreference,
    force_terminal: bool,
}

impl BackendSelector {
    /// Create a new backend selector with auto-detection
    pub fn new() -> Self {
        Self {
            preference: BackendPreference::Auto,
            force_terminal: false,
        }
    }

    /// Create a selector with a specific backend preference
    pub fn with_preference(preference: BackendPreference) -> Self {
        Self {
            preference,
            force_terminal: false,
        }
    }

    /// Force terminal mode regardless of other settings
    pub fn force_terminal(mut self) -> Self {
        self.force_terminal = true;
        self
    }

    /// Select the best available backend
    pub fn select(&self) -> Result<BackendType> {
        match &self.preference {
            BackendPreference::Specific(backend_type) => {
                if backend_type.is_available() {
                    Ok(*backend_type)
                } else {
                    Err(TategakiError::Rendering(format!(
                        "Requested backend {} is not available (feature not enabled)",
                        backend_type.name()
                    )))
                }
            }
            BackendPreference::Auto => self.auto_select(),
        }
    }

    /// Automatically select the best backend based on environment
    fn auto_select(&self) -> Result<BackendType> {
        // If terminal mode is forced, prefer notcurses
        if self.force_terminal {
            #[cfg(feature = "notcurses")]
            return Ok(BackendType::Notcurses);

            #[cfg(not(feature = "notcurses"))]
            return Err(TategakiError::Rendering(
                "Terminal mode requested but notcurses feature is not enabled".to_string(),
            ));
        }

        // Check if we're running in a TTY
        if Self::is_tty() {
            // In TTY, prefer notcurses over ratatui over GPUI
            #[cfg(feature = "notcurses")]
            return Ok(BackendType::Notcurses);

            #[cfg(not(feature = "notcurses"))]
            #[cfg(feature = "ratatui")]
            return Ok(BackendType::Ratatui);
        }

        // Not in TTY or terminal backends not available, try GPUI
        #[cfg(feature = "gpui")]
        if Self::has_display() {
            return Ok(BackendType::Gpui);
        }

        // Fallback: try any available backend
        #[cfg(feature = "gpui")]
        return Ok(BackendType::Gpui);

        #[cfg(not(feature = "gpui"))]
        #[cfg(feature = "notcurses")]
        return Ok(BackendType::Notcurses);

        #[cfg(not(feature = "gpui"))]
        #[cfg(not(feature = "notcurses"))]
        #[cfg(feature = "ratatui")]
        return Ok(BackendType::Ratatui);

        #[cfg(not(any(feature = "gpui", feature = "notcurses", feature = "ratatui")))]
        Err(TategakiError::Rendering(
            "No rendering backend is available (enable gpui, notcurses, or ratatui feature)"
                .to_string(),
        ))
    }

    /// Check if we're running in a TTY
    fn is_tty() -> bool {
        use std::io::IsTerminal;
        std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
    }

    /// Check if a display server is available (for GPUI)
    fn has_display() -> bool {
        #[cfg(target_os = "linux")]
        {
            // Check for Wayland or X11
            std::env::var("WAYLAND_DISPLAY").is_ok() || std::env::var("DISPLAY").is_ok()
        }

        #[cfg(target_os = "macos")]
        {
            // macOS always has a display server
            true
        }

        #[cfg(target_os = "windows")]
        {
            // Windows always has a display
            true
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            // Unknown platform, assume no display
            false
        }
    }
}

impl Default for BackendSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_selector_creation() {
        let selector = BackendSelector::new();
        assert_eq!(selector.preference, BackendPreference::Auto);
        assert!(!selector.force_terminal);
    }

    #[test]
    fn test_force_terminal() {
        let selector = BackendSelector::new().force_terminal();
        assert!(selector.force_terminal);
    }

    #[test]
    fn test_auto_select() {
        let selector = BackendSelector::new();
        let result = selector.select();
        // Should succeed on any platform with at least one backend enabled
        #[cfg(any(feature = "gpui", feature = "notcurses", feature = "ratatui"))]
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "notcurses")]
    fn test_specific_backend_notcurses() {
        let selector =
            BackendSelector::with_preference(BackendPreference::Specific(BackendType::Notcurses));
        let result = selector.select();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), BackendType::Notcurses);
    }
}
