# Logistics Gateway — Discovery

## Questions

### Q-001: HTTP Framework Selection
**Status:** Open  
**Priority:** High  
**Context:** Need to select HTTP framework for REST API

**Options:**
1. **Actix-web** - High performance, mature ecosystem
2. **Axum** - Modern, tokio-based, good ergonomics
3. **Warp** - Functional, composable filters
4. **Rocket** - Developer-friendly, macro-heavy

**Decision Pending:** Requires architecture phase

---

### Q-002: Persistence Layer
**Status:** Open  
**Priority:** High  
**Context:** Need to select persistence technology

**Options:**
1. **SQLite** - Simple, embedded, good for PoC
2. **PostgreSQL** - Production-ready, feature-rich
3. **In-memory** - Simplest, no persistence
4. **File-based JSON** - Simple, no database

**Decision Pending:** Requires architecture phase

---

### Q-003: Mock Carrier Behavior
**Status:** Open  
**Priority:** Medium  
**Context:** How realistic should mock carriers be?

**Options:**
1. **Static responses** - Fixed quotes and statuses
2. **Calculated responses** - Based on weight/distance
3. **Randomized responses** - Simulate real-world variability
4. **Configurable scenarios** - Success, failure, timeout

**Decision Pending:** Requires technical design phase

---

### Q-004: Margin Rule Complexity
**Status:** Open  
**Priority:** Medium  
**Context:** How complex should margin rules be?

**Options:**
1. **Fixed percentage** - Simple markup
2. **Tiered pricing** - Based on weight/zone
3. **Rule engine** - Complex conditional logic
4. **Formula-based** - Mathematical expressions

**Decision Pending:** Requires requirements phase

---

### Q-005: Error Handling Strategy
**Status:** Open  
**Priority:** High  
**Context:** How to handle carrier failures and errors?

**Options:**
1. **Fail-fast** - Return error immediately
2. **Retry with fallback** - Try alternative carriers
3. **Partial results** - Return available quotes
4. **Circuit breaker** - Temporarily disable failing carriers

**Decision Pending:** Requires architecture phase

## Decisions

### D-001: Use Hexagonal Architecture
**Status:** Accepted  
**Date:** 2026-03-19  
**Context:** Architecture pattern selection

**Decision:** Implement hexagonal (ports and adapters) architecture to separate domain logic from infrastructure concerns.

**Rationale:**
- Aligns with SDD framework principles
- Enables easy testing with mock adapters
- Supports future carrier integrations
- Clear separation of concerns

**Consequences:**
- More initial structure setup
- Better long-term maintainability
- Easier to add new carriers

---

### D-002: Rust Implementation
**Status:** Accepted  
**Date:** 2026-03-19  
**Context:** Technology stack selection

**Decision:** Implement PoC in Rust to maintain consistency with SDD framework.

**Rationale:**
- Type safety reduces runtime errors
- Performance suitable for microservice
- Framework already in Rust
- Team capability available

**Consequences:**
- Steeper learning curve for some team members
- Longer initial development time
- Better runtime reliability

## Assumptions

### A-001: Mock Carrier Realism
**Assumption:** Mock carriers will simulate realistic behavior including success, failure, and timeout scenarios.

**Impact:** If mocks are too simple, PoC won't validate error handling capabilities.

**Validation:** Include at least 3 error scenarios per carrier.

---

### A-002: Single Region
**Assumption:** PoC will handle domestic shipping only (single country).

**Impact:** Simplifies address validation and carrier rules.

**Validation:** Document international shipping as future enhancement.

---

### A-003: Sequential Processing
**Assumption:** Quote requests will be processed sequentially, not in parallel.

**Impact:** Simpler implementation, may be slower for many carriers.

**Validation:** Profile performance; consider parallelization if needed.

---

### A-004: Synchronous API
**Assumption:** API will be synchronous (request-response), not event-driven.

**Impact:** Simpler implementation, may not suit all use cases.

**Validation:** Document async as future enhancement.

## Risks

### R-001: Scope Creep
**Probability:** Medium  
**Impact:** High  
**Description:** PoC may expand beyond intended scope.

**Mitigation:**
- Strict adherence to defined scope
- Regular scope reviews
- Clear acceptance criteria

**Owner:** Product Owner

---

### R-002: Framework Limitations
**Probability:** Low  
**Impact:** Medium  
**Description:** SDD framework may have limitations affecting PoC.

**Mitigation:**
- Document workarounds
- Provide feedback for framework improvements
- Focus on framework validation goals

**Owner:** Development Team

---

### R-003: Mock Carrier Complexity
**Probability:** Medium  
**Impact:** Medium  
**Description:** Mock carriers may be too complex or too simple.

**Mitigation:**
- Define clear mock behavior specifications
- Include configurable scenarios
- Balance realism with simplicity

**Owner:** Development Team

---

### R-004: Time Constraints
**Probability:** Medium  
**Impact:** High  
**Description:** PoC may not complete within allocated time.

**Mitigation:**
- Prioritize core features
- Defer nice-to-have features
- Regular progress tracking

**Owner:** Project Manager

## Domain Model (Preliminary)

### Entities

**Address**
- street: String
- city: String
- state: String
- postal_code: String
- country: String

**Parcel**
- weight_kg: f64
- length_cm: f64
- width_cm: f64
- height_cm: f64

**QuoteRequest**
- origin: Address
- destination: Address
- parcel: Parcel
- requested_at: DateTime

**Quote**
- id: Uuid
- carrier: String
- base_price: Decimal
- final_price: Decimal
- estimated_days: u32
- valid_until: DateTime

**Carrier**
- id: String
- name: String
- enabled: bool
- config: CarrierConfig

**Shipment**
- id: Uuid
- quote_id: Uuid
- status: ShipmentStatus
- tracking_number: String
- created_at: DateTime

**Tracking**
- shipment_id: Uuid
- status: TrackingStatus
- location: Option<String>
- updated_at: DateTime

**MarginRule**
- id: Uuid
- name: String
- condition: MarginCondition
- adjustment: MarginAdjustment
- priority: u32

### Value Objects

**Money**
- amount: Decimal
- currency: String

**Dimensions**
- length: f64
- width: f64
- height: f64
- unit: String

**Weight**
- value: f64
- unit: String

## Next Steps

1. Finalize requirements based on discovery
2. Complete architecture design
3. Define technical specifications
4. Create implementation plan
5. Begin core domain implementation
