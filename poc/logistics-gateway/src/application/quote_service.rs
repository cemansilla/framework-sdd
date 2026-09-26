use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{
    CarrierAdapter, CarrierError, MarginContext, MarginPolicy, Quote, QuoteRepository,
    QuoteRequest, RepositoryError,
};

#[derive(Debug, thiserror::Error)]
pub enum QuoteServiceError {
    #[error("Carrier error: {0}")]
    CarrierError(#[from] CarrierError),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    #[error("No carriers available")]
    NoCarriersAvailable,
    #[allow(dead_code)]
    #[error("Quote not found: {0}")]
    NotFound(Uuid),
}

pub struct QuoteService {
    carriers: Vec<Arc<dyn CarrierAdapter>>,
    margin_policy: Arc<dyn MarginPolicy>,
    quote_repo: Arc<dyn QuoteRepository>,
}

impl QuoteService {
    pub fn new(
        carriers: Vec<Arc<dyn CarrierAdapter>>,
        margin_policy: Arc<dyn MarginPolicy>,
        quote_repo: Arc<dyn QuoteRepository>,
    ) -> Self {
        Self {
            carriers,
            margin_policy,
            quote_repo,
        }
    }

    pub async fn request_quote(
        &self,
        request: QuoteRequest,
    ) -> Result<Vec<Quote>, QuoteServiceError> {
        if self.carriers.is_empty() {
            return Err(QuoteServiceError::NoCarriersAvailable);
        }

        let mut quotes = Vec::new();
        let mut errors = Vec::new();

        // Query all carriers in parallel
        let carrier_futures = self
            .carriers
            .iter()
            .filter(|c| c.is_healthy())
            .map(|carrier| {
                let carrier = Arc::clone(carrier);
                let request = request.clone();
                async move {
                    match carrier.quote(&request).await {
                        Ok(quote) => Ok((carrier.name().to_string(), quote)),
                        Err(e) => Err((carrier.name().to_string(), e)),
                    }
                }
            });

        let results = futures::future::join_all(carrier_futures).await;

        for result in results {
            match result {
                Ok((_carrier_name, mut quote)) => {
                    // Apply margin
                    let context = MarginContext {
                        weight_kg: request.parcel.billable_weight_kg(),
                        origin_zone: request.origin.state.clone(),
                        destination_zone: request.destination.state.clone(),
                        base_price: quote.base_price.clone(),
                    };

                    let (final_price, breakdown) =
                        self.margin_policy.apply(quote.base_price.clone(), &context);
                    quote = quote.with_final_price(final_price, breakdown);

                    // Save quote
                    self.quote_repo.save(&quote).await?;
                    quotes.push(quote);
                }
                Err((carrier_name, error)) => {
                    tracing::warn!("Carrier {} failed: {}", carrier_name, error);
                    errors.push((carrier_name, error));
                }
            }
        }

        if quotes.is_empty() && !errors.is_empty() {
            return Err(QuoteServiceError::CarrierError(errors[0].1.clone()));
        }

        Ok(quotes)
    }

    #[allow(dead_code)]
    pub async fn get_quote(&self, id: Uuid) -> Result<Quote, QuoteServiceError> {
        self.quote_repo
            .find_by_id(id)
            .await?
            .ok_or(QuoteServiceError::NotFound(id))
    }

    #[allow(dead_code)]
    pub async fn get_quotes_by_request(
        &self,
        request_id: Uuid,
    ) -> Result<Vec<Quote>, QuoteServiceError> {
        Ok(self.quote_repo.find_by_request_id(request_id).await?)
    }
}
