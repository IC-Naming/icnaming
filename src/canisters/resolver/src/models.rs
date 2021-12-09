use std::collections::HashMap;

pub struct Resolver {
    name: String,
    string_value_map: HashMap<String, String>,
}

impl Resolver {
    pub fn new(name: String) -> Resolver {
        Resolver {
            name,
            string_value_map: HashMap::new(),
        }
    }
    pub(crate) fn get_name(&self) -> &String {
        &self.name
    }
    pub(crate) fn set_string_map(&mut self, map: &HashMap<String, String>) {
        self.string_value_map = map.clone();
    }
    pub fn set_record_value(&mut self, key: String, value: String) {
        self.string_value_map.insert(key, value);
    }
    pub fn remove_record_value(&mut self, key: String) {
        self.string_value_map.remove(&key);
    }

    pub(crate) fn get_record_value(&self) -> &HashMap<String, String> {
        &self.string_value_map
    }
}
