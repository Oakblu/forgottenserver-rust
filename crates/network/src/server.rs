//! Service manager for network listeners.
//!
//! Migrated from forgottenserver server.h / server.cpp.

// ---------------------------------------------------------------------------
// ServiceConfig
// ---------------------------------------------------------------------------

/// Configuration for a single network service (listener).
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub port: u16,
    pub name: String,
    /// Whether this service exclusively owns its port (no port sharing).
    ///
    /// C++: `ServiceBase::is_single_socket()` — when `true`, no other
    /// protocol may be registered on the same port.
    pub single_socket: bool,
}

impl ServiceConfig {
    /// Constructs a new `ServiceConfig` that allows port sharing.
    pub fn new(port: u16, name: impl Into<String>) -> Self {
        Self {
            port,
            name: name.into(),
            single_socket: false,
        }
    }

    /// Constructs a `ServiceConfig` that exclusively owns its port.
    pub fn single_socket(port: u16, name: impl Into<String>) -> Self {
        Self {
            port,
            name: name.into(),
            single_socket: true,
        }
    }
}

// ---------------------------------------------------------------------------
// AddServiceError
// ---------------------------------------------------------------------------

/// Errors returned by [`ServiceManager::add_service`].
#[derive(Debug, PartialEq, Eq)]
pub enum AddServiceError {
    /// Port 0 is not a valid service port.
    InvalidPort,
    /// The new service and an existing service on the same port conflict
    /// because at least one of them requires exclusive (single-socket) access.
    PortConflict {
        port: u16,
        existing_name: String,
        new_name: String,
    },
}

impl std::fmt::Display for AddServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPort => write!(f, "port 0 is not valid"),
            Self::PortConflict {
                port,
                existing_name,
                new_name,
            } => write!(f, "{new_name} and {existing_name} cannot share port {port}"),
        }
    }
}

// ---------------------------------------------------------------------------
// ServiceManager
// ---------------------------------------------------------------------------

/// Manages a collection of network services and tracks connection statistics.
///
/// Mirrors the C++ `ServiceManager` class from `server.h / server.cpp`.
pub struct ServiceManager {
    services: Vec<ServiceConfig>,
    is_running: bool,
    connection_count: u32,
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceManager {
    /// Creates a new `ServiceManager` with no services, not running, and zero
    /// connections.
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            is_running: false,
            connection_count: 0,
        }
    }

    /// Registers a new service on the given port.
    ///
    /// # Errors
    ///
    /// Returns [`AddServiceError::InvalidPort`] if `config.port == 0`.
    ///
    /// Returns [`AddServiceError::PortConflict`] when:
    /// - an existing service on the same port requires single-socket access, or
    /// - the new service requires single-socket access and any other service
    ///   is already registered on that port.
    ///
    /// C++: `ServiceManager::add<ProtocolType>()` rejects port 0 and rejects
    /// port sharing when either side calls `is_single_socket()`.
    pub fn add_service(&mut self, config: ServiceConfig) -> Result<(), AddServiceError> {
        if config.port == 0 {
            return Err(AddServiceError::InvalidPort);
        }

        // Check for conflicts with existing services on the same port.
        for existing in &self.services {
            if existing.port != config.port {
                continue;
            }
            // Conflict when either side is single-socket.
            if existing.single_socket || config.single_socket {
                return Err(AddServiceError::PortConflict {
                    port: config.port,
                    existing_name: existing.name.clone(),
                    new_name: config.name.clone(),
                });
            }
        }

        self.services.push(config);
        Ok(())
    }

    /// Returns the number of registered services.
    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    /// Returns a slice of all registered services.
    pub fn services(&self) -> &[ServiceConfig] {
        &self.services
    }

    /// Returns all services registered on the given port.
    pub fn services_on_port(&self, port: u16) -> Vec<&ServiceConfig> {
        self.services.iter().filter(|s| s.port == port).collect()
    }

    /// Returns `true` if any service is registered on the given port.
    pub fn has_port(&self, port: u16) -> bool {
        self.services.iter().any(|s| s.port == port)
    }

    /// Marks the manager as running.
    ///
    /// C++: `ServiceManager::run()` — starts the io_context event loop.
    pub fn start(&mut self) {
        self.is_running = true;
    }

    /// Marks the manager as stopped and clears all registered services.
    ///
    /// C++: `ServiceManager::stop()` — signals all `ServicePort`s via
    /// `onStopServer()` and clears the acceptor map.
    pub fn stop(&mut self) {
        if !self.is_running {
            return;
        }
        self.is_running = false;
        self.services.clear();
    }

    /// Returns `true` if the manager is running.
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Returns the current number of active connections.
    pub fn get_connection_count(&self) -> u32 {
        self.connection_count
    }

    /// Increments the connection counter by one.
    pub fn increment_connection_count(&mut self) {
        self.connection_count = self.connection_count.saturating_add(1);
    }

    /// Decrements the connection counter by one, saturating at zero.
    pub fn decrement_connection_count(&mut self) {
        self.connection_count = self.connection_count.saturating_sub(1);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service(port: u16, name: &str) -> ServiceConfig {
        ServiceConfig::new(port, name)
    }

    fn make_single_socket(port: u16, name: &str) -> ServiceConfig {
        ServiceConfig::single_socket(port, name)
    }

    // -----------------------------------------------------------------------
    // new → empty, not running, zero connections
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_has_no_services() {
        let mgr = ServiceManager::new();
        assert_eq!(mgr.service_count(), 0);
    }

    #[test]
    fn test_new_is_not_running() {
        let mgr = ServiceManager::new();
        assert!(!mgr.is_running());
    }

    #[test]
    fn test_new_has_zero_connections() {
        let mgr = ServiceManager::new();
        assert_eq!(mgr.get_connection_count(), 0);
    }

    // -----------------------------------------------------------------------
    // add_service happy path
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_service_increments_count() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_service(7171, "game")).unwrap();
        assert_eq!(mgr.service_count(), 1);
        mgr.add_service(make_service(7172, "login")).unwrap();
        assert_eq!(mgr.service_count(), 2);
    }

    #[test]
    fn test_add_two_shared_socket_services_on_same_port_succeeds() {
        let mut mgr = ServiceManager::new();
        // Two non-single-socket services may share a port.
        mgr.add_service(make_service(7171, "game")).unwrap();
        mgr.add_service(make_service(7171, "oldgame")).unwrap();
        assert_eq!(mgr.service_count(), 2);
    }

    // -----------------------------------------------------------------------
    // add_service error cases (C++ equivalent checks)
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_service_port_zero_returns_invalid_port() {
        let mut mgr = ServiceManager::new();
        let result = mgr.add_service(make_service(0, "bad"));
        assert_eq!(result, Err(AddServiceError::InvalidPort));
    }

    #[test]
    fn test_add_service_port_zero_does_not_register() {
        let mut mgr = ServiceManager::new();
        let _ = mgr.add_service(make_service(0, "bad"));
        assert_eq!(mgr.service_count(), 0);
    }

    #[test]
    fn test_add_single_socket_service_when_port_occupied_returns_conflict() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_service(7171, "game")).unwrap();
        // Adding a single-socket service on the same port must fail.
        let result = mgr.add_service(make_single_socket(7171, "exclusive"));
        assert_eq!(
            result,
            Err(AddServiceError::PortConflict {
                port: 7171,
                existing_name: "game".into(),
                new_name: "exclusive".into(),
            })
        );
    }

    #[test]
    fn test_add_shared_socket_service_when_port_has_single_socket_returns_conflict() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_single_socket(7171, "exclusive"))
            .unwrap();
        // Adding any service to a single-socket port must fail.
        let result = mgr.add_service(make_service(7171, "other"));
        assert_eq!(
            result,
            Err(AddServiceError::PortConflict {
                port: 7171,
                existing_name: "exclusive".into(),
                new_name: "other".into(),
            })
        );
    }

    #[test]
    fn test_port_conflict_error_contains_service_names() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_service(7171, "alpha")).unwrap();
        let result = mgr.add_service(make_single_socket(7171, "beta"));
        assert_eq!(
            result,
            Err(AddServiceError::PortConflict {
                port: 7171,
                existing_name: "alpha".into(),
                new_name: "beta".into(),
            })
        );
    }

    #[test]
    fn test_conflict_does_not_register_service() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_single_socket(7171, "exclusive"))
            .unwrap();
        let _ = mgr.add_service(make_service(7171, "other"));
        assert_eq!(mgr.service_count(), 1); // only the first one
    }

    // -----------------------------------------------------------------------
    // has_port / services_on_port
    // -----------------------------------------------------------------------

    #[test]
    fn test_has_port_false_when_no_service() {
        let mgr = ServiceManager::new();
        assert!(!mgr.has_port(7171));
    }

    #[test]
    fn test_has_port_true_after_add() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_service(7171, "game")).unwrap();
        assert!(mgr.has_port(7171));
    }

    #[test]
    fn test_services_on_port_returns_matching() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_service(7171, "game")).unwrap();
        mgr.add_service(make_service(7172, "login")).unwrap();
        let on_7171 = mgr.services_on_port(7171);
        assert_eq!(on_7171.len(), 1);
        assert_eq!(on_7171[0].name, "game");
    }

    #[test]
    fn test_services_on_port_empty_for_unknown_port() {
        let mgr = ServiceManager::new();
        assert!(mgr.services_on_port(9999).is_empty());
    }

    // -----------------------------------------------------------------------
    // start / stop
    // -----------------------------------------------------------------------

    #[test]
    fn test_start_marks_running() {
        let mut mgr = ServiceManager::new();
        mgr.start();
        assert!(mgr.is_running());
    }

    #[test]
    fn test_stop_marks_not_running() {
        let mut mgr = ServiceManager::new();
        mgr.start();
        mgr.stop();
        assert!(!mgr.is_running());
    }

    #[test]
    fn test_stop_clears_all_services() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_service(7171, "game")).unwrap();
        mgr.add_service(make_service(7172, "login")).unwrap();
        mgr.start();
        mgr.stop();
        assert_eq!(mgr.service_count(), 0);
    }

    #[test]
    fn test_stop_when_not_running_is_noop() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_service(7171, "game")).unwrap();
        // stop without start should be a no-op (services remain)
        mgr.stop();
        assert!(!mgr.is_running());
        assert_eq!(mgr.service_count(), 1); // untouched
    }

    #[test]
    fn test_stop_idempotent() {
        let mut mgr = ServiceManager::new();
        mgr.start();
        mgr.stop();
        mgr.stop(); // second stop must not panic
        assert!(!mgr.is_running());
    }

    // -----------------------------------------------------------------------
    // connection count increment / decrement
    // -----------------------------------------------------------------------

    #[test]
    fn test_increment_connection_count() {
        let mut mgr = ServiceManager::new();
        mgr.increment_connection_count();
        assert_eq!(mgr.get_connection_count(), 1);
        mgr.increment_connection_count();
        assert_eq!(mgr.get_connection_count(), 2);
    }

    #[test]
    fn test_decrement_connection_count() {
        let mut mgr = ServiceManager::new();
        mgr.increment_connection_count();
        mgr.increment_connection_count();
        mgr.decrement_connection_count();
        assert_eq!(mgr.get_connection_count(), 1);
    }

    #[test]
    fn test_decrement_saturates_at_zero() {
        let mut mgr = ServiceManager::new();
        mgr.decrement_connection_count(); // already 0, should not underflow
        assert_eq!(mgr.get_connection_count(), 0);
    }

    #[test]
    fn test_connection_count_not_affected_by_stop() {
        let mut mgr = ServiceManager::new();
        mgr.increment_connection_count();
        mgr.increment_connection_count();
        mgr.start();
        mgr.stop();
        // stop only clears services, not live connection count
        assert_eq!(mgr.get_connection_count(), 2);
    }

    // -----------------------------------------------------------------------
    // services slice
    // -----------------------------------------------------------------------

    #[test]
    fn test_services_returns_registered_services() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_service(7171, "game")).unwrap();
        let services = mgr.services();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].port, 7171);
        assert_eq!(services[0].name, "game");
    }

    // -----------------------------------------------------------------------
    // ServiceConfig constructors
    // -----------------------------------------------------------------------

    #[test]
    fn test_service_config_new_not_single_socket() {
        let cfg = ServiceConfig::new(7171, "game");
        assert!(!cfg.single_socket);
    }

    #[test]
    fn test_service_config_single_socket_flag_set() {
        let cfg = ServiceConfig::single_socket(7171, "exclusive");
        assert!(cfg.single_socket);
    }

    // -----------------------------------------------------------------------
    // AddServiceError display
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_service_error_invalid_port_display() {
        let err = AddServiceError::InvalidPort;
        assert!(err.to_string().contains("0"));
    }

    #[test]
    fn test_add_service_error_port_conflict_display() {
        let err = AddServiceError::PortConflict {
            port: 7171,
            existing_name: "alpha".into(),
            new_name: "beta".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("7171"));
        assert!(msg.contains("alpha"));
        assert!(msg.contains("beta"));
    }

    // -----------------------------------------------------------------------
    // Default impl (C++ default-constructed ServiceManager: empty, !running)
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_matches_new() {
        let mgr: ServiceManager = Default::default();
        assert_eq!(mgr.service_count(), 0);
        assert!(!mgr.is_running());
        assert_eq!(mgr.get_connection_count(), 0);
    }

    // -----------------------------------------------------------------------
    // has_port branch coverage — exercise both true and false predicate paths
    // when multiple services are registered.
    // C++ analog: ServiceManager::acceptors.find(port) hit-and-miss in add<>().
    // -----------------------------------------------------------------------

    #[test]
    fn test_has_port_scans_through_non_matching_services() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_service(7171, "game")).unwrap();
        mgr.add_service(make_service(7172, "login")).unwrap();
        mgr.add_service(make_service(7173, "status")).unwrap();
        // Force the predicate to evaluate `false` then `true` then short-circuit.
        assert!(mgr.has_port(7173));
        // And evaluate `false` for every entry.
        assert!(!mgr.has_port(9999));
    }

    // -----------------------------------------------------------------------
    // Port → service routing audit (C++: ServicePort::make_protocol matches
    // protocolID against services). Rust analog is `services_on_port` lookup.
    // -----------------------------------------------------------------------

    #[test]
    fn test_services_on_port_returns_all_shared_services_in_insertion_order() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_service(7171, "game")).unwrap();
        mgr.add_service(make_service(7171, "oldgame")).unwrap();
        mgr.add_service(make_service(7172, "login")).unwrap();
        let on_7171 = mgr.services_on_port(7171);
        assert_eq!(on_7171.len(), 2);
        assert_eq!(on_7171[0].name, "game");
        assert_eq!(on_7171[1].name, "oldgame");
    }

    #[test]
    fn test_login_game_and_status_services_can_coexist_on_distinct_ports() {
        // Mirrors ServiceManager::add<LoginProtocol/GameProtocol/StatusProtocol>
        // from server.h's templated registration — each is bound to its own port.
        let mut mgr = ServiceManager::new();
        mgr.add_service(ServiceConfig::single_socket(7171, "login"))
            .unwrap();
        mgr.add_service(ServiceConfig::new(7172, "game")).unwrap();
        mgr.add_service(ServiceConfig::new(7172, "status")).unwrap();
        assert_eq!(mgr.service_count(), 3);
        assert!(mgr.has_port(7171));
        assert_eq!(mgr.services_on_port(7172).len(), 2);
    }

    // -----------------------------------------------------------------------
    // C++ ServicePort::add_service rejects ANY addition once a single-socket
    // service is present, regardless of ordering. Verify both orderings.
    // -----------------------------------------------------------------------

    #[test]
    fn test_single_socket_then_single_socket_same_port_conflicts() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_single_socket(7171, "first")).unwrap();
        let result = mgr.add_service(make_single_socket(7171, "second"));
        assert_eq!(
            result,
            Err(AddServiceError::PortConflict {
                port: 7171,
                existing_name: "first".into(),
                new_name: "second".into(),
            })
        );
    }

    // -----------------------------------------------------------------------
    // C++ ServiceManager::is_running == !acceptors.empty(). After stop() the
    // services map is cleared and connection counters are independent.
    // -----------------------------------------------------------------------

    #[test]
    fn test_restart_after_stop_registers_fresh_services() {
        let mut mgr = ServiceManager::new();
        mgr.add_service(make_service(7171, "game")).unwrap();
        mgr.start();
        mgr.stop();
        assert_eq!(mgr.service_count(), 0);
        // Re-registration after stop must succeed.
        mgr.add_service(make_service(7171, "game")).unwrap();
        assert_eq!(mgr.service_count(), 1);
    }
}
