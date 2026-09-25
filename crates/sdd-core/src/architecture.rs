use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Architecture {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub style: ArchitectureStyle,
    pub components: Vec<Component>,
    pub interfaces: Vec<Interface>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArchitectureStyle {
    Hexagonal,
    Layered,
    Microservices,
    EventDriven,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub description: String,
    pub responsibility: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    pub id: String,
    pub name: String,
    pub description: String,
    pub port_type: PortType,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PortType {
    Driving,
    Driven,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub input: String,
    pub output: String,
    pub errors: Vec<String>,
}

impl Architecture {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        style: ArchitectureStyle,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            style,
            components: Vec::new(),
            interfaces: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_component(mut self, component: Component) -> Self {
        self.components.push(component);
        self.updated_at = Utc::now();
        self
    }

    pub fn add_interface(mut self, interface: Interface) -> Self {
        self.interfaces.push(interface);
        self.updated_at = Utc::now();
        self
    }
}

impl Component {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        responsibility: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            responsibility: responsibility.into(),
            dependencies: Vec::new(),
        }
    }

    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }
}

impl Interface {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        port_type: PortType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            port_type,
            operations: Vec::new(),
        }
    }

    pub fn add_operation(mut self, operation: Operation) -> Self {
        self.operations.push(operation);
        self
    }
}

impl Operation {
    pub fn new(
        name: impl Into<String>,
        input: impl Into<String>,
        output: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            input: input.into(),
            output: output.into(),
            errors: Vec::new(),
        }
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adr {
    pub id: Uuid,
    pub adr_id: String,
    pub title: String,
    pub status: AdrStatus,
    pub context: String,
    pub decision: String,
    pub consequences: Vec<String>,
    pub related_adrs: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded { by: String },
}

impl Adr {
    pub fn new(
        adr_id: impl Into<String>,
        title: impl Into<String>,
        context: impl Into<String>,
        decision: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            adr_id: adr_id.into(),
            title: title.into(),
            status: AdrStatus::Proposed,
            context: context.into(),
            decision: decision.into(),
            consequences: Vec::new(),
            related_adrs: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_consequence(mut self, consequence: impl Into<String>) -> Self {
        self.consequences.push(consequence.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn add_related(mut self, adr_id: impl Into<String>) -> Self {
        self.related_adrs.push(adr_id.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn accept(mut self) -> Self {
        self.status = AdrStatus::Accepted;
        self.updated_at = Utc::now();
        self
    }

    pub fn deprecate(mut self) -> Self {
        self.status = AdrStatus::Deprecated;
        self.updated_at = Utc::now();
        self
    }

    pub fn supersede(mut self, by: impl Into<String>) -> Self {
        self.status = AdrStatus::Superseded { by: by.into() };
        self.updated_at = Utc::now();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_creation() {
        let arch = Architecture::new(
            "SDD Architecture",
            "Hexagonal architecture for SDD framework",
            ArchitectureStyle::Hexagonal,
        );
        assert_eq!(arch.style, ArchitectureStyle::Hexagonal);
        assert!(arch.components.is_empty());
    }

    #[test]
    fn test_add_component() {
        let component = Component::new("core", "Core", "Domain logic", "Business rules");
        let arch = Architecture::new("SDD", "Description", ArchitectureStyle::Hexagonal)
            .add_component(component);
        assert_eq!(arch.components.len(), 1);
    }

    #[test]
    fn test_adr_creation() {
        let adr = Adr::new(
            "ADR-001",
            "Use Hexagonal",
            "Need clean architecture",
            "Adopt hexagonal architecture",
        );
        assert_eq!(adr.status, AdrStatus::Proposed);
    }

    #[test]
    fn test_adr_accept() {
        let adr = Adr::new(
            "ADR-001",
            "Use Hexagonal",
            "Need clean architecture",
            "Adopt hexagonal architecture",
        )
        .accept();
        assert_eq!(adr.status, AdrStatus::Accepted);
    }

    #[test]
    fn test_adr_supersede() {
        let adr = Adr::new(
            "ADR-001",
            "Use Hexagonal",
            "Need clean architecture",
            "Adopt hexagonal architecture",
        )
        .supersede("ADR-002");
        assert_eq!(
            adr.status,
            AdrStatus::Superseded {
                by: "ADR-002".to_string()
            }
        );
    }
}
