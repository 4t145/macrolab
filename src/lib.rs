#![feature(async_fn_in_trait)]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

mod mat;
mod last;
mod message;
// macro define a BtreeMap
macro_rules! map {
    ($K:ty => $V:ty; $($k:expr => $v:expr),*$(,)?) => {
        {
            let mut _map = std::collections::btree_map::BTreeMap::<$K, $V>::new();
            $(
                _map.insert($k.into(), $v.into());
            )*
            _map
        }
    };
    ($($k:expr => $v:expr),*$(,)?) => {
        {
            let mut _map = std::collections::btree_map::BTreeMap::new();
            $(
                _map.insert($k, $v);
            )*
            _map
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_map_macro() {
        let map = map!{
            String => String;
            "a" => "b",
            "c" => "d"
        };

        assert_eq!(map.get("a"), Some(&"b".to_string()));
        assert_eq!(map.get("c"), Some(&"d".to_string()));
        let map = map!{
            "a" => "b",
            "c" => "d"
        };
        assert_eq!(map.get("a"), Some(&"b"));
        assert_eq!(map.get("c"), Some(&"d"));
    }
    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
