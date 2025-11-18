#[cfg(test)]
mod tests {
    use crate::ecosystem_service::EcosystemGraph;
    use crate::models::{
        ComponentStatus, EcosystemComponent, EcosystemComponentType, EcosystemLink,
    };
    use serde_json::json;

    fn comp(
        id: &str,
        name: &str,
        t: EcosystemComponentType,
        status: ComponentStatus,
    ) -> EcosystemComponent {
        EcosystemComponent {
            id: id.to_string(),
            name: name.to_string(),
            component_type: t,
            version: "1.0.0".to_string(),
            status,
            metadata: json!({}),
        }
    }

    #[test]
    fn add_components_and_links() {
        let mut g = EcosystemGraph::new();
        g.add_component(comp(
            "api",
            "API",
            EcosystemComponentType::API,
            ComponentStatus::Healthy,
        ));
        g.add_component(comp(
            "core",
            "Core",
            EcosystemComponentType::Service,
            ComponentStatus::Healthy,
        ));
        g.add_component(comp(
            "web",
            "Web",
            EcosystemComponentType::UI,
            ComponentStatus::Degraded,
        ));

        assert!(g
            .add_link(EcosystemLink {
                from_id: "core".into(),
                to_id: "api".into(),
                relation: "calls".into()
            })
            .is_ok());
        assert!(g
            .add_link(EcosystemLink {
                from_id: "api".into(),
                to_id: "web".into(),
                relation: "serves".into()
            })
            .is_ok());

        let deps = g.dependencies_for("web");
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].id, "api");

        let hs = g.health_summary();
        assert_eq!(hs.healthy, 2);
        assert_eq!(hs.degraded, 1);
        assert_eq!(hs.down, 0);
        assert_eq!(hs.total_components, 3);
        assert_eq!(hs.total_links, 2);
    }
}
