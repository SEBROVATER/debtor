use std::sync::Arc;

use async_trait::async_trait;

use crate::{ApplicationError, UnavailableReason};

/// Checks the health of the application's local database dependency.
#[async_trait]
pub trait DatabaseReadiness: Send + Sync {
    /// Acquires a database connection and performs a trivial non-monetary query.
    async fn check(&self) -> Result<(), ApplicationError>;
}

/// Inbound readiness use case.
#[async_trait]
pub trait ReadinessUseCases: Send + Sync {
    /// Checks mandatory local dependencies required to serve the application.
    async fn check(&self) -> Result<(), ApplicationError>;
}

/// Reports whether a mandatory in-process supervisor is healthy.
pub trait SupervisorReadiness: Send + Sync {
    /// Returns whether the supervisor can continue serving the process.
    fn is_healthy(&self) -> bool;
}

/// Readiness workflow backed by a narrow database port.
pub struct ReadinessService {
    database: Arc<dyn DatabaseReadiness>,
    supervisor: Option<Arc<dyn SupervisorReadiness>>,
}

impl ReadinessService {
    /// Creates a readiness service with an injected database dependency.
    pub fn new(database: Arc<dyn DatabaseReadiness>) -> Self {
        Self {
            database,
            supervisor: None,
        }
    }

    /// Creates a readiness service with a mandatory supervisor health source.
    pub fn with_supervisor(
        database: Arc<dyn DatabaseReadiness>,
        supervisor: Arc<dyn SupervisorReadiness>,
    ) -> Self {
        Self {
            database,
            supervisor: Some(supervisor),
        }
    }
}

#[async_trait]
impl ReadinessUseCases for ReadinessService {
    async fn check(&self) -> Result<(), ApplicationError> {
        if self
            .supervisor
            .as_ref()
            .is_some_and(|supervisor| !supervisor.is_healthy())
        {
            return Err(ApplicationError::Unavailable(
                UnavailableReason::RuntimeSupervisor,
            ));
        }
        self.database.check().await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::*;
    use crate::StorageReason;

    struct Fake(Result<(), ApplicationError>);

    #[async_trait]
    impl DatabaseReadiness for Fake {
        async fn check(&self) -> Result<(), ApplicationError> {
            match &self.0 {
                Ok(()) => Ok(()),
                Err(ApplicationError::Storage(reason)) => Err(ApplicationError::Storage(*reason)),
                Err(_) => Err(ApplicationError::Storage(StorageReason::Unexpected)),
            }
        }
    }

    struct Supervisor(bool);

    impl SupervisorReadiness for Supervisor {
        fn is_healthy(&self) -> bool {
            self.0
        }
    }

    #[tokio::test]
    async fn delegates_to_the_database_port() {
        let healthy = ReadinessService::new(Arc::new(Fake(Ok(()))));
        assert!(healthy.check().await.is_ok());

        let failed = ReadinessService::new(Arc::new(Fake(Err(ApplicationError::Storage(
            StorageReason::Unexpected,
        )))));
        assert!(matches!(
            failed.check().await,
            Err(ApplicationError::Storage(StorageReason::Unexpected))
        ));

        let unhealthy =
            ReadinessService::with_supervisor(Arc::new(Fake(Ok(()))), Arc::new(Supervisor(false)));
        assert!(matches!(
            unhealthy.check().await,
            Err(ApplicationError::Unavailable(
                UnavailableReason::RuntimeSupervisor
            ))
        ));
    }
}
