use std::{any::TypeId, collections::HashMap, sync::LazyLock};

use parking_lot::RwLock;

static CACHE: LazyLock<RwLock<HashMap<TypeId, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn short_type_name<T: ?Sized + 'static>() -> String {
    let type_id = TypeId::of::<T>();

    {
        let cache = CACHE.read();

        if let Some(cached) = cache.get(&type_id) {
            return cached.clone();
        }
    }

    let short_name = short_type_name_impl::<T>();

    {
        let mut cache = CACHE.write();
        cache.insert(type_id, short_name.clone());
    }

    short_name
}

fn short_type_name_impl<T: ?Sized>() -> String {
    let mut name = std::any::type_name::<T>();
    let mut result = String::new();

    while let Some(index) = name.find([' ', '<', '>', '(', ')', '[', ']', ',', ';']) {
        let (segment, rest) = name.split_at(index);

        result += segment.rsplit(':').next().unwrap();
        result.push_str(&rest[..1]);
        name = &rest[1..];
    }

    if !name.is_empty() {
        result += name.rsplit(':').next().unwrap();
    }

    result
}
