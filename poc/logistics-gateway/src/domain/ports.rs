use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use super::entities::{Quote, QuoteRequest, Shipment, Tracking};

#[allow(dead_code)]
#[derive(Debug, Clone, Error)]
pub enum CarrierError {
    #[error("Carrier unavailable: {0}")]
    Unavailable(String),
    #[error("Quote not available: {0}")]
    QuoteNotAvailable(String),
    #[error("External error: {0}")]
    ExternalError(String),
    #[error("Timeout")]
    Timeout,
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

#[async_trait]
pub trait CarrierAdapter: Send + Sync {
    fn name(&self) -> &str;

    async fn quote(&self, request: &QuoteRequest) -> Result<Quote, CarrierError>;

    async fn create_shipment(&self, quote: &Quote) -> Result<Shipment, CarrierError>;

    async fn tracking(&self, tracking_number: &str) -> Result<Tracking, CarrierError>;

    fn is_healthy(&self) -> bool;
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

#[async_trait]
pub trait QuoteRepository: Send + Sync {
    async fn save(&self, quote: &Quote) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Quote>, RepositoryError>;
    #[allow(dead_code)]
    async fn find_by_request_id(&self, request_id: Uuid) -> Result<Vec<Quote>, RepositoryError>;
    #[allow(dead_code)]
    async fn list(&self) -> Result<Vec<Quote>, RepositoryError>;
}

#[async_trait]
pub trait ShipmentRepository: Send + Sync {
    async fn save(&self, shipment: &Shipment) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Shipment>, RepositoryError>;
    #[allow(dead_code)]
    async fn find_by_quote_id(&self, quote_id: Uuid) -> Result<Option<Shipment>, RepositoryError>;
    async fn list(&self) -> Result<Vec<Shipment>, RepositoryError>;
    #[allow(dead_code)]
    async fn update_status(
        &self,
        id: Uuid,
        status: super::entities::ShipmentStatus,
    ) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait TrackingRepository: Send + Sync {
    async fn save(&self, tracking: &Tracking) -> Result<(), RepositoryError>;
    #[allow(dead_code)]
    async fn find_by_shipment_id(
        &self,
        shipment_id: Uuid,
    ) -> Result<Option<Tracking>, RepositoryError>;
}

use super::models::Money;

pub struct MarginContext {
    pub weight_kg: f64,
    pub origin_zone: String,
    pub destination_zone: String,
    pub base_price: Money,
}

pub trait MarginPolicy: Send + Sync {
    fn apply(
        &self,
        base_price: Money,
        context: &MarginContext,
    ) -> (Money, super::entities::MarginBreakdown);
}
