use crate::config::*;
use crate::suffix::BitvectorSuffix;

pub struct Builder {
    // trie level < sparse_start_level_: LOUDS-Dense
    // trie level >= sparse_start_level_: LOUDS-Sparse
    include_dense: bool,
    sparse_dense_ratio: u32,
    sparse_start_level: level_t,

    // LOUDS-Sparse bit/byte vectors
    labels: Vec<Vec<label_t>>,
    child_indicator_bits: Vec<Vec<word_t>>,
    louds_bits: Vec<Vec<word_t>>,

    // LOUDS-Dense bit vectors
    bitmap_labels: Vec<Vec<word_t>>,
    bitmap_child_indicator_bits: Vec<Vec<word_t>>,
    prefixkey_indicator_bits: Vec<Vec<word_t>>,

    suffix_type: SuffixType,
    hash_suffix_len: level_t,
    real_suffix_len: level_t,
    suffixes: Vec<Vec<word_t>>,
    suffix_counts: Vec<position_t>,

    node_counts: Vec<position_t>,
    is_last_item_terminator: Vec<bool>,
}

impl Builder {
    pub fn set_bit(bits: &mut Vec<word_t>, pos: position_t) {
        let word_id: position_t = pos / K_WORD_SIZE;
        let offset: position_t = pos % K_WORD_SIZE;
        bits[word_id] |= K_MSB_MASK >> offset;
    }

    pub fn read_bit(bits: &Vec<word_t>, pos: position_t) -> bool {
        let word_id = pos / K_WORD_SIZE;
        let offset = pos % K_WORD_SIZE;
        bits[word_id] & (K_MSB_MASK >> offset) != 0
    }

    pub fn new(include_dense: bool, sparse_dense_ratio: u32) -> Self {
        Builder {
            include_dense: include_dense,
            sparse_dense_ratio: sparse_dense_ratio,
            sparse_start_level: 0,

            labels: Vec::new(),
            child_indicator_bits: Vec::new(),
            louds_bits: Vec::new(),

            // LOUDS-Dense bit vectors
            bitmap_labels: Vec::new(),
            bitmap_child_indicator_bits: Vec::new(),
            prefixkey_indicator_bits: Vec::new(),

            suffix_type: SuffixType::KNone,
            hash_suffix_len: 0,
            real_suffix_len: 0,
            suffixes: Vec::new(),
            suffix_counts: Vec::new(),
            node_counts: Vec::new(),
            is_last_item_terminator: Vec::new(),
        }
    }

    pub fn get_sparse_start_level(&self) -> level_t {
        self.sparse_start_level
    }

    pub fn get_bit_map_labels(&self) -> &Vec<Vec<word_t>> {
        &self.bitmap_labels
    }

    pub fn get_bitmap_child_indicator_bits(&self) -> &Vec<Vec<word_t>> {
        &self.bitmap_child_indicator_bits
    }

    pub fn get_prefixkey_indicator_bits(&self) -> &Vec<Vec<word_t>> {
        &self.prefixkey_indicator_bits
    }

    pub fn get_suffix_type(&self) -> SuffixType {
        self.suffix_type
    }

    pub fn get_suffix_counts(&self) -> &Vec<position_t> {
        &self.suffix_counts
    }

    pub fn get_labels(&self) -> &Vec<Vec<label_t>> {
        &self.labels
    }

    pub fn get_node_counts(&self) -> &Vec<position_t> {
        &self.node_counts
    }

    pub fn get_child_indicator_bits(&self) -> &Vec<Vec<word_t>> {
        &self.child_indicator_bits
    }

    pub fn get_louds_bits(&self) -> &Vec<Vec<word_t>> {
        &self.louds_bits
    }

    pub fn build(&mut self, keys: &Vec<Vec<u8>>) {
        self.build_sparse(keys);
        if self.include_dense {
            self.determine_cutoff_level();
            self.build_dense();
        }
    }

    fn build_sparse(&mut self, keys: &Vec<Vec<u8>>) {
        let mut i = 0;
        while i < keys.len() {
            let mut level: level_t = self.skip_common_prefix(keys[i].as_slice());
            let curpos = i;
            while (i + 1 < keys.len()) && Builder::is_same_key(keys[curpos].as_slice(), keys[i + 1].as_slice()) {
                i += 1;
            }
            if i < keys.len() - 1 {
                level =
                    self.insert_key_bytes_to_trie_until_unique(keys[curpos].as_slice(), keys[i + 1].as_slice(), level);
            } else {
                level = self.insert_key_bytes_to_trie_until_unique(keys[curpos].as_slice(), &[], level);
            }
            // suffixだけ別で管理したいのでnext_keyと比較している
            // TODO: FSAにするならここを変える必要がありそう，同じsuffixがすでにあれば
            self.insert_suffix(level);
            i += 1;
        }
    }

    // 1つ前のprefixと比較している
    fn skip_common_prefix(&mut self, key: &key_t) -> level_t {
        let mut level: level_t = 0;
        while level < key.len() && self.is_char_common_prefix(key[level], level) {
            let pos = self.get_num_items(level) - 1;
            let bits = &mut self.child_indicator_bits[level];
            Builder::set_bit(bits, pos);
            level += 1;
        }
        level
    }

    fn is_char_common_prefix(&self, c: label_t, level: level_t) -> bool {
        (level < self.get_tree_height())
            && (self.is_last_item_terminator.get(level).is_none()
                || !self.is_last_item_terminator[level])
            && (c == *self.labels[level].last().unwrap())
    }

    fn get_tree_height(&self) -> level_t {
        self.labels.len()
    }

    fn get_num_items(&self, level: level_t) -> position_t {
        self.labels[level].len()
    }

    fn is_same_key(a: &key_t, b: &key_t) -> bool {
        a == b
    }

    fn insert_key_bytes_to_trie_until_unique(
        &mut self,
        key: &key_t,
        next_key: &key_t,
        start_level: level_t,
    ) -> level_t {
        let mut level: level_t = start_level;
        let mut is_start_of_node: bool = false;
        let mut is_term: bool = false;

        if self.is_level_empty(level) {
            is_start_of_node = true; // ノードの中で長男ということ
        }

        // After skipping the common prefix, the first following byte
        // shoud be in an the node as the previous key.
        self.insert_key_byte(key[level], level, is_start_of_node, is_term);
        level += 1;
        if level > next_key.len() || !Builder::is_same_key(&key[..level], &next_key[..level]) {
            return level;
        }

        // All the following bytes inserted must be the start of a
        // new node.
        is_start_of_node = true;
        while level < key.len() && level < next_key.len() && key[level] == next_key[level] {
            self.insert_key_byte(key[level], level, is_start_of_node, is_term);
            level += 1;
        }

        // The last byte inserted makes key unique in the trie.
        if level < key.len() {
            self.insert_key_byte(key[level], level, is_start_of_node, is_term);
        } else {
            // TODO: 深さ全部同じなので最後の値を入れる必要はないはず
            is_term = true;
            self.insert_key_byte(K_TERMINATOR, level, is_start_of_node, is_term);
        }
        level += 1;

        return level;
    }

    fn is_level_empty(&self, level: level_t) -> bool {
        (level >= self.get_tree_height()) || (self.labels[level].len() == 0)
    }

    fn insert_key_byte(
        &mut self,
        c: node_t,
        level: level_t,
        is_start_of_node: bool,
        is_term: bool,
    ) {
        // level should be at most equal to tree height
        if level >= self.get_tree_height() {
            self.add_level();
        }
        // sets parent node's child indicator
        if level > 0 {
            let pos = self.get_num_items(level - 1) - 1;
            let bits = &mut self.child_indicator_bits[level - 1];
            Builder::set_bit(bits, pos);
        }
        self.labels[level].push(c);
        if is_start_of_node {
            let pos = self.get_num_items(level) - 1;
            let bits = &mut self.louds_bits[level];
            Builder::set_bit(bits, pos);
            self.node_counts[level] += 1;
        }
        self.is_last_item_terminator[level] = is_term;
        self.move_to_next_item_slot(level);
    }

    // 未知なる深さへ
    fn add_level(&mut self) {
        self.labels.push(Vec::new());
        let height = self.get_tree_height();
        self.child_indicator_bits.push(Vec::new());
        self.louds_bits.push(Vec::new());
        self.suffixes.push(Vec::new());
        self.suffix_counts.push(0);

        self.node_counts.push(0);
        self.is_last_item_terminator.push(false);

        self.child_indicator_bits[height - 1].push(0);
        self.louds_bits[height - 1].push(0);
    }

    // S-HasChildとS-LOUDSはそれぞれ64bitずつ増やす
    fn move_to_next_item_slot(&mut self, level: level_t) {
        let num_items = self.get_num_items(level);
        if num_items % K_WORD_SIZE == 0 {
            self.child_indicator_bits[level].push(0);
            self.louds_bits[level].push(0);
        }
    }

    fn insert_suffix(&mut self, level: level_t) {
        if level >= self.get_tree_height() {
            self.add_level();
        }

        let suffix_word = BitvectorSuffix::construct_suffix(self.suffix_type);
        self.store_suffix(level, suffix_word);
    }

    fn store_suffix(&mut self, level: level_t, suffix: word_t) {
        let suffix_len: level_t = self.get_suffix_len();
        let pos: position_t = self.suffix_counts[level - 1] * suffix_len;
        assert!(pos <= self.suffixes[level - 1].len() * suffix_len);
        if pos == (self.suffixes[level - 1].len() * K_WORD_SIZE) {
            self.suffixes[level - 1].push(0);
        }
        let mut word_id = pos / K_WORD_SIZE;
        let offset = pos % K_WORD_SIZE;
        let word_remaining_len = K_WORD_SIZE - offset;
        if suffix_len <= word_remaining_len {
            let shifted_suffix = suffix << (word_remaining_len - suffix_len) % 64;
            self.suffixes[level - 1][word_id] += shifted_suffix;
        } else {
            let suffix_left_part = suffix >> (suffix_len - word_remaining_len);
            self.suffixes[level - 1][word_id] += suffix_left_part;
            self.suffixes[level - 1].push(0);
            word_id += 1;
            let suffix_right_part = suffix << (K_WORD_SIZE - (suffix_len - word_remaining_len));
            self.suffixes[level - 1][word_id] += suffix_right_part;
        }
        self.suffix_counts[level - 1] += 1;
    }

    fn determine_cutoff_level(&mut self) {
        let mut cutoff_level: level_t = 0;
        let mut dense_mem: u64 = self.compute_dense_mem(cutoff_level);
        let mut sparse_mem = self.compute_sparse_mem(cutoff_level);
        while (cutoff_level < self.get_tree_height())
            && (dense_mem * (self.sparse_dense_ratio as u64) < sparse_mem)
        {
            cutoff_level += 1;
            dense_mem = self.compute_dense_mem(cutoff_level);
            sparse_mem = self.compute_sparse_mem(cutoff_level);
        }
        self.sparse_start_level = cutoff_level as level_t;
    }

    fn compute_dense_mem(&self, downto_level: level_t) -> u64 {
        let mut mem: u64 = 0;
        for level in 0..downto_level {
            mem += (2 * K_FANOUT * self.node_counts[level]) as u64;
            if level > 0 {
                mem += (self.node_counts[level - 1] / 8 + 1) as u64;
            }
            mem += (self.suffix_counts[level] * self.get_suffix_len() / 8) as u64
        }
        mem
    }

    fn get_suffix_len(&self) -> level_t {
        self.hash_suffix_len + self.real_suffix_len
    }

    fn compute_sparse_mem(&self, start_level: level_t) -> u64 {
        let mut mem: u64 = 0;
        for level in start_level..self.get_tree_height() {
            let num_items: position_t = self.labels[level].len();
            mem += (num_items + 2 * num_items / 8 + 1) as u64;
            mem += (self.suffix_counts[level] * self.get_suffix_len() / 8) as u64;
        }
        mem
    }

    fn build_dense(&mut self) {
        for level in 0..self.sparse_start_level {
            self.init_dense_vectors(level);
            if self.get_num_items(level) == 0 {
                continue;
            }

            let mut node_num: position_t = 0;
            if self.is_terminator(level, 0) {
                Builder::set_bit(&mut self.prefixkey_indicator_bits[level], 0);
            } else {
                self.set_label_and_child_indicator_bitmap(level, node_num, 0);
            }
            for pos in 1..self.get_num_items(level) {
                if self.is_start_of_node(level, pos) {
                    node_num += 1;
                    if self.is_terminator(level, pos) {
                        Builder::set_bit(&mut self.prefixkey_indicator_bits[level], node_num);
                        continue;
                    }
                }
                self.set_label_and_child_indicator_bitmap(level, node_num, pos);
            }
        }
    }

    fn init_dense_vectors(&mut self, level: level_t) {
        self.bitmap_labels.push(Vec::new());
        self.bitmap_child_indicator_bits.push(Vec::new());
        self.prefixkey_indicator_bits.push(Vec::new());

        for nc in 0..self.node_counts[level] {
            let mut i = 0;
            while i < K_FANOUT {
                self.bitmap_labels[level].push(0);
                self.bitmap_child_indicator_bits[level].push(0);
                i += K_WORD_SIZE;
            }
            if nc % K_WORD_SIZE == 0 {
                self.prefixkey_indicator_bits[level].push(0);
            }
        }
    }

    fn is_terminator(&self, level: level_t, pos: position_t) -> bool {
        let label: label_t = self.labels[level][pos];
        (label == K_TERMINATOR) && !Builder::read_bit(&self.child_indicator_bits[level], pos)
    }

    fn is_start_of_node(&self, level: level_t, pos: position_t) -> bool {
        Builder::read_bit(&self.louds_bits[level], pos)
    }

    fn set_label_and_child_indicator_bitmap(
        &mut self,
        level: level_t,
        node_num: position_t,
        pos: position_t,
    ) {
        let label: label_t = self.labels[level][pos];
        Builder::set_bit(
            &mut self.bitmap_labels[level],
            node_num * K_FANOUT + label as usize,
        );
        if Builder::read_bit(&self.child_indicator_bits[level], pos) {
            Builder::set_bit(
                &mut self.bitmap_child_indicator_bits[level],
                node_num * K_FANOUT + label as usize,
            );
        }
    }
}
