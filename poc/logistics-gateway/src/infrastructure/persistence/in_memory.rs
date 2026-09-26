use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{
    Quote, QuoteRepository, RepositoryError, Shipment, ShipmentRepository, ShipmentStatus,
    Tracking, TrackingRepository,
};

#[derive(Debug, Default)]
pub struct InMemoryQuoteRepository {
    quotes: Arc<RwLock<Vec<Quote>>>,
}

impl InMemoryQuoteRepository {
    pub fn new() -> Self {
        Self {
            quotes: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl QuoteRepository for InMemoryQuoteRepository {
    async fn save(&self, quote: &Quote) -> Result<(), RepositoryError> {
        let mut quotes = self.quotes.write().await;
        quotes.push(quote.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Quote>, RepositoryError> {
        let quotes = self.quotes.read().await;
        Ok(quotes.iter().find(|q| q.id == id).cloned())
    }

    async fn find_by_request_id(&self, request_id: Uuid) -> Result<Vec<Quote>, RepositoryError> {
        let quotes = self.quotes.read().await;
        Ok(quotes
            .iter()
            .filter(|q| q.request_id == request_id)
            .cloned()
            .collect())
    }

    async fn list(&self) -> Result<Vec<Quote>, RepositoryError> {
        let quotes = self.quotes.read().await;
        Ok(quotes.clone())
    }
}

#[derive(Debug, Default)]
pub struct InMemoryShipmentRepository {
    shipments: Arc<RwLock<Vec<Shipment>>>,
}

impl InMemoryShipmentRepository {
    pub fn new() -> Self {
        Self {
            shipments: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl ShipmentRepository for InMemoryShipmentRepository {
    async fn save(&self, shipment: &Shipment) -> Result<(), RepositoryError> {
        let mut shipments = self.shipments.write().await;
        shipments.push(shipment.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Shipment>, RepositoryError> {
        let shipments = self.shipments.read().await;
        Ok(shipments.iter().find(|s| s.id == id).cloned())
    }

    async fn find_by_quote_id(&self, quote_id: Uuid) -> Result<Option<Shipment>, RepositoryError> {
        let shipments = self.shipments.read().await;
        Ok(shipments.iter().find(|s| s.quote_id == quote_id).cloned())
    }

    async fn list(&self) -> Result<Vec<Shipment>, RepositoryError> {
        let shipments = self.shipments.read().await;
        Ok(shipments.clone())
    }

    async fn update_status(&self, id: Uuid, status: ShipmentStatus) -> Result<(), RepositoryError> {
        let mut shipments = self.shipments.write().await;
        if let Some(shipment) = shipments.iter_mut().find(|s| s.id == id) {
            shipment.status = status;
            Ok(())
        } else {
            Err(RepositoryError::NotFound(format!("Shipment {}", id)))
        }
    }
}

#[derive(Debug, Default)]
pub struct InMemoryTrackingRepository {
    trackings: Arc<RwLock<Vec<Tracking>>>,
}

impl InMemoryTrackingRepository {
    pub fn new() -> Self {
        Self {
            trackings: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl TrackingRepository for InMemoryTrackingRepository {
    async fn save(&self, tracking: &Tracking) -> Result<(), RepositoryError> {
        let mut trackings = self.trackings.write().await;
        // Update if exists, otherwise insert
        if let Some(existing) = trackings
            .iter_mut()
            .find(|t| t.shipment_id == tracking.shipment_id)
        {
            *existing = tracking.clone();
        } else {
            trackings.push(tracking.clone());
        }
        Ok(())
    }

    async fn find_by_shipment_id(
        &self,
        shipment_id: Uuid,
    ) -> Result<Option<Tracking>, RepositoryError> {
        let trackings = self.trackings.read().await;
        Ok(trackings
            .iter()
            .find(|t| t.shipment_id == shipment_id)
            .cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Address, Dimensions, Money, Parcel, Quote, QuoteRequest};

    #[tokio::test]
    async fn test_quote_repository() {
        let repo = InMemoryQuoteRepository::new();
        let request = QuoteRequest::new(
            Address::new("Street 1", "City", "State", "12345", "AR"),
            Address::new("Street 2", "City", "State", "12345", "AR"),
            Parcel::new(2.5, Dimensions::new(30.0, 20.0, 15.0).unwrap()).unwrap(),
        );

        let quote = Quote::new(&request, "oca", Money::from_f64(1000.0, "ARS"), 3);

        repo.save(&quote).await.unwrap();
        let found = repo.find_by_id(quote.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, quote.id);
    }

    #[tokio::test]
    async fn test_shipment_repository() {
        let repo = InMemoryShipmentRepository::new();
        let request = QuoteRequest::new(
            Address::new("Street 1", "City", "State", "12345", "AR"),
            Address::new("Street 2", "City", "State", "12345", "AR"),
            Parcel::new(2.5, Dimensions::new(30.0, 20.0, 15.0).unwrap()).unwrap(),
        );
        let quote = Quote::new(&request, "oca", Money::from_f64(1000.0, "ARS"), 3);
        let shipment = Shipment::new(&quote, "OCA-123");

        repo.save(&shipment).await.unwrap();
        let found = repo.find_by_id(shipment.id).await.unwrap();
        assert!(found.is_some());
    }
}
