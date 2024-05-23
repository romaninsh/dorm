use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct UniqueIdVendor {
    map: IndexMap<String, String>,
}

impl UniqueIdVendor {
    pub fn new() -> UniqueIdVendor {
        UniqueIdVendor {
            map: IndexMap::new(),
        }
    }

    // If desired_name is taken will add _2, _3, etc.
    pub fn get_uniq_id(&mut self, desired_name: &str) -> String {
        let mut name = desired_name.to_string();
        let mut i = 2;
        while self.map.contains_key(&name) {
            name = format!("{}_{}", desired_name, i);
            i += 1;
        }
        self.map.insert(name.clone(), name.clone());

        name
    }

    // Provided desired names ("n", "na", "nam") find available one
    // If none are available, will add _2, _3 to last option.
    pub fn get_one_of_uniq_id(&mut self, desired_names: Vec<&str>) -> String {
        for name in &desired_names {
            if !self.map.contains_key(*name) {
                self.map.insert(name.to_string(), name.to_string());
                return name.to_string();
            }
        }

        let last_option = desired_names.last().unwrap();
        self.get_uniq_id(last_option)
    }

    pub fn all_prefixes(name: &str) -> Vec<&str> {
        (1..name.len()).into_iter().map(|i| &name[..i]).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_id() {
        let mut vendor = UniqueIdVendor::new();

        assert_eq!(vendor.get_uniq_id("name"), "name");
        assert_eq!(vendor.get_uniq_id("name"), "name_2");
        assert_eq!(vendor.get_uniq_id("name"), "name_3");
        assert_eq!(vendor.get_uniq_id("surname"), "surname");
    }

    #[test]
    fn test_prefixes() {
        assert_eq!(UniqueIdVendor::all_prefixes("name"), vec!["n", "na", "nam"]);
    }

    #[test]
    fn test_one_of_uniq_id() {
        let mut vendor = UniqueIdVendor::new();

        assert_eq!(
            vendor.get_one_of_uniq_id(UniqueIdVendor::all_prefixes("name")),
            "n"
        );
        assert_eq!(
            vendor.get_one_of_uniq_id(UniqueIdVendor::all_prefixes("name")),
            "na"
        );
        assert_eq!(
            vendor.get_one_of_uniq_id(UniqueIdVendor::all_prefixes("name")),
            "nam"
        );
        assert_eq!(
            vendor.get_one_of_uniq_id(UniqueIdVendor::all_prefixes("name")),
            "nam_2"
        );
        assert_eq!(
            vendor.get_one_of_uniq_id(UniqueIdVendor::all_prefixes("name")),
            "nam_3"
        );
    }
}
