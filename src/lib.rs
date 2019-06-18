// Copyright 2019 Peter Williams <pwil3058@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate ordered_collections;

use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use ordered_collections::{OrderedMap, OrderedSet};
use ordered_collections::ordered_iterators::*;
use ordered_collections::iter_ops::*;

mod yardstick;

pub trait Strength: Clone {
    fn new(incr_value: bool) -> Self;
    fn value(&self) -> f64;
    fn increase(&self);
    fn decrease(&self);
}

#[derive(Clone, Debug)]
pub struct Mop<T: Ord + Debug + Clone + Hash, S: Strength> {
    elements: OrderedSet<T>,
    children_r: RefCell<OrderedMap<T, Rc<Self>>>,
    children_v: RefCell<OrderedMap<T, Rc<Self>>>,
    trace_strength: S,
    epitome_strength: S,
    undif_strength: S,
}

impl<T: Ord + Debug + Clone + Hash, S: Strength> PartialEq for Mop<T, S> {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

impl<T: Ord + Debug + Clone + Hash, S: Strength> Eq for Mop<T, S> {}

impl<T: Ord + Debug + Clone + Hash, S: Strength> Hash for Mop<T, S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.elements.hash(state);
    }
}

// Support Methods
impl<'a, T: 'a + Ord + Debug + Clone + Hash, S: Strength> Mop<T, S> {
    fn tabula_rasa() -> Rc<Self> {
        Rc::new(Self {
            elements: OrderedSet::<T>::new(),
            children_r: RefCell::new(OrderedMap::<T, Rc<Self>>::new()),
            children_v: RefCell::new(OrderedMap::<T, Rc<Self>>::new()),
            trace_strength: S::new(false),
            epitome_strength: S::new(false),
            undif_strength: S::new(false),
        })
    }

    fn new_trace(elements: OrderedSet<T>) -> Rc<Self> {
        Rc::new(Self {
            elements: elements,
            children_r: RefCell::new(OrderedMap::<T, Rc<Self>>::new()),
            children_v: RefCell::new(OrderedMap::<T, Rc<Self>>::new()),
            trace_strength: S::new(true),
            epitome_strength: S::new(false),
            undif_strength: S::new(true),
        })
    }

    fn new_epitome(
        elements: OrderedSet<T>,
        children_v: RefCell<OrderedMap<T, Rc<Self>>>,
        undif_strength: &S,
    ) -> Rc<Self> {
        Rc::new(Self {
            elements: elements,
            children_r: RefCell::new(OrderedMap::<T, Rc<Self>>::new()),
            children_v: children_v,
            trace_strength: S::new(false),
            epitome_strength: undif_strength.clone(),
            undif_strength: undif_strength.clone(),
        })
    }

    fn insert_r_child<I: Iterator<Item = &'a T>>(&self, iter: I, child: &Rc<Self>) {
        let mut children_r = self.children_r.borrow_mut();
        for i in iter {
            children_r.insert(i.clone(), Rc::clone(child));
        }
    }

    fn insert_v_child<I: Iterator<Item = &'a T>>(&self, iter: I, child: &Rc<Self>) {
        let mut children_v = self.children_v.borrow_mut();
        for i in iter {
            children_v.insert(i.clone(), Rc::clone(child));
        }
    }

    fn delete_v_children<I: Iterator<Item = &'a T>>(&self, iter: I) {
        let mut children_v = self.children_v.borrow_mut();
        for i in iter {
            children_v.remove(i);
        }
    }

    fn get_r_child(&self, key: &T) -> Option<Rc<Self>> {
        let my_children = self.children_r.borrow();
        if let Some(child) = my_children.get(key) {
            Some(Rc::clone(child))
        } else {
            None
        }
    }

    // See Algorithm 6.5
    fn get_r_child_and_indices(&self, key: &T) -> Option<(Rc<Self>, OrderedSet<T>)> {
        let my_children = self.children_r.borrow();
        if let Some(child) = my_children.get(key) {
            let mut indices = OrderedSet::<T>::new();
            for i in child
                .elements
                .difference(&self.elements)
                .difference(self.children_v.borrow().keys())
            {
                if let Some(rdt_i) = my_children.get(i) {
                    if rdt_i == child {
                        indices.insert(i.clone());
                    }
                }
            }
            Some((Rc::clone(child), indices))
        } else {
            None
        }
    }

    // See Algorithm 6.8
    fn get_v_child_and_indices(&self, key: &T) -> Option<(Rc<Self>, OrderedSet<T>)> {
        let my_children = self.children_v.borrow();
        if let Some(child) = my_children.get(key) {
            let mut indices = OrderedSet::<T>::new();
            for i in child
                .elements
                .difference(&self.elements)
                .difference(self.children_r.borrow().keys())
            {
                if let Some(rdt_i) = my_children.get(i) {
                    if rdt_i == child {
                        indices.insert(i.clone());
                    }
                }
            }
            Some((Rc::clone(child), indices))
        } else {
            None
        }
    }

    fn merge_children(&self) -> RefCell<OrderedMap<T, Rc<Self>>> {
        let mut map = self
            .children_r
            .borrow()
            .merge(&self.children_v.borrow())
            .to_map();
        RefCell::new(map)
    }
}

// Main algorithms
impl<T: Ord + Debug + Clone + Hash, S: Strength> Mop<T, S> {
    fn algorithm_6_2_interpose(&self, j: &T, excerpt: &OrderedSet<T>) {
        let (j_mop, j_mop_indices) = self.get_r_child_and_indices(j).unwrap();
        let m = Self::new_epitome(
            j_mop.elements.intersection(excerpt).to_set(),
            j_mop.merge_children(),
            &j_mop.undif_strength,
        );
        m.insert_r_child(j_mop.elements.difference(&m.elements), &j_mop);
        assert!(m.verify_mop());
        self.insert_r_child(m.elements.difference(&self.elements), &m);
        assert!(self.verify_mop());
    }

    fn algorithm_6_3_split(&self, j: &T, excerpt: &OrderedSet<T>) {
        let (j_mop, j_mop_indices) = self.get_r_child_and_indices(j).unwrap();
        let m = Self::new_epitome(
            j_mop.elements.intersection(excerpt).to_set(),
            j_mop.merge_children(),
            &j_mop.undif_strength,
        );
        m.insert_v_child(j_mop.elements.difference(&m.elements), &j_mop);
        assert!(m.verify_mop());
        self.insert_r_child(excerpt.intersection(&j_mop_indices), &m);
        assert!(self.verify_mop());
    }

    fn algorithm_6_4_reorganize(&self,
        excerpt: &OrderedSet<T>,
        base_mop: &Rc<Self>,
        big_u: &mut HashSet<(Rc<Self>, Rc<Self>)>,
    ) {
        let mut big_a = excerpt.map_intersection(&self.children_r.borrow()).to_set();
        while let Some(j) = big_a.first() {
            let (j_mop, big_i_to) = self.get_r_child_and_indices(j).unwrap();
            let p = &j_mop;
            if !excerpt.is_superset(&big_i_to) {
                self.algorithm_6_3_split(j, excerpt);
                let j_mop = self.get_r_child(j).unwrap();
                j_mop.algorithm_6_9_fix_v_links(big_u);
                base_mop.algorithm_6_10_fix_v_links(&(Rc::clone(p), Rc::clone(&j_mop)));
                assert!(j_mop.verify_mop());
                assert!(base_mop.verify_mop());
                big_u.insert((Rc::clone(p), j_mop));
            } else if !excerpt.is_superset(&j_mop.elements) {
                self.algorithm_6_2_interpose(j, excerpt);
                let j_mop = self.get_r_child(j).unwrap();
                j_mop.algorithm_6_9_fix_v_links(big_u);
                base_mop.algorithm_6_10_fix_v_links(&(Rc::clone(p), Rc::clone(&j_mop)));
                assert!(j_mop.verify_mop());
                assert!(base_mop.verify_mop());
                big_u.insert((Rc::clone(p), j_mop));
            } else {
                j_mop.algorithm_6_4_reorganize(excerpt, base_mop, big_u);
            }

            big_a = big_a - big_i_to;
        }
        assert!(self.verify_mop());
    }

    fn algorithm_6_6_interpose(&self, j: &T, excerpt: &OrderedSet<T>) {
        let (j_mop, j_mop_indices) = self.get_v_child_and_indices(j).unwrap();
        let m = Self::new_epitome(
            j_mop.elements.intersection(excerpt).to_set(),
            j_mop.merge_children(),
            &j_mop.undif_strength,
        );
        m.insert_v_child(j_mop.elements.difference(&m.elements), &j_mop);
        assert!(m.verify_mop());
        self.insert_r_child(excerpt.intersection(&j_mop_indices), &m);
        self.delete_v_children(excerpt.intersection(&j_mop_indices));
        assert!(self.verify_mop());
    }

    fn algorithm_6_7_reorganize(
        &self,
        excerpt: &OrderedSet<T>,
        base_mop: &Rc<Self>,
        big_u: &mut HashSet<(Rc<Self>, Rc<Self>)>,
    ) {
        let mut big_a_v = excerpt.map_intersection(&self.children_v.borrow()).to_set();
        while let Some(j) = big_a_v.first() {
            let (j_mop_v, j_mop_v_indices) = self.get_v_child_and_indices(j).unwrap();
            if excerpt.is_superset(&j_mop_v.elements) {
                big_a_v = big_a_v - j_mop_v_indices;
            } else {
                self.algorithm_6_6_interpose(j, excerpt);
                let (j_mop, j_mop_indices) = self.get_r_child_and_indices(j).unwrap();
                j_mop.algorithm_6_9_fix_v_links(big_u);
                base_mop.algorithm_6_10_fix_v_links(&(Rc::clone(&j_mop_v), Rc::clone(&j_mop)));
                big_u.insert((Rc::clone(&j_mop_v), Rc::clone(&j_mop)));
                big_a_v = big_a_v - j_mop_indices;
                assert!(j_mop.verify_mop());
            }
        }
        let mut big_a = excerpt.map_intersection(&self.children_r.borrow()).to_set();
        while let Some(j) = big_a.first() {
            let (j_mop, j_mop_indices) = self.get_r_child_and_indices(j).unwrap();
            j_mop.algorithm_6_7_reorganize(excerpt, base_mop, big_u);
            assert!(j_mop.verify_mop());
            big_a = big_a - j_mop_indices;
        }
    }

    fn algorithm_6_9_fix_v_links(&self, big_u: &HashSet<(Rc<Self>, Rc<Self>)>) {
    }

    fn algorithm_6_10_fix_v_links(&self, big_u: &(Rc<Self>, Rc<Self>)) {
    }

    fn algorithm_6_11_absorb(&self, excerpt: &OrderedSet<T>, new_trace: &mut Option<Rc<Self>>) {
    }

    fn algorithm_6_12_decr_strengths(&self) {
        self.trace_strength.decrease();
        self.epitome_strength.decrease();
        self.undif_strength.decrease();
        let mut big_a = self.children_r.borrow().keys().to_set();
        while let Some(j) = big_a.first() {
            let (child, indices) = self.get_r_child_and_indices(j).unwrap();
            child.algorithm_6_12_decr_strengths();
            big_a = big_a - indices;
        }
    }
}


#[derive(Debug)]
pub struct RedundantDiscriminationTree<T: Ord + Debug + Clone + Hash, S: Strength> {
    mop: Rc<Mop<T, S>>,
}

impl<T: Ord + Debug + Clone + Hash, S: Strength> RedundantDiscriminationTree<T, S> {
    pub fn new() -> Self {
        Self {
            mop: Mop::<T, S>::tabula_rasa(),
        }
    }

    // Algorithm 6.1
    pub fn include_excerpt(&mut self, excerpt: OrderedSet<T>) {
        let mut big_u = HashSet::<(Rc<Mop<T, S>>, Rc<Mop<T, S>>)>::new();
        let mut new_trace: Option<Rc<Mop<T, S>>> = None;
        self.mop
            .algorithm_6_4_reorganize(&excerpt, &self.mop, &mut big_u);
        self.mop
            .algorithm_6_7_reorganize(&excerpt, &self.mop, &mut big_u);
        //assert!(self.mop.is_recursive_compatible_with(&excerpt));
        self.mop.algorithm_6_11_absorb(&excerpt, &mut new_trace);
    }

    pub fn include_experience(&mut self, experience: &[T]) {
        let excerpt: OrderedSet<T> = experience.iter().collect();
        self.include_excerpt(excerpt);
    }

    pub fn decrement_strengths(&mut self) {
        self.mop.algorithm_6_12_decr_strengths();
    }
}

// Debug Helpers
fn format_set<T: Ord + Debug>(set: &OrderedSet<T>) -> String {
    let v: Vec<&T> = set.iter().collect();
    format!("{:?}", v)
}

impl<T: Ord + Debug + Clone + Hash, S: Strength> Mop<T, S> {
    fn format_mop(&self) -> String {
        let big_c: Vec<&T> = self.elements.iter().collect();
        let childen_r = self.children_r.borrow();
        let big_i_r: Vec<&T> = childen_r.keys().collect();
        let childen_v = self.children_v.borrow();
        let big_i_v: Vec<&T> = childen_v.keys().collect();
        format!("C: {:?} I_r: {:?} I_v: {:?}", big_c, big_i_r, big_i_v)
    }

    fn verify_mop(&self) -> bool {
        let mut result = true;
        let r_indices = self.children_r.borrow().keys().to_set();
        let v_indices = self.children_v.borrow().keys().to_set();
        if !r_indices.is_disjoint(&self.elements) {
            println!(
                "real indices overlap C {} <> {}",
                format_set(&r_indices),
                format_set(&self.elements)
            );
            result = false;
        };
        if !v_indices.is_disjoint(&self.elements) {
            println!(
                "virt indices overlap C {} <> {}",
                format_set(&v_indices),
                format_set(&self.elements)
            );
            result = false;
        };
        if !r_indices.is_disjoint(&v_indices) {
            println!(
                "indices overlap {} <> {}",
                format_set(&r_indices),
                format_set(&v_indices)
            );
            result = false;
        };
        result
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
