use std::borrow::BorrowMut;

use clap_verbosity_flag::Level;
use color_eyre::owo_colors::OwoColorize;

const PREFIX: &'static str = ">> ntfy";

pub struct Logger {
    pub prefix: Option<String>,
    pub verbosity: Option<Level>,
}

impl Logger {
    pub fn new(verbosity: &clap_verbosity_flag::Verbosity) -> Self {
        return Logger {
            prefix: Some(PREFIX.to_string()),
            verbosity: verbosity.log_level(),
        };
    }

    pub fn new_with_prefix(verbosity: &clap_verbosity_flag::Verbosity, prefix: &str) -> Self {
        return Logger {
            prefix: Some(prefix.to_string()),
            verbosity: verbosity.log_level(),
        };
    }

    const fn empty() -> Self {
        return Logger {
            prefix: None,
            verbosity: None,
        };
    }

    fn fmt_print<S: Into<String>>(&self, level: &str, text: S) {
        match &self.prefix {
            None => eprintln!(
                "{}",
                format!("{} | {}", level, text.into())
            ),
            Some(prefix) => eprintln!(
                "{}",
                format!("{} | {} | {}", prefix, level, text.into())
            )
        }
        
    }

    /// log without a level
    pub fn log<S: Into<String>>(&self, text: S) {
        // only hide if 'quiet'
        if self.verbosity.is_some() {
            match &self.prefix {
                None => eprintln!("{}", format!("{}", text.into())),
                Some(prefix) => eprintln!("{}", format!("{} | {}", prefix, text.into()))
            }

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

    pub fn debug<S: Into<String>>(&self, text: S) {
        if self.verbosity.is_some_and(|v| v >= Level::Debug) {
            let level = "debug".purple().to_string();
            self.fmt_print(&level, text)
        }
    }
}

// == global logger == //

pub struct GlobalLogger {
    _logger: Logger,
}

static mut GLOBAL_LOGGER: GlobalLogger = GlobalLogger::empty();

impl GlobalLogger {
    const fn empty() -> Self {
        return GlobalLogger {
            _logger: Logger::empty(),
        };
    }

    pub fn singleton() -> &'static Logger {
        unsafe {
            return &GLOBAL_LOGGER._logger;
        }
    }

    pub fn setup(verbosity: &clap_verbosity_flag::Verbosity) -> &'static Logger {
        GlobalLogger::set_verbosity(verbosity);
        GlobalLogger::set_prefix(PREFIX);
        return GlobalLogger::singleton();
    }

    pub fn get_prefix() -> &'static Option<String> {
        unsafe {
            return &GLOBAL_LOGGER._logger.prefix;
        }
    }

    pub fn set_prefix(prefix: &str) -> &'static Logger {
        unsafe {
            *GLOBAL_LOGGER._logger.prefix.borrow_mut() = Some(prefix.to_string());
            return &GLOBAL_LOGGER._logger;
        }
    }

    pub fn get_verbosity() -> Option<Level> {
        unsafe {
            return GLOBAL_LOGGER._logger.verbosity;
        }
    }

    pub fn set_verbosity(verbosity: &clap_verbosity_flag::Verbosity) -> &'static Logger {
        unsafe {
            *GLOBAL_LOGGER._logger.verbosity.borrow_mut() = verbosity.log_level();
            return &GLOBAL_LOGGER._logger;
        }
    }

    pub fn log<S: Into<String>>(text: S) {
        return GlobalLogger::singleton().log(text);
    }

    pub fn success<S: Into<String>>(text: S) {
        return GlobalLogger::singleton().success(text);
    }
    pub fn warn<S: Into<String>>(text: S) {
        return GlobalLogger::singleton().warn(text);
    }

    pub fn error<S: Into<String>>(text: S) {
        return GlobalLogger::singleton().error(text);
    }
    pub fn info<S: Into<String>>(text: S) {
        return GlobalLogger::singleton().info(text);
    }

    pub fn debug<S: Into<String>>(text: S) {
        return GlobalLogger::singleton().debug(text);
    }

    pub fn stdout<S: Into<String>>(text: S) {
        return GlobalLogger::singleton().stdout(text);
    }

    pub fn stderr<S: Into<String>>(text: S) {
        return GlobalLogger::singleton().stderr(text);
    }
}

#[cfg(test)]
mod tests {

    use super::{GlobalLogger, Logger};

    #[test]
    fn test_instances() {
        let high_verbosity = clap_verbosity_flag::Verbosity::new(4.into(), 0.into());
        let local_logger = Logger::new(&high_verbosity);
        let prefixxed_logger = Logger::new_with_prefix(&high_verbosity, "! hi !");

        let global_logger = GlobalLogger::singleton();

        assert!(global_logger.verbosity != local_logger.verbosity);

        local_logger.log("Log 1");
        global_logger.log("Log 2");
        GlobalLogger::log("Log 3");
        prefixxed_logger.log("Log 4");

        local_logger.error("Err 1");
        global_logger.error("Err 2");
        GlobalLogger::error("Err 3");

        local_logger.warn("Warn 1");
        global_logger.warn("Warn 2");
        GlobalLogger::warn("Warn 3");

        local_logger.info("Info 1");
        global_logger.info("Info 2");
        GlobalLogger::info("Info 3");

        local_logger.debug("Dbg 1");
        global_logger.debug("Dbg 2");
        GlobalLogger::debug("Dbg 3");

        local_logger.success("Success 1");
        global_logger.success("Success 2");
        GlobalLogger::success("Success 3");

        local_logger.stdout("stdout 1");
        global_logger.stdout("stdout 2");
        GlobalLogger::stdout("stdout 3");

        local_logger.stderr("stderr 1");
        global_logger.stderr("stderr 2");
        GlobalLogger::stderr("stderr 3");
    }
}
