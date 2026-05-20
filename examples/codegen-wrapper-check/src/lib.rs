#![allow(dead_code)]

mod generated {
    include!("../../codegen/generated/gemstone_wrappers.rs");
}

mod connector_generated {
    include!("../../codegen/generated/connector_mapping_wrappers.rs");
}

use std::collections::BTreeMap;

pub fn sample_booking() -> generated::BookingDraft {
    generated::BookingDraft {
        name: "example".to_string(),
        amount: 100,
        currency: "GBP".to_string(),
        tags: vec!["codegen".to_string(), "compile-smoke".to_string()],
        labels: BTreeMap::from([("source".to_string(), "compile-smoke".to_string())]),
        note: None,
    }
}

pub fn sample_connector_booking() -> connector_generated::Booking {
    connector_generated::Booking {
        status: "confirmed".to_string(),
        customer: connector_generated::Customer {
            name: "example".to_string(),
            vip: true,
        },
        amount: 100,
        labels: BTreeMap::from([("source".to_string(), "connector-compile-smoke".to_string())]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gemstone_rs::BridgeMapped;

    #[test]
    fn generated_booking_is_bridge_mapped() {
        let value = sample_booking().to_bridge_value();
        assert!(matches!(value, gemstone_rs::BridgeValue::KeyedDictionary(_)));
    }

    #[test]
    fn connector_generated_booking_is_bridge_mapped() {
        let value = sample_connector_booking().to_bridge_value();
        assert!(matches!(value, gemstone_rs::BridgeValue::KeyedDictionary(_)));
    }
}
