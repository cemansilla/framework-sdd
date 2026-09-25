mod application;
mod domain;
mod infrastructure;

use std::sync::Arc;

use application::{MarginEngine, QuoteService, ShipmentService};
use infrastructure::{
    create_all_carriers, InMemoryQuoteRepository, InMemoryShipmentRepository,
    InMemoryTrackingRepository,
};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Logistics Gateway PoC");

    // Create carriers
    let carriers = create_all_carriers();
    tracing::info!("Initialized {} carriers", carriers.len());

    // Create repositories
    let quote_repo = Arc::new(InMemoryQuoteRepository::new());
    let shipment_repo = Arc::new(InMemoryShipmentRepository::new());
    let tracking_repo = Arc::new(InMemoryTrackingRepository::new());

    // Create margin engine with default 10% margin
    let margin_engine = Arc::new(MarginEngine::with_default_margin(10.0));

    // Create services
    let quote_service = Arc::new(QuoteService::new(
        carriers.clone(),
        margin_engine.clone(),
        quote_repo.clone(),
    ));
    let shipment_service = Arc::new(ShipmentService::new(
        carriers.clone(),
        shipment_repo.clone(),
        tracking_repo.clone(),
    ));

    tracing::info!("Services initialized");

    // Demo: Request a quote
    tracing::info!("=== Demo: Quote Request ===");
    let request = domain::QuoteRequest::new(
        domain::Address::new("Av. Corrientes 1234", "Buenos Aires", "CABA", "C1043", "AR"),
        domain::Address::new("Calle 50 678", "La Plata", "Buenos Aires", "B1900", "AR"),
        domain::Parcel::new(2.5, domain::Dimensions::new(30.0, 20.0, 15.0).unwrap()).unwrap(),
    );

    match quote_service.request_quote(request).await {
        Ok(quotes) => {
            tracing::info!("Received {} quotes", quotes.len());
            for quote in &quotes {
                tracing::info!(
                    "Carrier: {}, Base Price: {}, Final Price: {}, Days: {}",
                    quote.carrier,
                    quote.base_price.amount,
                    quote.final_price.amount,
                    quote.estimated_days
                );
            }

            // Demo: Create shipment from first quote
            if let Some(quote) = quotes.first() {
                tracing::info!("=== Demo: Create Shipment ===");
                match shipment_service.create_shipment(quote).await {
                    Ok(shipment) => {
                        tracing::info!(
                            "Shipment created: ID={}, Tracking={}",
                            shipment.id,
                            shipment.tracking_number
                        );

                        // Demo: Get tracking
                        tracing::info!("=== Demo: Get Tracking ===");
                        match shipment_service.get_tracking(shipment.id).await {
                            Ok(tracking) => {
                                tracing::info!("Status: {}", tracking.status);
                                tracing::info!("Description: {}", tracking.status_description);
                                if let Some(location) = &tracking.location {
                                    tracing::info!("Location: {}", location);
                                }
                                tracing::info!("History events: {}", tracking.history.len());
                            }
                            Err(e) => tracing::error!("Failed to get tracking: {}", e),
                        }
                    }
                    Err(e) => tracing::error!("Failed to create shipment: {}", e),
                }
            }
        }
        Err(e) => tracing::error!("Failed to request quotes: {}", e),
    }

    tracing::info!("Logistics Gateway PoC completed successfully");
}
