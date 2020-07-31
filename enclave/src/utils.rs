use std::vec::Vec;
use primitive::*;


// なんかめっちゃ長くなってしまったけど
pub fn _sorted_merge(sorted_list_1: &Vec<UnixEpoch>, sorted_list_2: &Vec<UnixEpoch>) -> Vec<UnixEpoch> {
    let len1 = sorted_list_1.len();
    let len2 = sorted_list_2.len();
    let size = len1 + len2;
    let mut cursor1 = 0;
    let mut cursor2 = 0;
    let mut tmp_max = 0;

    let mut merged_vec = Vec::with_capacity(size);
    for _ in 0..size {
        let mut candidate = if sorted_list_1[cursor1] < sorted_list_2[cursor2] {
            cursor1 += 1;
            sorted_list_1[cursor1 - 1]
        } else if sorted_list_1[cursor1] == sorted_list_2[cursor2] {
            cursor1 += 1;
            cursor2 += 1;
            sorted_list_1[cursor1 - 1]
        } else {
            cursor2 += 1;
            sorted_list_2[cursor2 - 1]
        };
        
        if tmp_max != candidate { 
            tmp_max = candidate; 
            merged_vec.push(tmp_max);
        }
        
        if len1 == cursor1 && cursor2 < len2 {
            for j in cursor2..len2 {
                candidate = sorted_list_2[j];
                if tmp_max != candidate { tmp_max = candidate; } else { continue; };
                merged_vec.push(candidate);
            }
            break;
        }
        if len2 == cursor2 && cursor1 < len1 {
            for j in cursor1..len1 {
                candidate = sorted_list_1[j];
                if tmp_max != candidate { tmp_max = candidate; } else { continue; };
                merged_vec.push(candidate);
            }
            break;
        }
        if len1 == cursor1 && len2 == cursor2 { break; }
    }
    merged_vec
}

// 昇順ソート+ユニーク性
// あえてジェネリクスにする必要はない，むしろ型で守っていく
// Vecだと遅いけどLinkedListよりはキャッシュに乗るので早い気がするのでVecでいく
pub fn _sorted_push(sorted_list: &mut Vec<UnixEpoch>, unixepoch: UnixEpoch) {
    let mut index = 0;
    for elm in sorted_list.iter() {
        if *elm > unixepoch {
            sorted_list.insert(index, unixepoch);
            return;
        } else if *elm == unixepoch {
            return;
        } else {
            index += 1;
        }
    }
    sorted_list.push(unixepoch);
}