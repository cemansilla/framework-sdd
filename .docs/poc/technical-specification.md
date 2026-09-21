# PoC — Technical Specification

## 1. Stack

La PoC debe utilizar Rust para mantener coherencia con el framework, salvo que durante la fase de diseño se justifique otra elección.

Componentes iniciales:

- HTTP API;
- persistence layer;
- domain layer;
- carrier adapters;
- mock carrier implementations;
- test suite.

La tecnología concreta de HTTP y persistencia deberá quedar registrada como decisión técnica durante la PoC.

## 2. Domain

Entidades mínimas:

- Address;
- Parcel;
- QuoteRequest;
- Quote;
- Carrier;
- Shipment;
- Tracking;
- MarginRule.

## 3. Ports

El dominio debe definir contratos para:

- CarrierAdapter;
- QuoteRepository;
- ShipmentRepository;
- TrackingRepository;
- MarginPolicy.

## 4. API

Endpoints iniciales sugeridos:

```text
POST /quotes
GET /carriers
POST /shipments
GET /shipments/{id}
GET /shipments/{id}/tracking
```

Los contratos definitivos deben surgir de los requisitos y diseño producidos durante la PoC.

## 5. Mock carriers

Cada mock debe poder simular:

- éxito;
- tarifa no disponible;
- timeout;
- error externo;
- tracking;
- creación de shipment.

## 6. Persistence

Persistir al menos:

- quotes;
- shipments;
- carrier configuration;
- margin rules.

## 7. Testing

La PoC debe verificar:

- domain rules;
- adapter contracts;
- API contracts;
- persistence;
- error paths;
- margin calculation;
- full quote flow;
- full shipment flow.

## 8. Observability

Registrar como mínimo:

- request id;
- carrier;
- operation;
- success/failure;
- latency;
- normalized error.

## 9. Architecture constraint

El dominio no debe depender de HTTP, base de datos ni implementaciones concretas de carriers.
