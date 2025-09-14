use console;
use std::io::{self, Write};

/// Terminal control and ANSI color handling
pub struct Terminal {
    pub supports_color: bool,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            supports_color: console::colors_enabled(),
        }
    }

    /// Clear the entire screen
    pub fn clear_screen(&self) -> io::Result<()> {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush()
    }



    /// Hide cursor during updates to prevent flicker
    pub fn hide_cursor(&self) -> io::Result<()> {
        print!("\x1B[?25l");
        io::stdout().flush()
    }

    /// Show cursor
    pub fn show_cursor(&self) -> io::Result<()> {
        print!("\x1B[?25h");
        io::stdout().flush()
    }

    /// Get color style based on performance level
    pub fn get_performance_style(&self, percentage: f64) -> console::Style {
        let mut style = console::Style::new();
        if !self.supports_color {
            return style;
        }

        if percentage >= 80.0 {
            style = style.green(); // Excellent
        } else if percentage >= 60.0 {
            style = style.yellow(); // Good
        } else {
            style = style.red(); // Poor
        }
        style
    }


}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}
