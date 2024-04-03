use clap_verbosity_flag::Level;
use color_eyre::owo_colors::OwoColorize;

pub struct Logger {
    pub prefix: String,
    pub verbosity: Option<Level>,
}

impl Logger {
    pub fn new(verbosity: &clap_verbosity_flag::Verbosity) -> Self {
        return Logger {
            prefix: String::from(">> ntfy"),
            verbosity: verbosity.log_level(),
        };
    }

    fn fmt_print<S: Into<String>>(&self, level: &str, text: S) {
        eprintln!(
            "{}",
            format!("{} | {} | {}", self.prefix, level, text.into())
        )
    }

    /// log without a level
    pub fn log<S: Into<String>>(&self, text: S) {
        // only hide if 'quiet'
        if self.verbosity.is_some() {
            eprintln!("{}", format!("{} | {}", self.prefix, text.into()));
        }
    }

    pub fn success<S: Into<String>>(&self, text: S) {
        // only hide if 'quiet'
        if self.verbosity.is_some() {
            let level = "success".green().to_string();
            self.fmt_print(&level, text)
        }
    }

    /// Print something to stdout (unless -q)
    pub fn stdout<S: Into<String>>(&self, text: S) {
        if self.verbosity.is_some() {
            print!("{}", text.into());
        }
    }

    /// Print something to stderr (unless -q)
    pub fn stderr<S: Into<String>>(&self, text: S) {
        if self.verbosity.is_some() {
            eprint!("{}", text.into());
        }
    }

    pub fn error<S: Into<String>>(&self, text: S) {
        if self.verbosity.is_some_and(|v| v >= Level::Error) {
            let level = "error".red().to_string();
            self.fmt_print(&level, text)
        }
    }

    pub fn warn<S: Into<String>>(&self, text: S) {
        if self.verbosity.is_some_and(|v| v >= Level::Warn) {
            let level = "warn".yellow().to_string();
            self.fmt_print(&level, text)
        }
    }

    pub fn info<S: Into<String>>(&self, text: S) {
        if self.verbosity.is_some_and(|v| v >= Level::Info) {
            let level = "info".blue().to_string();
            self.fmt_print(&level, text)
        }
    }
}
