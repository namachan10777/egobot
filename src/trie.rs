// copyright (c) 2019 Nakano Masaki <namachan10777@gmail>
use serde::Serialize;
use serde_derive::{Deserialize, Serialize};
use std::usize;

#[derive(Clone, PartialEq, Debug, Copy, Serialize, Deserialize)]
struct Node {
    base: usize,
    check: usize,
    id: usize,
}
impl Default for Node {
    fn default() -> Node {
        Node::from(DecodedNode::Blank)
    }
}

impl Node {
    fn root(base: usize) -> Self {
        Node::from(DecodedNode::Root(base))
    }

    fn term(check: usize, id: usize) -> Self {
        Node::from(DecodedNode::Term(check, id))
    }

    // for test
    #[allow(dead_code)]
    fn sec(check: usize, base: usize, id: Option<usize>) -> Self {
        Node::from(DecodedNode::Sec(check, base, id))
    }

    fn node(check: usize, base: usize, id: usize) -> Self {
        Node { check, base, id }
    }

    fn blank() -> Self {
        Node::default()
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum DecodedNode {
    // base
    Root(usize),
    // check, base, id
    Sec(usize, usize, Option<usize>),
    // check, base
    Term(usize, usize),
    Blank,
}
impl Default for DecodedNode {
    fn default() -> DecodedNode {
        DecodedNode::Blank
    }
}

const NO_PARENT: usize = usize::MAX;
const NO_ITEM: usize = usize::MAX;
const NO_CHILD: usize = usize::MAX;

impl Into<DecodedNode> for Node {
    fn into(self) -> DecodedNode {
        if self.check == NO_PARENT {
            if self.base == NO_CHILD {
                DecodedNode::Blank
            } else {
                DecodedNode::Root(self.base)
            }
        } else if self.base == NO_CHILD {
            DecodedNode::Term(self.check, self.id)
        } else if self.id == NO_ITEM {
            DecodedNode::Sec(self.check, self.base, None)
        } else {
            DecodedNode::Sec(self.check, self.base, Some(self.id))
        }
    }
}

impl From<DecodedNode> for Node {
    fn from(dnode: DecodedNode) -> Self {
        match dnode {
            DecodedNode::Root(base) => Node {
                base,
                check: NO_PARENT,
                id: 0,
            },
            DecodedNode::Term(check, id) => Node {
                base: NO_CHILD,
                check,
                id,
            },
            DecodedNode::Sec(check, base, None) => Node {
                base,
                check,
                id: NO_ITEM,
            },
            DecodedNode::Sec(check, base, Some(id)) => Node { base, check, id },
            DecodedNode::Blank => Node {
                base: NO_CHILD,
                check: NO_PARENT,
                id: NO_ITEM,
            },
        }
    }
}

#[cfg(test)]
mod node_test {
    use super::*;

    #[test]
    fn test_from_into() {
        let root_decoded = DecodedNode::Root(129);
        let root_raw = Node {
            base: 129,
            check: NO_PARENT,
            id: 0,
        };
        let term_decoded = DecodedNode::Term(2158, 87);
        let term_raw = Node {
            base: NO_CHILD,
            check: 2158,
            id: 87,
        };
        let sec_no_property_decoded = DecodedNode::Sec(52128, 59182, None);
        let sec_no_property_raw = Node {
            base: 59182,
            check: 52128,
            id: NO_ITEM,
        };
        let sec_has_property_decoded = DecodedNode::Sec(711475, 365123, Some(214));
        let sec_has_property_raw = Node {
            base: 365123,
            check: 711475,
            id: 214,
        };
        assert_eq!(Node::from(root_decoded.clone()), root_raw);
        assert_eq!(Into::<DecodedNode>::into(root_raw), root_decoded);

        assert_eq!(Node::from(term_decoded.clone()), term_raw);
        assert_eq!(Into::<DecodedNode>::into(term_raw), term_decoded);

        assert_eq!(
            Node::from(sec_no_property_decoded.clone()),
            sec_no_property_raw
        );
        assert_eq!(
            Into::<DecodedNode>::into(sec_no_property_raw),
            sec_no_property_decoded
        );

        assert_eq!(
            Node::from(sec_has_property_decoded.clone()),
            sec_has_property_raw
        );
        assert_eq!(
            Into::<DecodedNode>::into(sec_has_property_raw),
            sec_has_property_decoded
        );
    }
}

#[derive(Serialize, Deserialize)]
pub struct Trie<T: Serialize> {
    capacities: Vec<u8>,
    cache: Vec<usize>,
    // 圧縮済みの遷移表
    tree: Vec<Node>,
    // 辞書本体
    storage: Vec<Vec<T>>,
}

const ROW_LEN: usize = 256;
type Row = [Node; ROW_LEN];

impl<T: Serialize> Default for Trie<T> {
    fn default() -> Trie<T> {
        let mut tree = vec![Node::blank(); 256];
        tree[0] = Node::root(0);
        Trie {
            cache: vec![0; 256],
            capacities: vec![254],
            tree,
            storage: Vec::new(),
        }
    }
}

impl<T: Serialize> Trie<T> {
    // Ok(idx)
    // Err((passed times, last idx))
    fn explore(&self, way: &[u8]) -> Result<usize, (usize, usize)> {
        let mut here = 0usize;
        for (octet_count, octet) in way.iter().enumerate() {
            let check = here;
            if self.tree[here].base == NO_CHILD {
                return Err((octet_count, check));
            }
            here = self.tree[here].base ^ (*octet as usize);
            if self.tree[here].check != check {
                return Err((octet_count, check));
            }
        }
        Ok(here)
    }
}
#[cfg(test)]
mod test_explore {
    use super::*;
    #[test]
    fn test_explore() {
        let mut tree = Vec::new();
        tree.resize(1024, Node::blank());
        /* Root(0): 0 -+- 1 -> Term(0): 1
         *             |
         *             +- 2 -> Sec(0,4): 2 -+- 2 -> Term(2): 6
         *                                  |
         *                                  +- 3 -> Sec(2, 4): 7 -+- 1 -> Term(7): 5
         */
        tree[0] = Node::root(0);
        tree[1] = Node::term(0, 0);
        tree[2] = Node::sec(0, 4, Some(1));
        tree[6] = Node::term(2, 2);
        tree[7] = Node::sec(2, 4, Some(3));
        tree[5] = Node::term(7, 4);
        let trie = Trie {
            cache: vec![0; 256],
            capacities: vec![249, 255],
            tree,
            storage: Vec::new() as Vec<Vec<String>>,
        };
        assert_eq!(trie.explore(&[1]), Ok(1));
        assert_eq!(trie.explore(&[2]), Ok(2));
        assert_eq!(trie.explore(&[2, 2]), Ok(6));
        assert_eq!(trie.explore(&[2, 3]), Ok(7));
        assert_eq!(trie.explore(&[2, 3, 1]), Ok(5));
        assert_eq!(trie.explore(&[3]), Err((0, 0)));
        assert_eq!(trie.explore(&[2, 1, 0]), Err((1, 2)));
        assert_eq!(trie.explore(&[2, 3, 0]), Err((2, 7)));
    }
}

impl<T: Serialize> Trie<T> {
    // To reallocate base and expand tree if need to do.
    // FIXME bottleneck
    fn reallocate_base(&mut self, target: &[bool; 256], cnt: u8) -> usize {
        for block_idx in self.cache[cnt as usize]..self.capacities.len() {
            if self.capacities[block_idx] >= cnt {
                for innser_offset in 0..256 {
                    let mut safe = true;
                    let offset = (block_idx << 8) | innser_offset;
                    for (target_idx, target) in target.iter().enumerate() {
                        if *target
                            && DecodedNode::Blank
                                != Into::<DecodedNode>::into(self.tree[offset ^ target_idx])
                        {
                            safe = false;
                            break;
                        }
                    }
                    if safe {
                        for i in (cnt as usize)..self.cache.len() {
                            self.cache[i] = std::cmp::max(self.cache[i], block_idx);
                        }
                        return offset;
                    }
                }
            }
        }
        let half = self.tree.len();
        // expand tree
        self.tree.resize(half * 2, Node::blank());
        self.capacities.resize(self.capacities.len() * 2, 255);
        // search base straddling border of allocated area.
        half
    }
}

#[cfg(test)]
mod test_reallocate_base {
    use super::*;
    #[test]
    fn test_reallocate_base() {
        let mut mask = [false; 256];
        mask[0] = true;
        let mut tree = vec![Node::term(0, 0); 512];
        tree[6] = Node::blank();
        let mut trie: Trie<String> = Trie {
            cache: vec![0; 256],
            capacities: vec![1, 0],
            tree,
            storage: Vec::new(),
        };
        assert_eq!(trie.reallocate_base(&mask, 1), 0 ^ 6);

        mask[0] = false;
        mask[47] = true;
        assert_eq!(trie.reallocate_base(&mask, 1), 6 ^ 47);

        mask[47] = true;
        mask[99] = true;
        trie.tree = vec![Node::blank(); 512];
        trie.tree[47] = Node::term(0, 0);
        trie.tree[1 ^ 99] = Node::term(0, 0);
        trie.capacities = vec![253, 255];
        assert_eq!(trie.reallocate_base(&mask, 2), 2);

        mask[47] = false;
        mask[99] = false;
        mask[0] = true;
        trie.tree = vec![Node::term(0, 0); 512];
        trie.tree[511] = Node::blank();
        trie.capacities = vec![0, 1];
        assert_eq!(trie.reallocate_base(&mask, 1), 511);
        assert_eq!(trie.tree.len(), 512);

        trie.tree = vec![Node::term(0, 0); 512];
        trie.capacities = vec![0, 0];
        assert_eq!(trie.reallocate_base(&mask, 1), 512);
        assert_eq!(trie.tree.len(), 1024);
    }
}

impl<T: Serialize> Trie<T> {
    fn read_row(&self, parent_idx: usize) -> Row {
        let mut buf: Row = [Node::blank(); 256];
        let base = self.tree[parent_idx].base;
        for (i, buf_i) in buf.iter_mut().enumerate() {
            if self.tree[base ^ i].check == parent_idx {
                *buf_i = self.tree[base ^ i];
            }
        }
        buf
    }

    fn erase_row(&mut self, parent_idx: usize) {
        let base = self.tree[parent_idx].base;
        for i in 0..256 {
            if self.tree[base ^ i].check == parent_idx {
                self.tree[base ^ i] = Node::blank();
            }
        }
    }

    fn count_children(&self, parent_idx: usize) -> usize {
        let mut cnt = 0usize;
        let base = self.tree[parent_idx].base;
        for i in 0..256 {
            if self.tree[base ^ i].check == parent_idx {
                cnt += 1;
            }
        }
        cnt
    }
}

#[cfg(test)]
mod test_read_erase_row {
    use super::*;
    #[test]
    fn test_read() {
        let mut tree = [Node::blank(); 512].to_vec();
        tree[0] = Node::root(0);
        tree[1] = Node::sec(0, 64, None);
        tree[2] = Node::term(0, 0);
        tree[64] = Node::term(1, 0);
        let trie: Trie<String> = Trie {
            cache: vec![0; 256],
            capacities: vec![251, 255],
            tree,
            storage: Vec::new(),
        };

        let row1 = trie.read_row(0).to_vec();
        let mut row1_ans = vec![Node::blank(); 256];
        row1_ans[1] = Node::sec(0, 64, None);
        row1_ans[2] = Node::term(0, 0);
        assert_eq!(row1, row1_ans);

        let row2 = trie.read_row(1).to_vec();
        let mut row2_ans = vec![Node::blank(); 256];
        row2_ans[0] = Node::term(1, 0);
        assert_eq!(row2, row2_ans)
    }

    #[test]
    fn test_erase() {
        let mut tree = [Node::blank(); 512].to_vec();
        tree[0] = Node::root(0);
        tree[1] = Node::sec(0, 64, None);
        tree[2] = Node::term(0, 0);
        tree[64] = Node::term(1, 0);
        let mut trie: Trie<String> = Trie {
            cache: vec![0; 256],
            capacities: vec![251, 255],
            tree,
            storage: Vec::new(),
        };

        trie.erase_row(0);
        let mut tree1 = [Node::blank(); 512].to_vec();
        tree1[0] = Node::root(0);
        tree1[64] = Node::term(1, 0);
        assert_eq!(trie.tree, tree1);
    }
}

impl<T: Serialize> Trie<T> {
    // This function forcely overwrite tree
    // 存在しなかったのにrowに入っているとfromを誤認する
    fn paste(&mut self, row: Row, addition: Row, from: usize) -> usize {
        // make mask
        let mut mask = [false; 256];
        let mut cnt = 0;
        for i in 0..256 {
            if row[i].check != NO_PARENT
                || row[i].base != NO_CHILD
                || addition[i].check != NO_PARENT
                || addition[i].base != NO_CHILD
            {
                mask[i] = true;
                cnt += 1;
            }
        }
        let to = self.reallocate_base(&mask, cnt);
        self.capacities[to >> 8] -= cnt;
        // place
        for (i, row) in row.iter().enumerate() {
            if row.check != NO_PARENT {
                // place bro
                self.tree[to ^ i] = *row;
                // update children's check
                for j in 0..256 {
                    if row.base != NO_CHILD && self.tree[row.base ^ j].check == from ^ i {
                        self.tree[row.base ^ j].check = to ^ i;
                    }
                }
            }
        }
        // additional placement without updation of children's check
        for (i, addition) in addition.iter().enumerate() {
            if addition.check != NO_PARENT || addition.base != NO_CHILD {
                self.tree[to ^ i] = *addition;
            }
        }
        to
    }
}

#[cfg(test)]
mod test_paste {
    // TODO improve test case
    use super::*;
    #[test]
    fn test_paste() {
        let mut tree = vec![Node::blank(); 512];
        tree[0] = Node::root(0);
        tree[64] = Node::term(5, 0);

        let mut row = [Node::blank(); 256];
        row[1] = Node::sec(0, 64, None);
        row[2] = Node::term(1, 0);

        let mut trie: Trie<String> = Trie {
            cache: vec![0; 256],
            capacities: vec![253, 255],
            tree,
            storage: Vec::new(),
        };

        assert_eq!(trie.paste(row, [Node::default(); 256], 4), 0);
        let mut ans = vec![Node::blank(); 512];
        ans[0] = Node::root(0);
        ans[64] = Node::term(1, 0);
        ans[1] = Node::sec(0, 64, None);
        ans[2] = Node::term(1, 0);

        assert_eq!(trie.tree, ans);

        let mut tree2 = vec![Node::blank(); 512];
        tree2[0] = Node::root(0);
        let mut trie2: Trie<String> = Trie {
            cache: vec![0; 256],
            capacities: vec![251, 255],
            tree: tree2,
            storage: Vec::new(),
        };
        let mut row = [Node::blank(); 256];
        row[0] = Node::sec(0, 0, None);
        assert_eq!(trie2.paste([Node::blank(); 256], row, 0), 1);
        let mut ans = vec![Node::blank(); 512];
        ans[0] = Node::root(0);
        ans[1] = Node::sec(0, 0, None);
        assert_eq!(&trie2.tree, &ans);
    }
}

impl<T: Serialize> Trie<T> {
    fn insert_by_push_out(&mut self, target_idx: usize, parent_idx: usize) -> usize {
        let parent = self.tree[parent_idx];
        let target = self.tree[target_idx];

        if target.check == NO_PARENT {
            if target.base == NO_CHILD {
                return target_idx;
            } else {
                unreachable!();
            }
        }

        let old_base = if parent.check < self.tree.len() {
            self.tree[parent.check].base
        } else {
            NO_CHILD
        };
        let row = self.read_row(target.check);
        self.erase_row(target.check);
        let parent_moved = self.tree[parent_idx] == Node::blank();
        // insert dummy
        self.tree[target_idx] = Node::node(0, NO_CHILD, NO_ITEM);
        // replace parent
        let new_base = self.paste(row, [Node::blank(); 256], self.tree[target.check].base);
        // update parent of target
        self.tree[target.check].base = new_base;
        // if parent was included in target of push_out
        // 親のcheckが変わっただけでもここの判定に入ってしまう
        if parent_moved {
            // old_base ^ parent_idx: relative position
            // (old_base ^ parent_idx) ^ new_base: new absolute position
            // A ^ B = C ⇒ C ^ A = B ∩ C ^ B = A
            self.tree[target_idx] = Node::term(old_base ^ parent_idx ^ new_base, NO_ITEM);
        } else {
            self.tree[target_idx] = Node::term(parent_idx, NO_ITEM);
        }
        target_idx
    }

    fn insert_by_slide_brothers(&mut self, target_idx: usize, parent_idx: usize) -> usize {
        let parent = self.tree[parent_idx];
        let row = self.read_row(parent_idx);
        self.erase_row(parent_idx);
        let mut addition = [Node::blank(); 256];
        addition[parent.base ^ target_idx] = Node::node(parent_idx, NO_CHILD, NO_ITEM);
        let new_base = self.paste(row, addition, parent.base);
        self.tree[parent_idx].base = new_base;
        target_idx ^ parent.base ^ new_base
    }
}

impl<T: Serialize> Trie<T> {
    pub fn add(&mut self, way: &[u8], cargo: T) -> Result<(), ()> {
        let mut parent_idx = 0;
        for octet in way {
            if self.tree[parent_idx].base == NO_CHILD {
                self.tree[parent_idx].base = 0;
            }
            let child_idx = self.tree[parent_idx].base ^ (*octet as usize);
            let child = self.tree[child_idx];
            if child.check == NO_PARENT {
                if child.base == NO_CHILD {
                    self.tree[child_idx] = Node::node(parent_idx, NO_CHILD, NO_ITEM);
                    parent_idx = child_idx;
                }
                // root case
                else {
                    parent_idx = self.insert_by_slide_brothers(child_idx, parent_idx);
                }
            } else if child.check == parent_idx {
                parent_idx = child_idx;
            }
            // conflict case
            else {
                let brother_num = self.count_children(parent_idx);
                let stranger_num = self.count_children(child.check);
                parent_idx = if brother_num > stranger_num {
                    self.insert_by_push_out(child_idx, parent_idx)
                } else {
                    self.insert_by_slide_brothers(child_idx, parent_idx)
                };
            }
        }
        self.tree[parent_idx].id = if self.tree[parent_idx].id == NO_ITEM {
            self.storage.push(vec![cargo]);
            self.storage.len() - 1
        } else {
            self.storage[self.tree[parent_idx].id].push(cargo);
            self.tree[parent_idx].id
        };
        Ok(())
    }
}

impl<T: Serialize> Trie<T> {
    pub fn find(&self, way: &[u8]) -> Result<&[T], ()> {
        match self.explore(way) {
            Ok(idx) => match Into::<DecodedNode>::into(self.tree[idx]) {
                DecodedNode::Blank => Err(()),
                DecodedNode::Root(_) => Err(()),
                DecodedNode::Term(_, id) => Ok(&self.storage[id][..]),
                DecodedNode::Sec(_, _, Some(id)) => Ok(&self.storage[id][..]),
                DecodedNode::Sec(_, _, None) => Err(()),
            },
            Err(_) => Err(()),
        }
    }
}

#[cfg(test)]
mod test_add_find {
    use super::*;
    #[test]
    fn test_add_find() {
        let mut trie: Trie<String> = Trie::default();
        trie.add(&[2, 1], String::from("21")).unwrap();
        trie.add(&[1, 1], String::from("11")).unwrap();
        trie.add(&[1, 2, 3], String::from("123")).unwrap();
        trie.add(&[0], String::from("0")).unwrap();
        trie.add(&[0, 0], String::from("00")).unwrap();
        trie.add(&[1, 2], String::from("12")).unwrap();
        trie.add(&[1, 2, 0], String::from("120")).unwrap();
        trie.add(&[3, 1, 3], String::from("313")).unwrap();
        trie.add(&[1, 6, 1], String::from("161")).unwrap();
        trie.add(&[0, 1], String::from("01")).unwrap();
        trie.add(&[2, 0], String::from("20")).unwrap();

        assert_eq!(trie.find(&[0]), Ok(&[String::from("0")][..]));
        assert_eq!(trie.find(&[0]), Ok(&[String::from("0")][..]));
        assert_eq!(trie.find(&[0, 0]), Ok(&[String::from("00")][..]));
        assert_eq!(trie.find(&[1, 2, 3]), Ok(&[String::from("123")][..]));
        assert_eq!(trie.find(&[1, 2]), Ok(&[String::from("12")][..]));
        assert_eq!(trie.find(&[1, 2, 0]), Ok(&[String::from("120")][..]));
        assert_eq!(trie.find(&[3, 1, 3]), Ok(&[String::from("313")][..]));
        assert_eq!(trie.find(&[1, 6, 1]), Ok(&[String::from("161")][..]));
        assert_eq!(trie.find(&[0, 1]), Ok(&[String::from("01")][..]));
        assert_eq!(trie.find(&[2, 0]), Ok(&[String::from("20")][..]));
        assert_eq!(trie.find(&[2, 1]), Ok(&[String::from("21")][..]));
        assert_eq!(trie.find(&[1]), Err(()));
        assert_eq!(trie.find(&[7, 4]), Err(()));
    }

    #[test]
    fn test_add() {
        let mut trie: Trie<String> = Trie::default();
        trie.add("張り込め".as_bytes(), String::from("張り込め"))
            .unwrap();
        trie.add(
            "ニッカーボッカー".as_bytes(),
            String::from("ニッカーボッカー"),
        )
        .unwrap();
        trie.add("証城寺".as_bytes(), String::from("証城寺"))
            .unwrap();
        trie.add("差し昇っ".as_bytes(), String::from("差し登っ"))
            .unwrap();
        trie.add("抜け出せれ".as_bytes(), String::from("抜け出せれ"))
            .unwrap();
        trie.add("たい".as_bytes(), String::from("たい")).unwrap();
        trie.add("アオガエル".as_bytes(), String::from("アオガエル"))
            .unwrap();
        trie.add("長府浜浦".as_bytes(), String::from("長府浜浦"))
            .unwrap();
        trie.add("中佃".as_bytes(), String::from("中佃")).unwrap();
        trie.add("幻視".as_bytes(), String::from("幻視")).unwrap();
        trie.add("小船木".as_bytes(), String::from("小船木"))
            .unwrap();
        trie.add("浅黒かれ".as_bytes(), String::from("浅黒かれ"))
            .unwrap();
        trie.add("扁かろ".as_bytes(), String::from("扁かろ"))
            .unwrap();
        trie.add("咲き乱れ".as_bytes(), String::from("咲き乱れ"))
            .unwrap();

        assert_eq!(
            trie.find("張り込め".as_bytes()),
            Ok(&[String::from("張り込め")][..])
        );
        assert_eq!(
            trie.find("ニッカーボッカー".as_bytes()),
            Ok(&[String::from("ニッカーボッカー")][..])
        );
        assert_eq!(
            trie.find("証城寺".as_bytes()),
            Ok(&[String::from("証城寺")][..])
        );
        assert_eq!(
            trie.find("差し昇っ".as_bytes()),
            Ok(&[String::from("差し登っ")][..])
        );
        assert_eq!(
            trie.find("抜け出せれ".as_bytes()),
            Ok(&[String::from("抜け出せれ")][..])
        );
        assert_eq!(
            trie.find("たい".as_bytes()),
            Ok(&[String::from("たい")][..])
        );
        assert_eq!(
            trie.find("アオガエル".as_bytes()),
            Ok(&[String::from("アオガエル")][..])
        );
        assert_eq!(
            trie.find("長府浜浦".as_bytes()),
            Ok(&[String::from("長府浜浦")][..])
        );
        assert_eq!(
            trie.find("中佃".as_bytes()),
            Ok(&[String::from("中佃")][..])
        );
        assert_eq!(
            trie.find("幻視".as_bytes()),
            Ok(&[String::from("幻視")][..])
        );
        assert_eq!(
            trie.find("小船木".as_bytes()),
            Ok(&[String::from("小船木")][..])
        );
        assert_eq!(
            trie.find("浅黒かれ".as_bytes()),
            Ok(&[String::from("浅黒かれ")][..])
        );
        assert_eq!(
            trie.find("扁かろ".as_bytes()),
            Ok(&[String::from("扁かろ")][..])
        );
        assert_eq!(
            trie.find("咲き乱れ".as_bytes()),
            Ok(&[String::from("咲き乱れ")][..])
        );
    }
}

impl<T: Serialize + Clone> Trie<T> {
    fn sort_dict(src: &mut Vec<(&[u8], T)>) {
        src.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    // expect sorted
    fn get_domain<'a, 'b>(
        src: &'a [(&'b [u8], T)],
        select: usize,
        target: u8,
    ) -> &'a [(&'b [u8], T)] {
        let mut begin = None;
        let mut end = None;

        for (i, src) in src.iter().enumerate() {
            if src.0[select] == target && src.0.len() > select + 1 && begin == None {
                begin = Some(i);
            }
            if begin != None && end == None && src.0[select] != target {
                end = Some(i);
            }
        }
        match (begin, end) {
            (Some(begin), Some(end)) => &src[begin..end],
            (Some(begin), None) => &src[begin..],
            _ => unreachable!(),
        }
    }

    pub fn add_static(&mut self, src: &[(&[u8], T)], select: usize, parent_idx: usize) -> usize {
        let mut row = [Node::default(); 256];
        let mut mask = [false; 256];
        let mut update = [false; 256];
        let mut before = -1i16;
        let mut cnt = 0u8;

        for (way, cargo) in src {
            mask[way[select] as usize] = true;
            if way[select] as i16 > before {
                cnt += 1;
                before = way[select] as i16;
                row[way[select] as usize] = Node::term(parent_idx, NO_ITEM);
            }
            if way.len() <= select + 1 {
                if row[way[select] as usize].id != NO_ITEM {
                    self.storage[row[way[select] as usize].id as usize].push(cargo.clone());
                } else {
                    self.storage.push(vec![cargo.clone()]);
                    row[way[select] as usize].id = self.storage.len() - 1;
                }
            } else {
                update[way[select] as usize] = true;
            }
        }

        let base = self.reallocate_base(&mask, cnt);
        for i in 0..256 {
            if mask[i] {
                self.tree[i ^ base] = row[i];
            }
        }
        for (i, update) in update.iter().enumerate() {
            if *update {
                let idx = i ^ base;
                self.tree[idx].base =
                    self.add_static(Self::get_domain(src, select, i as u8), select + 1, idx);
            }
        }
        base
    }

    pub fn static_construction(src: &mut Vec<(&[u8], T)>) -> Trie<T> {
        Trie::sort_dict(src);
        let mut trie = Trie::default();
        trie.tree[0] = Node::root(0);
        trie.tree[0].base = trie.add_static(src, 0, 0);
        trie
    }
}
#[cfg(test)]
mod test_static_construction {
    use super::*;
    #[test]
    fn test_sort() {
        let mut dict = vec![
            (&[1, 2, 3][..], "123"),
            (&[1, 2, 3, 5][..], "1235"),
            (&[1, 2, 3, 4][..], "1234"),
            (&[1, 2, 3][..], "123"),
        ];
        Trie::sort_dict(&mut dict);
        assert_eq!(
            dict,
            vec![
                (&[1, 2, 3][..], "123"),
                (&[1, 2, 3][..], "123"),
                (&[1, 2, 3, 4][..], "1234"),
                (&[1, 2, 3, 5][..], "1235"),
            ]
        );
    }

    #[test]
    fn test_get_domain() {
        let src = [
            (&[0, 0, 0][..], "000"),
            (&[0, 1, 2][..], "012"),
            (&[0, 1, 3][..], "013"),
            (&[0, 1, 3][..], "013"),
            (&[0, 1, 3, 4][..], "0134"),
            (&[1, 0][..], "10"),
            (&[1, 1][..], "11"),
            (&[2, 1, 2][..], "212"),
        ];
        assert_eq!(
            Trie::get_domain(&src[..], 0, 0),
            &[
                (&[0, 0, 0][..], "000"),
                (&[0, 1, 2][..], "012"),
                (&[0, 1, 3][..], "013"),
                (&[0, 1, 3][..], "013"),
                (&[0, 1, 3, 4][..], "0134"),
            ][..]
        );
        assert_eq!(
            Trie::get_domain(Trie::get_domain(&src[..], 0, 0), 1, 1),
            &[
                (&[0, 1, 2][..], "012"),
                (&[0, 1, 3][..], "013"),
                (&[0, 1, 3][..], "013"),
                (&[0, 1, 3, 4][..], "0134"),
            ][..]
        );
        assert_eq!(
            Trie::get_domain(&src[..], 0, 1),
            &[(&[1, 0][..], "10"), (&[1, 1][..], "11"),][..]
        );
        assert_eq!(
            Trie::get_domain(&src[..], 0, 2),
            &[(&[2, 1, 2][..], "212"),][..]
        );
    }

    #[test]
    fn test_add_static() {
        let trie = Trie::static_construction(&mut vec![
            ("咲き乱れ".as_bytes(), String::from("咲き乱れ")),
            ("張り込め".as_bytes(), String::from("張り込め")),
            ("1".as_bytes(), String::from("1")),
            ("1月".as_bytes(), String::from("1月")),
            ("幻視".as_bytes(), String::from("幻視")),
            ("アオガエル".as_bytes(), String::from("アオガエル")),
            ("扁かろ".as_bytes(), String::from("扁かろ")),
            ("証城寺".as_bytes(), String::from("証城寺")),
            ("たい".as_bytes(), String::from("たい")),
            (
                "ニッカーボッカー".as_bytes(),
                String::from("ニッカーボッカー"),
            ),
            ("抜け出せれ".as_bytes(), String::from("抜け出せれ")),
            ("長府浜浦".as_bytes(), String::from("長府浜浦")),
            ("中佃".as_bytes(), String::from("中佃")),
            ("小船木".as_bytes(), String::from("小船木")),
            ("差し昇っ".as_bytes(), String::from("差し登っ")),
            ("浅黒かれ".as_bytes(), String::from("浅黒かれ")),
        ]);

        assert_eq!(
            trie.find("張り込め".as_bytes()),
            Ok(&[String::from("張り込め")][..])
        );
        assert_eq!(trie.find("1".as_bytes()), Ok(&[String::from("1")][..]));
        assert_eq!(trie.find("1月".as_bytes()), Ok(&[String::from("1月")][..]));
        assert_eq!(
            trie.find("ニッカーボッカー".as_bytes()),
            Ok(&[String::from("ニッカーボッカー")][..])
        );
        assert_eq!(
            trie.find("証城寺".as_bytes()),
            Ok(&[String::from("証城寺")][..])
        );
        assert_eq!(
            trie.find("差し昇っ".as_bytes()),
            Ok(&[String::from("差し登っ")][..])
        );
        assert_eq!(
            trie.find("抜け出せれ".as_bytes()),
            Ok(&[String::from("抜け出せれ")][..])
        );
        assert_eq!(
            trie.find("たい".as_bytes()),
            Ok(&[String::from("たい")][..])
        );
        assert_eq!(
            trie.find("アオガエル".as_bytes()),
            Ok(&[String::from("アオガエル")][..])
        );
        assert_eq!(
            trie.find("長府浜浦".as_bytes()),
            Ok(&[String::from("長府浜浦")][..])
        );
        assert_eq!(
            trie.find("中佃".as_bytes()),
            Ok(&[String::from("中佃")][..])
        );
        assert_eq!(
            trie.find("幻視".as_bytes()),
            Ok(&[String::from("幻視")][..])
        );
        assert_eq!(
            trie.find("小船木".as_bytes()),
            Ok(&[String::from("小船木")][..])
        );
        assert_eq!(
            trie.find("浅黒かれ".as_bytes()),
            Ok(&[String::from("浅黒かれ")][..])
        );
        assert_eq!(
            trie.find("扁かろ".as_bytes()),
            Ok(&[String::from("扁かろ")][..])
        );
        assert_eq!(
            trie.find("咲き乱れ".as_bytes()),
            Ok(&[String::from("咲き乱れ")][..])
        );
    }
}
