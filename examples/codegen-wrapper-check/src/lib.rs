#![allow(dead_code)]

mod generated {
    include!("../../codegen/generated/gemstone_wrappers.rs");
}

pub fn sample_booking() -> generated::BookingDraft {
    generated::BookingDraft {
        name: "example".to_string(),
        amount: 100,
        currency: "GBP".to_string(),
        tags: vec!["codegen".to_string(), "compile-smoke".to_string()],
        note: None,
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
}
