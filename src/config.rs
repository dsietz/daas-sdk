use crate::*;

/// Trait for managing environment variables
pub trait EnvVarManager {
    /// Determines if an environment variable is set
    fn is_set(env_var: &str) -> bool {
        match env::var(env_var) {
            Ok(_ev) => true,
            Err(err) => {
                debug!("Could not find environment variable {}. Error: {}", env_var, err);
                false
            }
        }
    }
}

pub struct ServicePath {
    application: String,
    version: String,
    module: String,
    root_path: String,
}

impl ServicePath {
    pub fn new(application: String, module: String, version: String) -> ServicePath {
        ServicePath {
            application: application.clone(),
            version: version.clone(),
            module: module.clone(),
            root_path: format!("/api/{}/{}/{}",application, module, version),
        }
    }
}

pub struct Config {
    service_path: ServicePath,
}

impl EnvVarManager for Config {}

impl Config {
    pub fn new(application: String, module: String, version: String) -> Config {
        Config {
            service_path: ServicePath::new(application, module, version)
        }
    }

    pub fn get_root_path(&self) -> String {
        self.service_path.root_path.clone()
    }
}


/// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;

    struct TestEnvVarMngr{}

    impl EnvVarManager for TestEnvVarMngr{}

    #[test]
    fn test_config_ok() {
        let cfg = Config::new("daas".to_string(), "data".to_string(), "v1".to_string());

        assert_eq!(cfg.get_root_path(), "/api/daas/data/v1".to_string());
    }

    #[test]
    fn test_envvarmngr_is_set_false() {
        assert!(!TestEnvVarMngr::is_set("DUMMY_ENV_VAR_FALSE"));
    }

    #[test]
    fn test_envvarmngr_is_set_true() {
        env::set_var("DUMMY_ENV_VAR_TRUE", "VALUE");
        assert!(TestEnvVarMngr::is_set("DUMMY_ENV_VAR_TRUE"));
    }

    #[test]
    fn test_service_path_root_path_ok() {
        let sp = ServicePath::new("daas".to_string(), "data".to_string(), "v1".to_string());

        assert_eq!(sp.root_path, "/api/daas/data/v1".to_string());
    }
}