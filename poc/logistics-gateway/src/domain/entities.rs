use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::{Address, Money, Parcel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRequest {
    pub id: Uuid,
    pub origin: Address,
    pub destination: Address,
    pub parcel: Parcel,
    pub requested_at: DateTime<Utc>,
}

impl QuoteRequest {
    pub fn new(origin: Address, destination: Address, parcel: Parcel) -> Self {
        Self {
            id: Uuid::new_v4(),
            origin,
            destination,
            parcel,
            requested_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: Uuid,
    pub request_id: Uuid,
    pub carrier: String,
    pub origin: Address,
    pub destination: Address,
    pub parcel: Parcel,
    pub base_price: Money,
    pub final_price: Money,
    pub estimated_days: u32,
    pub valid_until: DateTime<Utc>,
    pub margin_breakdown: Option<MarginBreakdown>,
    pub created_at: DateTime<Utc>,
}

impl Quote {
    pub fn new(
        request: &QuoteRequest,
        carrier: impl Into<String>,
        base_price: Money,
        estimated_days: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            request_id: request.id,
            carrier: carrier.into(),
            origin: request.origin.clone(),
            destination: request.destination.clone(),
            parcel: request.parcel.clone(),
            base_price: base_price.clone(),
            final_price: base_price,
            estimated_days,
            valid_until: now + chrono::Duration::hours(24),
            margin_breakdown: None,
            created_at: now,
        }
    }

    pub fn is_valid(&self) -> bool {
        Utc::now() < self.valid_until
    }

    pub fn with_final_price(mut self, final_price: Money, breakdown: MarginBreakdown) -> Self {
        self.final_price = final_price;
        self.margin_breakdown = Some(breakdown);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginBreakdown {
    pub base_price: Money,
    pub adjustments: Vec<MarginAdjustment>,
    pub total_margin: Money,
}

impl MarginBreakdown {
    pub fn new(base_price: Money) -> Self {
        Self {
            total_margin: Money::from_f64(0.0, &base_price.currency),
            base_price,
            adjustments: Vec::new(),
        }
    }

    pub fn add_adjustment(&mut self, name: String, amount: Money) {
        self.total_margin = self.total_margin.add(&amount);
        self.adjustments.push(MarginAdjustment { name, amount });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginAdjustment {
    pub name: String,
    pub amount: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShipmentStatus {
    Created,
    PickedUp,
    InTransit,
    OutForDelivery,
    Delivered,
    Failed,
    Cancelled,
}

impl std::fmt::Display for ShipmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipmentStatus::Created => write!(f, "created"),
            ShipmentStatus::PickedUp => write!(f, "picked_up"),
            ShipmentStatus::InTransit => write!(f, "in_transit"),
            ShipmentStatus::OutForDelivery => write!(f, "out_for_delivery"),
            ShipmentStatus::Delivered => write!(f, "delivered"),
            ShipmentStatus::Failed => write!(f, "failed"),
            ShipmentStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shipment {
    pub id: Uuid,
    pub quote_id: Uuid,
    pub carrier: String,
    pub tracking_number: String,
    pub status: ShipmentStatus,
    pub origin: Address,
    pub destination: Address,
    pub parcel: Parcel,
    pub created_at: DateTime<Utc>,
    pub estimated_delivery: DateTime<Utc>,
}

impl Shipment {
    pub fn new(quote: &Quote, tracking_number: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            quote_id: quote.id,
            carrier: quote.carrier.clone(),
            tracking_number: tracking_number.into(),
            status: ShipmentStatus::Created,
            origin: quote.origin.clone(),
            destination: quote.destination.clone(),
            parcel: quote.parcel.clone(),
            created_at: now,
            estimated_delivery: now + chrono::Duration::days(quote.estimated_days as i64),
        }
    }

    #[allow(dead_code)]
    pub fn update_status(&mut self, status: ShipmentStatus) {
        self.status = status;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracking {
    pub shipment_id: Uuid,
    pub status: ShipmentStatus,
    pub status_description: String,
    pub location: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub history: Vec<TrackingEvent>,
}

impl Tracking {
    pub fn new(shipment_id: Uuid, status: ShipmentStatus, description: impl Into<String>) -> Self {
        Self {
            shipment_id,
            status: status.clone(),
            status_description: description.into(),
            location: None,
            updated_at: Utc::now(),
            history: vec![TrackingEvent {
                status,
                description: "Shipment created".to_string(),
                location: None,
                timestamp: Utc::now(),
            }],
        }
    }

    pub fn add_event(
        &mut self,
        status: ShipmentStatus,
        description: impl Into<String>,
        location: Option<String>,
    ) {
        let description_str = description.into();
        self.status = status.clone();
        self.status_description = description_str.clone();
        self.location = location.clone();
        self.updated_at = Utc::now();
        self.history.push(TrackingEvent {
            status,
            description: description_str,
            location,
            timestamp: Utc::now(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEvent {
    pub status: ShipmentStatus,
    pub description: String,
    pub location: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Carrier {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub capabilities: Vec<CarrierCapability>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CarrierCapability {
    Quote,
    Shipment,
    Tracking,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Address, Dimensions, Parcel};

    #[test]
    fn test_quote_creation() {
        let request = QuoteRequest::new(
            Address::new("Street 1", "City", "State", "12345", "AR"),
            Address::new("Street 2", "City", "State", "12345", "AR"),
            Parcel::new(2.5, Dimensions::new(30.0, 20.0, 15.0).unwrap()).unwrap(),
        );
        let base_price = Money::from_f64(1000.0, "ARS");
        let quote = Quote::new(&request, "oca", base_price.clone(), 3);

        assert_eq!(quote.request_id, request.id);
        assert_eq!(quote.carrier, "oca");
        assert_eq!(quote.base_price.amount.to_string(), "1000");
        assert_eq!(quote.estimated_days, 3);
        assert!(quote.is_valid());
    }

    #[test]
    fn test_shipment_creation() {
        let request = QuoteRequest::new(
            Address::new("Street 1", "City", "State", "12345", "AR"),
            Address::new("Street 2", "City", "State", "12345", "AR"),
            Parcel::new(2.5, Dimensions::new(30.0, 20.0, 15.0).unwrap()).unwrap(),
        );
        let base_price = Money::from_f64(1000.0, "ARS");
        let quote = Quote::new(&request, "oca", base_price, 3);

        let shipment = Shipment::new(&quote, "OCA-123456");
        assert_eq!(shipment.quote_id, quote.id);
        assert_eq!(shipment.carrier, "oca");
        assert_eq!(shipment.tracking_number, "OCA-123456");
        assert_eq!(shipment.status, ShipmentStatus::Created);
    }

    #[test]
    fn test_tracking_creation() {
        let shipment_id = Uuid::new_v4();
        let tracking = Tracking::new(shipment_id, ShipmentStatus::Created, "Shipment created");

        assert_eq!(tracking.shipment_id, shipment_id);
        assert_eq!(tracking.status, ShipmentStatus::Created);
        assert_eq!(tracking.history.len(), 1);
    }

    #[test]
    fn test_tracking_event_addition() {
        let shipment_id = Uuid::new_v4();
        let mut tracking = Tracking::new(shipment_id, ShipmentStatus::Created, "Created");

        tracking.add_event(
            ShipmentStatus::InTransit,
            "In transit",
            Some("Warehouse".to_string()),
        );

        assert_eq!(tracking.status, ShipmentStatus::InTransit);
        assert_eq!(tracking.history.len(), 2);
    }
}
