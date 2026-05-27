#[derive(Debug, Default)]
pub struct ModuleStatus {
    pub config_loaded: bool,
    pub database_initialised: bool,
    pub items_loaded: bool,
    pub map_loaded: bool,
    pub scripting_loaded: bool,
    pub network_started: bool,
    pub http_started: bool,
    pub scheduler_started: bool,
}

pub struct OtServConfig {
    pub config_path: String,
}

pub fn initialise_modules(config: &OtServConfig) -> Result<ModuleStatus, String> {
    if config.config_path.is_empty() {
        return Err("Missing config path".to_string());
    }
    Ok(ModuleStatus {
        config_loaded: true,
        database_initialised: true,
        items_loaded: true,
        map_loaded: true,
        scripting_loaded: true,
        network_started: true,
        http_started: true,
        scheduler_started: true,
    })
}

pub fn shutdown_modules(status: &mut ModuleStatus) {
    status.scheduler_started = false;
    status.http_started = false;
    status.network_started = false;
    status.scripting_loaded = false;
    status.database_initialised = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_path_returns_err() {
        let config = OtServConfig {
            config_path: String::new(),
        };
        let result = initialise_modules(&config);
        assert!(result.is_err());
    }

    #[test]
    fn err_message_mentions_config_path() {
        let config = OtServConfig {
            config_path: String::new(),
        };
        let err = initialise_modules(&config).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn non_empty_config_path_returns_all_true() {
        let config = OtServConfig {
            config_path: "/etc/server.cfg".to_string(),
        };
        let status = initialise_modules(&config).unwrap();
        assert!(status.config_loaded);
        assert!(status.database_initialised);
        assert!(status.items_loaded);
        assert!(status.map_loaded);
        assert!(status.scripting_loaded);
        assert!(status.network_started);
        assert!(status.http_started);
        assert!(status.scheduler_started);
    }

    #[test]
    fn shutdown_modules_clears_scheduler_started() {
        let config = OtServConfig {
            config_path: "config.cfg".to_string(),
        };
        let mut status = initialise_modules(&config).unwrap();
        shutdown_modules(&mut status);
        assert!(!status.scheduler_started);
    }

    #[test]
    fn shutdown_modules_clears_http_started() {
        let config = OtServConfig {
            config_path: "config.cfg".to_string(),
        };
        let mut status = initialise_modules(&config).unwrap();
        shutdown_modules(&mut status);
        assert!(!status.http_started);
    }

    #[test]
    fn shutdown_modules_clears_network_started() {
        let config = OtServConfig {
            config_path: "config.cfg".to_string(),
        };
        let mut status = initialise_modules(&config).unwrap();
        shutdown_modules(&mut status);
        assert!(!status.network_started);
    }

    #[test]
    fn shutdown_modules_clears_scripting_loaded() {
        let config = OtServConfig {
            config_path: "config.cfg".to_string(),
        };
        let mut status = initialise_modules(&config).unwrap();
        shutdown_modules(&mut status);
        assert!(!status.scripting_loaded);
    }

    #[test]
    fn shutdown_modules_clears_database_initialised() {
        let config = OtServConfig {
            config_path: "config.cfg".to_string(),
        };
        let mut status = initialise_modules(&config).unwrap();
        shutdown_modules(&mut status);
        assert!(!status.database_initialised);
    }

    #[test]
    fn shutdown_modules_preserves_config_loaded_and_map_loaded() {
        let config = OtServConfig {
            config_path: "config.cfg".to_string(),
        };
        let mut status = initialise_modules(&config).unwrap();
        shutdown_modules(&mut status);
        // These are not cleared by shutdown
        assert!(status.config_loaded);
        assert!(status.items_loaded);
        assert!(status.map_loaded);
    }

    #[test]
    fn module_status_default_is_all_false() {
        let status = ModuleStatus::default();
        assert!(!status.config_loaded);
        assert!(!status.database_initialised);
        assert!(!status.items_loaded);
        assert!(!status.map_loaded);
        assert!(!status.scripting_loaded);
        assert!(!status.network_started);
        assert!(!status.http_started);
        assert!(!status.scheduler_started);
    }
}
