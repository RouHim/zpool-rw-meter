use super::terminal::Terminal;

/// ASCII progress bar renderer matching shell script aesthetics
pub struct ProgressBar {
    width: usize,
    terminal: Terminal,
}

impl ProgressBar {
    pub fn new(width: usize) -> Self {
        Self {
            width,
            terminal: Terminal::new(),
        }
    }

    /// Render a progress bar with percentage
    /// Returns a string with the progress bar and percentage
    pub fn render(&self, percentage: f64, label: Option<&str>) -> String {
        let filled = (percentage / 100.0 * self.width as f64).round() as usize;
        let empty = self.width.saturating_sub(filled);

        let filled_chars = "#".repeat(filled);
        let empty_chars = ".".repeat(empty);

        let bar = format!("[{}{}]", filled_chars, empty_chars);
        let percent_text = format!("{:.1}%", percentage);

        let styled_bar = if self.terminal.supports_color {
            self.terminal
                .get_performance_style(percentage)
                .apply_to(&bar)
                .to_string()
        } else {
            bar
        };

        match label {
            Some(label) => format!("{} {} {}", label, styled_bar, percent_text),
            None => format!("{} {}", styled_bar, percent_text),
        }
    }


}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new(20) // Default width matching shell script
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_full() {
        let pb = ProgressBar::new(10);
        let result = pb.render(100.0, Some("Test"));
        assert!(result.contains("Test"));
        assert!(result.contains("[##########]"));
        assert!(result.contains("100.0%"));
    }

    #[test]
    fn test_progress_bar_half() {
        let pb = ProgressBar::new(10);
        let result = pb.render(50.0, None);
        assert!(result.contains("[#####.....]"));
        assert!(result.contains("50.0%"));
    }

    #[test]
    fn test_progress_bar_empty() {
        let pb = ProgressBar::new(10);
        let result = pb.render(0.0, None);
        assert!(result.contains("[..........]"));
        assert!(result.contains("0.0%"));
    }
}
