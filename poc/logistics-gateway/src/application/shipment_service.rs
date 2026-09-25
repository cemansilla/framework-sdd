use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{
    CarrierAdapter, CarrierError, Quote, RepositoryError, Shipment, ShipmentRepository,
    ShipmentStatus, Tracking, TrackingRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum ShipmentServiceError {
    #[error("Carrier error: {0}")]
    CarrierError(#[from] CarrierError),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    #[error("Quote not found: {0}")]
    QuoteNotFound(Uuid),
    #[error("Quote expired: {0}")]
    QuoteExpired(Uuid),
    #[error("Shipment not found: {0}")]
    ShipmentNotFound(Uuid),
    #[error("Carrier not found for quote: {0}")]
    CarrierNotFound(String),
}

pub struct ShipmentService {
    carriers: Vec<Arc<dyn CarrierAdapter>>,
    shipment_repo: Arc<dyn ShipmentRepository>,
    tracking_repo: Arc<dyn TrackingRepository>,
}

impl ShipmentService {
    pub fn new(
        carriers: Vec<Arc<dyn CarrierAdapter>>,
        shipment_repo: Arc<dyn ShipmentRepository>,
        tracking_repo: Arc<dyn TrackingRepository>,
    ) -> Self {
        Self {
            carriers,
            shipment_repo,
            tracking_repo,
        }
    }

    pub async fn create_shipment(&self, quote: &Quote) -> Result<Shipment, ShipmentServiceError> {
        // Validate quote
        if !quote.is_valid() {
            return Err(ShipmentServiceError::QuoteExpired(quote.id));
        }

        // Find carrier
        let carrier = self
            .carriers
            .iter()
            .find(|c| c.name() == quote.carrier)
            .ok_or_else(|| ShipmentServiceError::CarrierNotFound(quote.carrier.clone()))?;

        // Create shipment with carrier
        let shipment = carrier.create_shipment(quote).await?;

        // Save shipment
        self.shipment_repo.save(&shipment).await?;

        // Initialize tracking
        let tracking = Tracking::new(
            shipment.id,
            ShipmentStatus::Created,
            "Shipment created",
        );
        self.tracking_repo.save(&tracking).await?;

        Ok(shipment)
    }

    pub async fn get_shipment(&self, id: Uuid) -> Result<Shipment, ShipmentServiceError> {
        self.shipment_repo
            .find_by_id(id)
            .await?
            .ok_or(ShipmentServiceError::ShipmentNotFound(id))
    }

    pub async fn get_tracking(&self, shipment_id: Uuid) -> Result<Tracking, ShipmentServiceError> {
        let shipment = self.get_shipment(shipment_id).await?;

        // Find carrier
        let carrier = self
            .carriers
            .iter()
            .find(|c| c.name() == shipment.carrier)
            .ok_or_else(|| ShipmentServiceError::CarrierNotFound(shipment.carrier.clone()))?;

        // Get tracking from carrier
        let tracking = carrier.tracking(&shipment.tracking_number).await?;

        // Update tracking in repository
        self.tracking_repo.save(&tracking).await?;

        Ok(tracking)
    }

    pub async fn list_shipments(&self) -> Result<Vec<Shipment>, ShipmentServiceError> {
        Ok(self.shipment_repo.list().await?)
    }
}
