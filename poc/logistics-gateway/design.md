# Logistics Gateway — Technical Design

## Overview

This document provides detailed technical specifications for the Logistics Gateway PoC, building upon the architecture defined in architecture.md.

## API Design

### Endpoint Specifications

#### POST /quotes

**Request:**
```json
{
  "origin": {
    "street": "Av. Corrientes 1234",
    "city": "Buenos Aires",
    "state": "CABA",
    "postal_code": "C1043",
    "country": "AR"
  },
  "destination": {
    "street": "Calle 50 678",
    "city": "La Plata",
    "state": "Buenos Aires",
    "postal_code": "B1900",
    "country": "AR"
  },
  "parcel": {
    "weight_kg": 2.5,
    "length_cm": 30.0,
    "width_cm": 20.0,
    "height_cm": 15.0
  }
}
```

**Response (200 OK):**
```json
{
  "quotes": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "carrier": "oca",
      "base_price": 1500.00,
      "final_price": 1650.00,
      "currency": "ARS",
      "estimated_days": 3,
      "valid_until": "2026-03-20T23:59:59Z",
      "margin_breakdown": {
        "base_price": 1500.00,
        "margin_percentage": 10.0,
        "margin_amount": 150.00
      }
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "carrier": "andreani",
      "base_price": 1400.00,
      "final_price": 1540.00,
      "currency": "ARS",
      "estimated_days": 2,
      "valid_until": "2026-03-20T23:59:59Z",
      "margin_breakdown": {
        "base_price": 1400.00,
        "margin_percentage": 10.0,
        "margin_amount": 140.00
      }
    }
  ],
  "request_id": "req-550e8400-e29b-41d4-a716-446655440002"
}
```

**Error Responses:**
- 400 Bad Request: Invalid input
- 500 Internal Server Error: Unexpected error
- 503 Service Unavailable: All carriers unavailable

---

#### GET /carriers

**Response (200 OK):**
```json
{
  "carriers": [
    {
      "id": "oca",
      "name": "OCA",
      "enabled": true,
      "capabilities": ["quote", "shipment", "tracking"]
    },
    {
      "id": "andreani",
      "name": "Andreani",
      "enabled": true,
      "capabilities": ["quote", "shipment", "tracking"]
    },
    {
      "id": "express",
      "name": "Express",
      "enabled": true,
      "capabilities": ["quote", "shipment", "tracking"]
    }
  ]
}
```

---

#### POST /shipments

**Request:**
```json
{
  "quote_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response (201 Created):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440003",
  "quote_id": "550e8400-e29b-41d4-a716-446655440000",
  "carrier": "oca",
  "tracking_number": "OCA-123456789",
  "status": "created",
  "created_at": "2026-03-19T10:00:00Z",
  "estimated_delivery": "2026-03-22T23:59:59Z"
}
```

**Error Responses:**
- 400 Bad Request: Invalid quote ID
- 404 Not Found: Quote not found
- 410 Gone: Quote expired
- 500 Internal Server Error: Unexpected error

---

#### GET /shipments/{id}

**Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440003",
  "quote_id": "550e8400-e29b-41d4-a716-446655440000",
  "carrier": "oca",
  "tracking_number": "OCA-123456789",
  "status": "in_transit",
  "created_at": "2026-03-19T10:00:00Z",
  "estimated_delivery": "2026-03-22T23:59:59Z"
}
```

**Error Responses:**
- 404 Not Found: Shipment not found

---

#### GET /shipments/{id}/tracking

**Response (200 OK):**
```json
{
  "shipment_id": "550e8400-e29b-41d4-a716-446655440003",
  "status": "in_transit",
  "status_description": "Package is in transit",
  "location": "Distribution Center - Buenos Aires",
  "updated_at": "2026-03-20T14:30:00Z",
  "history": [
    {
      "status": "created",
      "description": "Shipment created",
      "location": "Origin",
      "timestamp": "2026-03-19T10:00:00Z"
    },
    {
      "status": "picked_up",
      "description": "Package picked up",
      "location": "Origin Warehouse",
      "timestamp": "2026-03-19T15:00:00Z"
    },
    {
      "status": "in_transit",
      "description": "Package is in transit",
      "location": "Distribution Center - Buenos Aires",
      "timestamp": "2026-03-20T14:30:00Z"
    }
  ]
}
```

**Error Responses:**
- 404 Not Found: Shipment not found
- 503 Service Unavailable: Carrier tracking unavailable

## Domain Model Details

### Value Objects

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimensions {
    pub length_cm: f64,
    pub width_cm: f64,
    pub height_cm: f64,
}

impl Dimensions {
    pub fn volumetric_weight_kg(&self) -> f64 {
        // Standard volumetric weight formula
        (self.length_cm * self.width_cm * self.height_cm) / 5000.0
    }
    
    pub fn billable_weight_kg(&self, actual_weight_kg: f64) -> f64 {
        actual_weight_kg.max(self.volumetric_weight_kg())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
}

impl Address {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.street.is_empty() {
            return Err(ValidationError::EmptyField("street".to_string()));
        }
        if self.city.is_empty() {
            return Err(ValidationError::EmptyField("city".to_string()));
        }
        if self.postal_code.is_empty() {
            return Err(ValidationError::EmptyField("postal_code".to_string()));
        }
        Ok(())
    }
}
```

### Entities

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRequest {
    pub id: Uuid,
    pub origin: Address,
    pub destination: Address,
    pub parcel: Parcel,
    pub requested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parcel {
    pub weight_kg: f64,
    pub dimensions: Dimensions,
}

impl Parcel {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.weight_kg <= 0.0 {
            return Err(ValidationError::InvalidValue("weight must be positive".to_string()));
        }
        if self.weight_kg > 50.0 {
            return Err(ValidationError::InvalidValue("weight exceeds maximum".to_string()));
        }
        self.dimensions.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: Uuid,
    pub request_id: Uuid,
    pub carrier: String,
    pub base_price: Money,
    pub final_price: Money,
    pub estimated_days: u32,
    pub valid_until: DateTime<Utc>,
    pub margin_applied: Option<MarginBreakdown>,
    pub created_at: DateTime<Utc>,
}

impl Quote {
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.valid_until
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracking {
    pub shipment_id: Uuid,
    pub status: ShipmentStatus,
    pub status_description: String,
    pub location: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub history: Vec<TrackingEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEvent {
    pub status: ShipmentStatus,
    pub description: String,
    pub location: Option<String>,
    pub timestamp: DateTime<Utc>,
}
```

### Margin Rules

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginRule {
    pub id: Uuid,
    pub name: String,
    pub condition: MarginCondition,
    pub adjustment: MarginAdjustment,
    pub priority: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarginCondition {
    Always,
    WeightRange { min_kg: f64, max_kg: f64 },
    PriceRange { min_amount: Decimal, max_amount: Decimal },
    Zone { zones: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarginAdjustment {
    Percentage(f64),
    FixedAmount(Money),
    Formula(String), // Future: expression evaluation
}

pub struct MarginEngine {
    rules: Vec<MarginRule>,
}

impl MarginEngine {
    pub fn apply(&self, base_price: Money, context: &MarginContext) -> (Money, MarginBreakdown) {
        let mut current_price = base_price.clone();
        let mut breakdown = MarginBreakdown::new(base_price.clone());
        
        let mut applicable_rules: Vec<&MarginRule> = self.rules
            .iter()
            .filter(|rule| rule.enabled && rule.condition.matches(context))
            .collect();
        
        applicable_rules.sort_by_key(|rule| rule.priority);
        
        for rule in applicable_rules {
            let adjustment = rule.adjustment.calculate(&current_price, context);
            current_price = current_price.add(&adjustment);
            breakdown.add_adjustment(rule.name.clone(), adjustment);
        }
        
        (current_price, breakdown)
    }
}
```

## Carrier Adapter Implementation

### CarrierAdapter Trait

```rust
#[async_trait]
pub trait CarrierAdapter: Send + Sync {
    fn name(&self) -> &str;
    
    async fn quote(&self, request: &QuoteRequest) -> Result<Quote, CarrierError>;
    
    async fn create_shipment(&self, quote: &Quote) -> Result<Shipment, CarrierError>;
    
    async fn tracking(&self, tracking_number: &str) -> Result<Tracking, CarrierError>;
    
    fn is_healthy(&self) -> bool;
}
```

### Mock Carrier Implementation

```rust
pub struct OcaMockAdapter {
    config: MockCarrierConfig,
}

impl OcaMockAdapter {
    pub fn new(config: MockCarrierConfig) -> Self {
        Self { config }
    }
    
    fn calculate_price(&self, request: &QuoteRequest) -> Decimal {
        // Base price calculation
        let distance = self.calculate_distance(&request.origin, &request.destination);
        let weight = request.parcel.dimensions.billable_weight_kg(request.parcel.weight_kg);
        
        let base_rate = Decimal::from_f64(100.0).unwrap(); // $100 per kg
        let distance_rate = Decimal::from_f64(distance as f64 * 0.5).unwrap(); // $0.5 per km
        
        (base_rate * Decimal::from_f64(weight).unwrap()) + distance_rate
    }
    
    fn calculate_distance(&self, origin: &Address, destination: &Address) -> u32 {
        // Simplified distance calculation
        // In real implementation, use geocoding API
        if origin.city == destination.city {
            10 // Same city: 10km
        } else if origin.state == destination.state {
            100 // Same state: 100km
        } else {
            500 // Different state: 500km
        }
    }
}

#[async_trait]
impl CarrierAdapter for OcaMockAdapter {
    fn name(&self) -> &str {
        "oca"
    }
    
    async fn quote(&self, request: &QuoteRequest) -> Result<Quote, CarrierError> {
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(self.config.response_delay_ms)).await;
        
        // Simulate failures
        if self.config.should_fail() {
            return Err(CarrierError::ExternalError("Simulated OCA error".to_string()));
        }
        
        let base_price = self.calculate_price(request);
        let estimated_days = self.estimate_delivery_days(request);
        
        Ok(Quote {
            id: Uuid::new_v4(),
            request_id: request.id,
            carrier: self.name().to_string(),
            base_price: Money::new(base_price, "ARS"),
            final_price: Money::new(base_price, "ARS"), // Margin applied later
            estimated_days,
            valid_until: Utc::now() + Duration::hours(24),
            margin_applied: None,
            created_at: Utc::now(),
        })
    }
    
    async fn create_shipment(&self, quote: &Quote) -> Result<Shipment, CarrierError> {
        tokio::time::sleep(Duration::from_millis(self.config.response_delay_ms)).await;
        
        if self.config.should_fail() {
            return Err(CarrierError::ExternalError("Simulated OCA error".to_string()));
        }
        
        let tracking_number = format!("OCA-{}", rand::random::<u64>());
        
        Ok(Shipment {
            id: Uuid::new_v4(),
            quote_id: quote.id,
            carrier: self.name().to_string(),
            tracking_number,
            status: ShipmentStatus::Created,
            origin: quote.origin.clone(),
            destination: quote.destination.clone(),
            parcel: quote.parcel.clone(),
            created_at: Utc::now(),
            estimated_delivery: Utc::now() + Duration::days(quote.estimated_days as i64),
        })
    }
    
    async fn tracking(&self, tracking_number: &str) -> Result<Tracking, CarrierError> {
        tokio::time::sleep(Duration::from_millis(self.config.response_delay_ms)).await;
        
        if self.config.should_fail() {
            return Err(CarrierError::ExternalError("Simulated OCA error".to_string()));
        }
        
        // Simulate tracking progression
        let status = self.simulate_tracking_status(tracking_number);
        
        Ok(Tracking {
            shipment_id: Uuid::new_v4(), // Would be looked up in real implementation
            status: status.clone(),
            status_description: self.status_description(&status),
            location: Some("Distribution Center".to_string()),
            updated_at: Utc::now(),
            history: vec![],
        })
    }
    
    fn is_healthy(&self) -> bool {
        true
    }
}
```

## Database Schema

### SQLite Schema (PoC)

```sql
-- Quotes table
CREATE TABLE quotes (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    carrier TEXT NOT NULL,
    base_price_amount TEXT NOT NULL,
    base_price_currency TEXT NOT NULL,
    final_price_amount TEXT NOT NULL,
    final_price_currency TEXT NOT NULL,
    estimated_days INTEGER NOT NULL,
    valid_until TEXT NOT NULL,
    margin_breakdown TEXT, -- JSON
    created_at TEXT NOT NULL
);

-- Shipments table
CREATE TABLE shipments (
    id TEXT PRIMARY KEY,
    quote_id TEXT NOT NULL,
    carrier TEXT NOT NULL,
    tracking_number TEXT NOT NULL,
    status TEXT NOT NULL,
    origin TEXT NOT NULL, -- JSON
    destination TEXT NOT NULL, -- JSON
    parcel TEXT NOT NULL, -- JSON
    created_at TEXT NOT NULL,
    estimated_delivery TEXT NOT NULL,
    FOREIGN KEY (quote_id) REFERENCES quotes(id)
);

-- Margin rules table
CREATE TABLE margin_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    condition_type TEXT NOT NULL,
    condition_data TEXT NOT NULL, -- JSON
    adjustment_type TEXT NOT NULL,
    adjustment_data TEXT NOT NULL, -- JSON
    priority INTEGER NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1
);

-- Carriers configuration table
CREATE TABLE carriers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    config TEXT NOT NULL -- JSON
);

-- Indexes
CREATE INDEX idx_quotes_request_id ON quotes(request_id);
CREATE INDEX idx_quotes_carrier ON quotes(carrier);
CREATE INDEX idx_shipments_quote_id ON shipments(quote_id);
CREATE INDEX idx_shipments_status ON shipments(status);
CREATE INDEX idx_shipments_tracking_number ON shipments(tracking_number);
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Quote expired: {0}")]
    QuoteExpired(Uuid),
    
    #[error("Carrier error: {0}")]
    CarrierError(#[from] CarrierError),
    
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum CarrierError {
    #[error("Carrier unavailable: {0}")]
    Unavailable(String),
    
    #[error("Quote not available: {0}")]
    QuoteNotAvailable(String),
    
    #[error("External error: {0}")]
    ExternalError(String),
    
    #[error("Timeout")]
    Timeout,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Empty field: {0}")]
    EmptyField(String),
    
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}
```

### Error Response Format

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "details": [
      {
        "field": "parcel.weight_kg",
        "message": "Weight must be positive"
      }
    ]
  },
  "request_id": "req-550e8400-e29b-41d4-a716-446655440000"
}
```

## Configuration

### Configuration Structure

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub carriers: CarriersConfig,
    pub margin: MarginConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CarriersConfig {
    pub oca: MockCarrierConfig,
    pub andreani: MockCarrierConfig,
    pub express: MockCarrierConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MockCarrierConfig {
    pub enabled: bool,
    pub response_delay_ms: u64,
    pub failure_rate: f64,
    pub timeout_rate: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarginConfig {
    pub default_margin_percentage: f64,
    pub rules: Vec<MarginRuleConfig>,
}
```

### Configuration File (config.toml)

```toml
[server]
host = "127.0.0.1"
port = 8080
request_timeout_ms = 30000

[database]
url = "sqlite://logistics_gateway.db"
pool_size = 10

[carriers.oca]
enabled = true
response_delay_ms = 100
failure_rate = 0.05
timeout_rate = 0.02

[carriers.andreani]
enabled = true
response_delay_ms = 150
failure_rate = 0.03
timeout_rate = 0.01

[carriers.express]
enabled = true
response_delay_ms = 80
failure_rate = 0.02
timeout_rate = 0.01

[margin]
default_margin_percentage = 10.0

[[margin.rules]]
name = "Heavy package surcharge"
condition_type = "WeightRange"
condition_data = { min_kg = 10.0, max_kg = 50.0 }
adjustment_type = "Percentage"
adjustment_data = { percentage = 15.0 }
priority = 1
enabled = true

[[margin.rules]]
name = "Long distance surcharge"
condition_type = "Zone"
condition_data = { zones = ["long_distance"] }
adjustment_type = "FixedAmount"
adjustment_data = { amount = 500.0, currency = "ARS" }
priority = 2
enabled = true
```

## Testing Strategy

### Unit Tests

- Domain model validation
- Margin calculation logic
- Error handling
- Value object behavior

### Integration Tests

- Carrier adapter contracts
- Repository operations
- Service layer interactions
- Database operations

### API Tests

- Endpoint request/response validation
- Error response format
- Input validation
- Authentication (future)

### End-to-End Tests

- Complete quote flow
- Complete shipment flow
- Tracking flow
- Error scenarios

### Performance Tests

- Quote request latency
- Shipment creation latency
- Concurrent request handling
- Database query performance

## Deployment

### Docker Compose (Development)

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite://data/logistics_gateway.db
      - RUST_LOG=info
    volumes:
      - ./data:/app/data
      - ./config:/app/config
```

### Dockerfile

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/logistics-gateway /app/
COPY --from=builder /app/config /app/config

EXPOSE 8080

CMD ["/app/logistics-gateway"]
```

## Monitoring

### Health Check Endpoint

```
GET /health
```

**Response (200 OK):**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "timestamp": "2026-03-19T10:00:00Z",
  "checks": {
    "database": "healthy",
    "carriers": {
      "oca": "healthy",
      "andreani": "healthy",
      "express": "healthy"
    }
  }
}
```

### Metrics

- Request count by endpoint
- Request latency by endpoint
- Error count by type
- Carrier success/failure rates
- Database query latency
- Active connections

## Implementation Plan

### Phase 1: Foundation (Week 1)
- Project setup
- Domain models
- Database schema
- Basic configuration

### Phase 2: Core Services (Week 2)
- Carrier adapters
- Quote service
- Margin engine
- Repository implementations

### Phase 3: API Layer (Week 3)
- REST API endpoints
- Input validation
- Error handling
- API tests

### Phase 4: Shipment Flow (Week 4)
- Shipment service
- Tracking service
- Integration tests
- E2E tests

### Phase 5: Quality & Polish (Week 5)
- Performance optimization
- Documentation
- Observability
- Final testing

## Conclusion

This technical design provides a comprehensive blueprint for implementing the Logistics Gateway PoC. The hexagonal architecture ensures flexibility and testability, while the detailed specifications enable consistent implementation across the team.
