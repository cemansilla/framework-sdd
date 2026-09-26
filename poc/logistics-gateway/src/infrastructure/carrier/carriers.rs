use super::mock_carrier::{MockCarrierAdapter, MockCarrierConfig};
use crate::domain::CarrierAdapter;
use std::sync::Arc;

pub fn create_oca_adapter() -> Arc<dyn CarrierAdapter> {
    let config = MockCarrierConfig {
        response_delay_ms: 100,
        failure_rate: 0.05,
        timeout_rate: 0.02,
        base_rate_per_kg: 120.0, // OCA is slightly more expensive
        distance_rate_per_km: 0.6,
    };
    Arc::new(MockCarrierAdapter::new("oca", config))
}

pub fn create_andreani_adapter() -> Arc<dyn CarrierAdapter> {
    let config = MockCarrierConfig {
        response_delay_ms: 150,
        failure_rate: 0.03,
        timeout_rate: 0.01,
        base_rate_per_kg: 110.0, // Andreani is mid-range
        distance_rate_per_km: 0.55,
    };
    Arc::new(MockCarrierAdapter::new("andreani", config))
}

pub fn create_express_adapter() -> Arc<dyn CarrierAdapter> {
    let config = MockCarrierConfig {
        response_delay_ms: 80,
        failure_rate: 0.02,
        timeout_rate: 0.01,
        base_rate_per_kg: 100.0, // Express is cheapest
        distance_rate_per_km: 0.5,
    };
    Arc::new(MockCarrierAdapter::new("express", config))
}

pub fn create_all_carriers() -> Vec<Arc<dyn CarrierAdapter>> {
    vec![
        create_oca_adapter(),
        create_andreani_adapter(),
        create_express_adapter(),
    ]
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

    // Production configs include random failure/timeout simulation, so retry
    // a few times to keep tests deterministic in practice.
    async fn quote_with_retry(
        carrier: &Arc<dyn CarrierAdapter>,
        request: &QuoteRequest,
    ) -> crate::domain::Quote {
        for _ in 0..20 {
            if let Ok(quote) = carrier.quote(request).await {
                return quote;
            }
        }
        panic!("quote should succeed within retries");
    }

    #[tokio::test]
    async fn test_oca_adapter() {
        let carrier = create_oca_adapter();
        let request = create_test_request();
        let quote = quote_with_retry(&carrier, &request).await;
        assert_eq!(quote.carrier, "oca");
    }

    #[tokio::test]
    async fn test_andreani_adapter() {
        let carrier = create_andreani_adapter();
        let request = create_test_request();
        let quote = quote_with_retry(&carrier, &request).await;
        assert_eq!(quote.carrier, "andreani");
    }

    #[tokio::test]
    async fn test_express_adapter() {
        let carrier = create_express_adapter();
        let request = create_test_request();
        let quote = quote_with_retry(&carrier, &request).await;
        assert_eq!(quote.carrier, "express");
    }

    #[tokio::test]
    async fn test_all_carriers() {
        let carriers = create_all_carriers();
        assert_eq!(carriers.len(), 3);

        let request = create_test_request();
        for carrier in carriers {
            let quote = quote_with_retry(&carrier, &request).await;
            assert!(!quote.carrier.is_empty());
        }
    }
}
