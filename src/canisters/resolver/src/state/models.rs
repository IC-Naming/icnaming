use std::collections::HashMap;

use candid::{CandidType, Deserialize};

use crate::models::Resolver;

#[derive(Debug, Clone, CandidType, Deserialize)]
pub(crate) struct ResolverStable {
    name: String,
    string_value_map: HashMap<String, String>,
}

// &Resolver to ResolverStable
impl From<&Resolver> for ResolverStable {
    fn from(resolver: &Resolver) -> Self {
        let string_value_map = resolver.get_record_value().clone();
        ResolverStable {
            name: resolver.get_name().clone(),
            string_value_map,
        }
    }
}

// &ResolverStable to Resolver
impl From<&ResolverStable> for Resolver {
    fn from(resolver_stable: &ResolverStable) -> Self {
        let mut result = Resolver::new(resolver_stable.name.clone());
        result.set_string_map(&resolver_stable.string_value_map.clone());
        result
    }
}
