use async_trait::async_trait;
use rand::Rng;
use std::time::Duration;
use uuid::Uuid;

use crate::domain::{
    Address, CarrierAdapter, CarrierError, Dimensions, Parcel, Quote, QuoteRequest, Shipment,
    ShipmentStatus, Tracking,
};

#[derive(Debug, Clone)]
pub struct MockCarrierConfig {
    pub response_delay_ms: u64,
    pub failure_rate: f64,
    pub timeout_rate: f64,
    pub base_rate_per_kg: f64,
    pub distance_rate_per_km: f64,
}

impl Default for MockCarrierConfig {
    fn default() -> Self {
        Self {
            response_delay_ms: 100,
            failure_rate: 0.05,
            timeout_rate: 0.02,
            base_rate_per_kg: 100.0,
            distance_rate_per_km: 0.5,
        }
    }
}

pub struct MockCarrierAdapter {
    name: String,
    config: MockCarrierConfig,
}

impl MockCarrierAdapter {
    pub fn new(name: impl Into<String>, config: MockCarrierConfig) -> Self {
        Self {
            name: name.into(),
            config,
        }
    }

    fn calculate_distance(&self, origin: &Address, destination: &Address) -> u32 {
        // Simplified distance calculation
        if origin.city == destination.city {
            10
        } else if origin.state == destination.state {
            100
        } else {
            500
        }
    }

    fn calculate_price(&self, request: &QuoteRequest) -> f64 {
        let distance = self.calculate_distance(&request.origin, &request.destination);
        let weight = request.parcel.billable_weight_kg();

        let base_price = self.config.base_rate_per_kg * weight;
        let distance_price = self.config.distance_rate_per_km * distance as f64;

        base_price + distance_price
    }

    fn estimate_delivery_days(&self, request: &QuoteRequest) -> u32 {
        let distance = self.calculate_distance(&request.origin, &request.destination);
        match distance {
            0..=50 => 1,
            51..=200 => 2,
            201..=500 => 3,
            _ => 5,
        }
    }

    fn should_fail(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.config.failure_rate
    }

    fn should_timeout(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.config.timeout_rate
    }

    fn generate_tracking_number(&self) -> String {
        let mut rng = rand::thread_rng();
        let number: u64 = rng.gen_range(100000000..999999999);
        format!("{}-{}", self.name.to_uppercase(), number)
    }

    fn simulate_tracking_status(&self, _tracking_number: &str) -> ShipmentStatus {
        // Simulate random tracking progression
        let mut rng = rand::thread_rng();
        let random: f64 = rng.gen();

        if random < 0.1 {
            ShipmentStatus::Created
        } else if random < 0.3 {
            ShipmentStatus::PickedUp
        } else if random < 0.7 {
            ShipmentStatus::InTransit
        } else if random < 0.9 {
            ShipmentStatus::OutForDelivery
        } else {
            ShipmentStatus::Delivered
        }
    }
}

#[async_trait]
impl CarrierAdapter for MockCarrierAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    async fn quote(&self, request: &QuoteRequest) -> Result<Quote, CarrierError> {
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(self.config.response_delay_ms)).await;

        // Simulate timeout
        if self.should_timeout() {
            return Err(CarrierError::Timeout);
        }

        // Simulate failure
        if self.should_fail() {
            return Err(CarrierError::ExternalError(format!(
                "Simulated {} error",
                self.name
            )));
        }

        let base_price = self.calculate_price(request);
        let estimated_days = self.estimate_delivery_days(request);

        Ok(Quote::new(
            request,
            self.name(),
            crate::domain::Money::from_f64(base_price, "ARS"),
            estimated_days,
        ))
    }

    async fn create_shipment(&self, quote: &Quote) -> Result<Shipment, CarrierError> {
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(self.config.response_delay_ms)).await;

        // Simulate timeout
        if self.should_timeout() {
            return Err(CarrierError::Timeout);
        }

        // Simulate failure
        if self.should_fail() {
            return Err(CarrierError::ExternalError(format!(
                "Simulated {} error",
                self.name
            )));
        }

        let tracking_number = self.generate_tracking_number();
        let shipment = Shipment::new(quote, tracking_number);

        Ok(shipment)
    }

    async fn tracking(&self, tracking_number: &str) -> Result<Tracking, CarrierError> {
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(self.config.response_delay_ms)).await;

        // Simulate timeout
        if self.should_timeout() {
            return Err(CarrierError::Timeout);
        }

        // Simulate failure
        if self.should_fail() {
            return Err(CarrierError::ExternalError(format!(
                "Simulated {} error",
                self.name
            )));
        }

        let status = self.simulate_tracking_status(tracking_number);
        let description = match status {
            ShipmentStatus::Created => "Shipment created",
            ShipmentStatus::PickedUp => "Package picked up",
            ShipmentStatus::InTransit => "Package in transit",
            ShipmentStatus::OutForDelivery => "Out for delivery",
            ShipmentStatus::Delivered => "Delivered",
            ShipmentStatus::Failed => "Delivery failed",
            ShipmentStatus::Cancelled => "Shipment cancelled",
        };

        let mut tracking = Tracking::new(Uuid::new_v4(), status.clone(), description);

        // Add some history
        if status != ShipmentStatus::Created {
            tracking.add_event(
                ShipmentStatus::Created,
                "Shipment created",
                Some("Origin".to_string()),
            );
        }
        if status == ShipmentStatus::InTransit
            || status == ShipmentStatus::OutForDelivery
            || status == ShipmentStatus::Delivered
        {
            tracking.add_event(
                ShipmentStatus::PickedUp,
                "Package picked up",
                Some("Origin Warehouse".to_string()),
            );
        }
        if status == ShipmentStatus::OutForDelivery || status == ShipmentStatus::Delivered {
            tracking.add_event(
                ShipmentStatus::InTransit,
                "Package in transit",
                Some("Distribution Center".to_string()),
            );
        }

        Ok(tracking)
    }

    fn is_healthy(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Address, Dimensions, Parcel, QuoteRequest};

    fn create_test_request() -> QuoteRequest {
        let origin = Address::new("Street 1", "Buenos Aires", "CABA", "C1000", "AR");
        let destination = Address::new("Street 2", "Córdoba", "Córdoba", "X5000", "AR");
        let parcel = Parcel::new(2.5, Dimensions::new(30.0, 20.0, 15.0).unwrap()).unwrap();
        QuoteRequest::new(origin, destination, parcel)
    }

    #[tokio::test]
    async fn test_mock_carrier_quote() {
        let config = MockCarrierConfig {
            response_delay_ms: 10,
            failure_rate: 0.0,
            timeout_rate: 0.0,
            ..Default::default()
        };
        let carrier = MockCarrierAdapter::new("test", config);
        let request = create_test_request();

        let quote = carrier.quote(&request).await.unwrap();
        assert_eq!(quote.carrier, "test");
        assert!(quote.base_price.amount > rust_decimal::Decimal::from(0));
    }

    #[tokio::test]
    async fn test_mock_carrier_create_shipment() {
        let config = MockCarrierConfig {
            response_delay_ms: 10,
            failure_rate: 0.0,
            timeout_rate: 0.0,
            ..Default::default()
        };
        let carrier = MockCarrierAdapter::new("test", config);
        let request = create_test_request();
        let quote = carrier.quote(&request).await.unwrap();

        let shipment = carrier.create_shipment(&quote).await.unwrap();
        assert_eq!(shipment.carrier, "test");
        assert!(!shipment.tracking_number.is_empty());
    }

    #[tokio::test]
    async fn test_mock_carrier_tracking() {
        let config = MockCarrierConfig {
            response_delay_ms: 10,
            failure_rate: 0.0,
            timeout_rate: 0.0,
            ..Default::default()
        };
        let carrier = MockCarrierAdapter::new("test", config);

        let tracking = carrier.tracking("TEST-123456").await.unwrap();
        assert!(!tracking.history.is_empty());
    }
}
