#![feature(core_intrinsics)]
#[warn(non_camel_case_types)]

mod bitvector;
mod builder;
pub mod config;
mod label_vector;
mod louds_dense;
mod louds_sparse;
mod popcount;
mod rank;
mod select;
mod suffix;
mod cache;
pub mod trie;


#[cfg(test)]
mod tests {
    use crate::{config::K_NOT_FOUND, trie::Trie};

    #[test]
    fn contains_check() {
        let a = vec![0, 0, 0, 0, 1, 0];
        let b = vec![48, 49, 50, 50, 1, 3];
        let c = vec![49, 49, 229, 0, 1, 0];
        let d = vec![50, 50, 54, 55, 56, 57];
        let keys: Vec<Vec<u8>> = vec![a, b, c, d];
        let trie = Trie::new(&keys);
        for key in keys.iter() {
            let key_id = trie.exact_search(key.as_slice());
            println!("key_id: {}", key_id);
            assert_ne!(key_id, K_NOT_FOUND);
        }

        let not_exist_item_a = vec![50, 50, 54, 55, 56, 0];
        let not_exist_item_b = vec![0, 0, 0, 0, 0, 0];
        let not_exist_item_c = vec![0, 0, 0, 0, 1, 100];
        let not_exist_item_d = vec![50, 50, 0, 55, 56, 57];
        let not_exist_keys: Vec<&[u8]> = vec![
            not_exist_item_a.as_slice(),
            not_exist_item_b.as_slice(),
            not_exist_item_c.as_slice(),
            not_exist_item_d.as_slice(),
        ];
        for key in not_exist_keys.iter() {
            let key_id = trie.exact_search(key);
            println!("key_id: {}", K_NOT_FOUND);
            assert_eq!(key_id, K_NOT_FOUND);
        }
    }

    #[test]
    fn serialize() {
        let a = vec![0, 0, 0, 0, 1, 0];
        let b = vec![48, 49, 50, 50, 1, 3];
        let c = vec![49, 49, 229, 0, 1, 0];
        let d = vec![50, 50, 54, 55, 56, 57];
        let keys: Vec<Vec<u8>> = vec![a, b, c, d];
        let trie = Trie::new(&keys);
        for key in keys.iter() {
            let key_id = trie.exact_search(key.as_slice());
            println!("key_id: {}", key_id);
            assert_ne!(key_id, K_NOT_FOUND);
        }

        let bytes = trie.serialize();
        println!("length : {}", bytes.len());
        let new_trie = Trie::deserialize(bytes.as_slice());
        for key in keys.iter() {
            let key_id = new_trie.exact_search(key.as_slice());
            println!("key_id: {}", key_id);
            assert_ne!(key_id, K_NOT_FOUND);
        }

        let key_id = new_trie.exact_search(vec![0, 0, 0, 0, 1, 0].as_slice());
        assert_ne!(key_id, K_NOT_FOUND);
        let key_id = new_trie.exact_search(vec![0, 0, 0, 0, 0, 0].as_slice());
        assert_eq!(key_id, K_NOT_FOUND);
        let key_id = new_trie.exact_search(vec![1, 0, 0, 0, 0, 0].as_slice());
        assert_eq!(key_id, K_NOT_FOUND);
        let key_id = new_trie.exact_search(vec![0, 0, 0, 0, 0, 1].as_slice());
        assert_eq!(key_id, K_NOT_FOUND);
        let key_id = new_trie.exact_search(vec![10, 10, 10, 10, 1, 100].as_slice());
        assert_eq!(key_id, K_NOT_FOUND);
    }
}
