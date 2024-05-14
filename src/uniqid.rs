use indexmap::IndexMap;

pub struct UniqueIdVendor {
    map: IndexMap<String, String>,
}

impl UniqueIdVendor {
    pub fn new() -> UniqueIdVendor {
        UniqueIdVendor {
            map: IndexMap::new(),
        }
    }

    // If desired_name is taken will add _1, _2, _3, etc.
    pub fn get_uniuqe_id(&mut self, desired_name: &str) -> String {
        let mut name = desired_name.to_string();
        let mut i = 1;
        while self.map.contains_key(&name) {
            name = format!("{}_{}", desired_name, i);
            i += 1;
        }
        self.map.insert(name.clone(), name.clone());

        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_id() {
        let mut vendor = UniqueIdVendor::new();

        assert_eq!(vendor.get_uniuqe_id("name"), "name");
        assert_eq!(vendor.get_uniuqe_id("name"), "name_1");
        assert_eq!(vendor.get_uniuqe_id("name"), "name_2");
        assert_eq!(vendor.get_uniuqe_id("surname"), "surname");
    }
}
