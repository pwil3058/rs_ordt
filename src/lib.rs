extern crate ordered_collections;

use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::rc::Rc;

use ordered_collections::{
    ordered_map::ord_map_iterators::{SkipAheadMapIterator, ToMap},
    ordered_set::ord_set_iterators::*,
    OrderedMap, OrderedSet,
};

pub mod strength;
#[cfg(test)]
mod yardstick;

use crate::strength::Strength;

#[derive(Clone, Debug)]
pub struct Mop<T: Ord + Debug + Clone, S: Strength> {
    elements: OrderedSet<T>,
    children_r: RefCell<OrderedMap<T, Rc<Self>>>,
    children_v: RefCell<OrderedMap<T, Rc<Self>>>,
    trace_strength: Cell<S>,
    epitome_strength: Cell<S>,
    undif_strength: Cell<S>,
}

impl<T: Ord + Debug + Clone, S: Strength> PartialEq for Mop<T, S> {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

impl<T: Ord + Debug + Clone, S: Strength> Eq for Mop<T, S> {}

impl<T: Ord + Debug + Clone, S: Strength> PartialOrd for Mop<T, S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord + Debug + Clone, S: Strength> Ord for Mop<T, S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.elements.cmp(&other.elements)
    }
}

impl<T: Ord + Clone + Debug, S: Strength> Mop<T, S> {
    pub fn elements(&self) -> &OrderedSet<T> {
        &self.elements
    }

    pub fn trace_strength(&self) -> f64 {
        self.trace_strength.get().value()
    }

    fn incr_trace_strength(&self) {
        self.trace_strength
            .set(self.trace_strength.get().incremented());
    }

    fn decr_trace_strength(&self) {
        self.trace_strength
            .set(self.trace_strength.get().decremented());
    }

    pub fn epitome_strength(&self) -> f64 {
        self.epitome_strength.get().value()
    }

    fn incr_epitome_strength(&self) {
        self.epitome_strength
            .set(self.epitome_strength.get().incremented());
    }

    fn decr_epitome_strength(&self) {
        self.epitome_strength
            .set(self.epitome_strength.get().decremented());
    }

    fn incr_undif_strength(&self) {
        self.undif_strength
            .set(self.undif_strength.get().incremented());
    }

    fn decr_undif_strength(&self) {
        self.undif_strength
            .set(self.undif_strength.get().decremented());
    }

    pub fn is_trace(&self) -> bool {
        self.trace_strength() > 0.0
    }

    pub fn is_epitome(&self) -> bool {
        self.children_r.borrow().len() > 0 || self.children_v.borrow().len() > 0
    }
}

// Support Methods
impl<'a, T: 'a + Ord + Debug + Clone, S: Strength> Mop<T, S> {
    fn tabula_rasa() -> Rc<Self> {
        Rc::new(Self {
            elements: OrderedSet::<T>::new(),
            children_r: RefCell::new(OrderedMap::<T, Rc<Self>>::new()),
            children_v: RefCell::new(OrderedMap::<T, Rc<Self>>::new()),
            trace_strength: Cell::new(S::new(false)),
            epitome_strength: Cell::new(S::new(false)),
            undif_strength: Cell::new(S::new(false)),
        })
    }

    fn new_trace(elements: OrderedSet<T>) -> Rc<Self> {
        Rc::new(Self {
            elements: elements,
            children_r: RefCell::new(OrderedMap::<T, Rc<Self>>::new()),
            children_v: RefCell::new(OrderedMap::<T, Rc<Self>>::new()),
            trace_strength: Cell::new(S::new(true)),
            epitome_strength: Cell::new(S::new(false)),
            undif_strength: Cell::new(S::new(true)),
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
            trace_strength: Cell::new(S::new(false)),
            epitome_strength: Cell::new(undif_strength.clone()),
            undif_strength: Cell::new(undif_strength.clone()),
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

    fn get_v_child(&self, key: &T) -> Option<Rc<Self>> {
        let my_children = self.children_v.borrow();
        if let Some(child) = my_children.get(key) {
            Some(Rc::clone(child))
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

    fn is_disjoint_child_indices(&self, set: &OrderedSet<T>) -> bool {
        self.children_r.borrow().keys().is_disjoint(set.iter())
            && self.children_v.borrow().keys().is_disjoint(set.iter())
    }

    fn merged_children(&self) -> RefCell<OrderedMap<T, Rc<Self>>> {
        let map = (self.children_r.borrow().iter() | self.children_v.borrow().iter()).to_map();
        RefCell::new(map)
    }

    fn is_recursive_compatible_with(&self, excerpt: &OrderedSet<T>) -> bool {
        if self.elements().is_subset(excerpt) {
            for key in excerpt.iter() {
                if let Some(mop) = self.get_r_child(key) {
                    if !mop.is_recursive_compatible_with(excerpt) {
                        return false;
                    }
                } else if let Some(mop) = self.get_v_child(key) {
                    if !mop.is_recursive_compatible_with(excerpt) {
                        return false;
                    }
                }
            }
        } else {
            return false;
        }
        true
    }
}

// Main algorithms
impl<T: Ord + Debug + Clone, S: Strength> Mop<T, S> {
    fn algorithm_6_2_interpose(&self, j: &T, excerpt: &OrderedSet<T>) {
        let (j_mop, j_mop_indices) = self.get_r_child_and_indices(j).unwrap();
        let m = Self::new_epitome(
            j_mop.elements.intersection(excerpt).to_set(),
            j_mop.merged_children(),
            &j_mop.undif_strength.get(),
        );
        m.insert_r_child(j_mop.elements.difference(&m.elements), &j_mop);
        self.insert_r_child(j_mop_indices.iter(), &m);
    }

    fn algorithm_6_3_split(&self, j: &T, excerpt: &OrderedSet<T>) {
        let (j_mop, j_mop_indices) = self.get_r_child_and_indices(j).unwrap();
        let m = Self::new_epitome(
            j_mop.elements.intersection(excerpt).to_set(),
            j_mop.merged_children(),
            &j_mop.undif_strength.get(),
        );
        m.insert_v_child(j_mop.elements.difference(&m.elements), &j_mop);
        self.insert_r_child(excerpt.intersection(&j_mop_indices), &m);
    }

    fn algorithm_6_4_reorganize(
        &self,
        excerpt: &OrderedSet<T>,
        base_mop: &Rc<Self>,
        big_u: &mut OrderedSet<(Rc<Self>, Rc<Self>)>,
    ) {
        let mut big_a = (excerpt.iter() & self.children_r.borrow().keys()).to_set();
        while let Some(j) = big_a.first() {
            let (j_mop, big_i_to) = self.get_r_child_and_indices(j).unwrap();
            let p = &j_mop;
            if !excerpt.is_superset(&big_i_to) {
                self.algorithm_6_3_split(j, excerpt);
                let j_mop = self.get_r_child(j).unwrap();
                j_mop.algorithm_6_9_fix_v_links(big_u);
                base_mop.algorithm_6_10_fix_v_links(&(Rc::clone(p), Rc::clone(&j_mop)));
                big_u.insert((Rc::clone(p), j_mop));
            } else if !excerpt.is_superset(&j_mop.elements.difference(&self.elements).to_set()) {
                self.algorithm_6_2_interpose(j, excerpt);
                let j_mop = self.get_r_child(j).unwrap();
                j_mop.algorithm_6_9_fix_v_links(big_u);
                base_mop.algorithm_6_10_fix_v_links(&(Rc::clone(p), Rc::clone(&j_mop)));
                big_u.insert((Rc::clone(p), j_mop));
            } else {
                j_mop.algorithm_6_4_reorganize(excerpt, base_mop, big_u);
            }

            big_a = big_a - big_i_to;
        }
    }

    fn algorithm_6_6_interpose(&self, j: &T, excerpt: &OrderedSet<T>) {
        let (j_mop_v, j_mop_v_indices) = self.get_v_child_and_indices(j).unwrap();
        let m = Self::new_epitome(
            j_mop_v.elements.intersection(excerpt).to_set(),
            j_mop_v.merged_children(),
            &j_mop_v.undif_strength.get(),
        );
        m.insert_v_child(j_mop_v.elements.difference(&m.elements), &j_mop_v);
        self.insert_r_child(excerpt.intersection(&j_mop_v_indices), &m);
        self.delete_v_children(excerpt.intersection(&j_mop_v_indices));
    }

    fn algorithm_6_7_reorganize(
        &self,
        excerpt: &OrderedSet<T>,
        base_mop: &Rc<Self>,
        big_u: &mut OrderedSet<(Rc<Self>, Rc<Self>)>,
    ) {
        let mut big_a_v = (excerpt.iter() & self.children_v.borrow().keys()).to_set();
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
            }
        }
        let mut big_a = (excerpt.iter() & self.children_r.borrow().keys()).to_set();
        while let Some(j) = big_a.first() {
            let (j_mop, j_mop_indices) = self.get_r_child_and_indices(j).unwrap();
            j_mop.algorithm_6_7_reorganize(excerpt, base_mop, big_u);
            big_a = big_a - j_mop_indices;
        }
    }

    fn algorithm_6_9_fix_v_links(&self, big_u: &OrderedSet<(Rc<Self>, Rc<Self>)>) {
        for (m1, m2) in big_u.iter() {
            if m2.elements.is_superset(&self.elements) {
                for k in m2.elements.iter() {
                    if let Some(k_mop_v) = self.get_v_child(k) {
                        if k_mop_v == *m1 {
                            self.children_v
                                .borrow_mut()
                                .insert(k.clone(), Rc::clone(m2));
                        }
                    }
                }
            }
        }
    }

    fn algorithm_6_10_fix_v_links(&self, mops: &(Rc<Self>, Rc<Self>)) {
        if mops.1.elements.is_superset(&self.elements) {
            let big_c_r = mops.1.elements.difference(&self.elements).to_set();
            for k in big_c_r.iter() {
                if let Some(mop_k_v) = self.get_v_child(k) {
                    if mop_k_v == mops.0 {
                        self.children_v
                            .borrow_mut()
                            .insert(k.clone(), Rc::clone(&mops.1));
                    }
                }
            }
            let mut big_a = (big_c_r.iter() & self.children_r.borrow().keys()).to_set();
            while let Some(j) = big_a.first() {
                let (j_mop, j_mop_indices) = self.get_r_child_and_indices(j).unwrap();
                j_mop.algorithm_6_10_fix_v_links(mops);
                big_a = big_a - j_mop_indices;
            }
        }
    }

    fn algorithm_6_12_decr_strengths(&self) {
        self.decr_trace_strength();
        self.decr_epitome_strength();
        self.decr_undif_strength();
        let mut big_a = self.children_r.borrow().keys().to_set();
        while let Some(j) = big_a.first() {
            let (child, indices) = self.get_r_child_and_indices(j).unwrap();
            child.algorithm_6_12_decr_strengths();
            big_a = big_a - indices;
        }
    }
}

trait Engine<T: Ord + Debug + Clone, S: Strength> {
    fn algorithm_6_11_absorb(&self, excerpt: &OrderedSet<T>, new_trace: &mut Option<Rc<Mop<T, S>>>);
    fn algorithm_6_13_complete_match(&self, query: &OrderedSet<T>) -> Option<Rc<Mop<T, S>>>;
    fn algorithm_6_14_patrial_match(&self, query: &OrderedSet<T>) -> OrderedSet<Rc<Mop<T, S>>>;
    fn algorithm_6_15_patrial_match_after(
        &self,
        query: &OrderedSet<T>,
        k: &T,
    ) -> OrderedSet<Rc<Mop<T, S>>>;
    fn algorithm_b8_mod_traces_after(&self, after: &T) -> OrderedSet<Rc<Mop<T, S>>>;
    fn algorithm_b10_mod_epitomes_after(&self, k: &T) -> OrderedSet<Rc<Mop<T, S>>>;
}

impl<T: Ord + Debug + Clone, S: Strength> Engine<T, S> for Rc<Mop<T, S>> {
    fn algorithm_6_11_absorb(
        &self,
        excerpt: &OrderedSet<T>,
        new_trace: &mut Option<Rc<Mop<T, S>>>,
    ) {
        let big_x_u = excerpt - &self.elements;
        if big_x_u.len() == 0 {
            *new_trace = Some(Rc::clone(self));
            self.incr_trace_strength();
        } else {
            let mut big_a = (big_x_u.iter() & self.children_r.borrow().keys()).to_set();
            while let Some(j) = big_a.first() {
                let (j_mop, j_mop_indices) = self.get_r_child_and_indices(j).unwrap();
                j_mop.algorithm_6_11_absorb(excerpt, new_trace);
                big_a = big_a - j_mop_indices;
            }
            let temp_set = ((big_x_u.iter() - self.children_r.borrow().keys())
                - self.children_v.borrow().keys())
            .to_set();
            if temp_set.len() > 0 {
                if let Some(p) = new_trace {
                    self.insert_v_child(temp_set.iter(), p);
                } else {
                    let p = Mop::<T, S>::new_trace(excerpt.clone());
                    self.insert_r_child(temp_set.iter(), &p);
                    *new_trace = Some(p);
                }
            }
            self.incr_epitome_strength();
        }
        self.incr_undif_strength();
    }

    fn algorithm_6_13_complete_match(&self, query: &OrderedSet<T>) -> Option<Rc<Mop<T, S>>> {
        let mut p = Rc::clone(self);
        let mut big_j = query - &self.elements;
        while let Some(j) = big_j.first() {
            if let Some(j_mop) = p.get_r_child(j) {
                p = j_mop;
                big_j = big_j.difference(&p.elements).to_set();
            } else if let Some(j_mop) = p.get_v_child(j) {
                p = j_mop;
                big_j = big_j.difference(&p.elements).to_set();
            } else {
                return None;
            }
        }
        Some(p)
    }

    fn algorithm_6_14_patrial_match(&self, query: &OrderedSet<T>) -> OrderedSet<Rc<Mop<T, S>>> {
        let mut big_s = OrderedSet::default();
        if self.is_disjoint_child_indices(query) {
            if !query.is_disjoint(self.elements()) {
                big_s.insert(Rc::clone(self));
            }
        } else {
            for j in query.difference(self.elements()) {
                if let Some(j_mop) = self.get_r_child(j) {
                    if j == j_mop
                        .elements()
                        .difference(self.elements())
                        .intersection(query.iter())
                        .next()
                        .unwrap()
                    {
                        big_s = big_s | j_mop.algorithm_6_15_patrial_match_after(query, j);
                    }
                } else if let Some(j_mop) = self.get_v_child(j) {
                    if j == j_mop
                        .elements()
                        .difference(self.elements())
                        .intersection(query.iter())
                        .next()
                        .unwrap()
                    {
                        big_s = big_s | j_mop.algorithm_6_15_patrial_match_after(query, j);
                    }
                }
            }
        }
        big_s
    }

    fn algorithm_6_15_patrial_match_after(
        &self,
        query: &OrderedSet<T>,
        k: &T,
    ) -> OrderedSet<Rc<Mop<T, S>>> {
        let mut big_s = OrderedSet::default();
        if self.is_disjoint_child_indices(query) {
            if !query.is_disjoint(self.elements()) {
                big_s.insert(Rc::clone(self));
            }
        } else {
            for j in query.difference(self.elements()).advance_past(k) {
                if let Some(j_mop) = self.get_r_child(j) {
                    if j == j_mop
                        .elements()
                        .difference(self.elements())
                        .intersection(query.iter())
                        .next()
                        .unwrap()
                    {
                        big_s = big_s | j_mop.algorithm_6_15_patrial_match_after(query, j);
                    }
                } else if let Some(j_mop) = self.get_v_child(j) {
                    if j == j_mop
                        .elements()
                        .difference(self.elements())
                        .intersection(query.iter())
                        .next()
                        .unwrap()
                    {
                        big_s = big_s | j_mop.algorithm_6_15_patrial_match_after(query, j);
                    }
                }
            }
        }
        big_s
    }

    fn algorithm_b8_mod_traces_after(&self, k: &T) -> OrderedSet<Rc<Mop<T, S>>> {
        let mut big_s = OrderedSet::default();
        if self.is_trace() {
            big_s.insert(Rc::clone(self));
        }
        for (j, j_mop) in
            (self.children_r.borrow().iter() | self.children_v.borrow().iter()).advance_past_key(k)
        {
            if j == j_mop.elements().difference(self.elements()).next().unwrap() {
                big_s = big_s | j_mop.algorithm_b8_mod_traces_after(j);
            }
        }
        big_s
    }

    fn algorithm_b10_mod_epitomes_after(&self, k: &T) -> OrderedSet<Rc<Mop<T, S>>> {
        let mut big_s = OrderedSet::default();
        if self.is_epitome() {
            big_s.insert(Rc::clone(self));
        }
        for (j, j_mop) in
            (self.children_r.borrow().iter() | self.children_v.borrow().iter()).advance_past_key(k)
        {
            if j == j_mop.elements().difference(self.elements()).next().unwrap() {
                big_s = big_s | j_mop.algorithm_b10_mod_epitomes_after(j);
            }
        }
        big_s
    }
}

pub trait Public<T: Ord + Debug + Clone, S: Strength> {
    fn traces(&self) -> OrderedSet<Rc<Mop<T, S>>>;
    fn epitomes(&self) -> OrderedSet<Rc<Mop<T, S>>>;
}

impl<T: Ord + Debug + Clone, S: Strength> Public<T, S> for Rc<Mop<T, S>> {
    fn traces(&self) -> OrderedSet<Rc<Mop<T, S>>> {
        let mut big_s = OrderedSet::default();
        if self.is_trace() {
            big_s.insert(Rc::clone(self));
        }
        for (j, j_mop) in self.children_r.borrow().iter() | self.children_v.borrow().iter() {
            if j == j_mop.elements().difference(self.elements()).next().unwrap() {
                big_s = big_s | j_mop.algorithm_b8_mod_traces_after(j);
            }
        }
        big_s
    }

    fn epitomes(&self) -> OrderedSet<Rc<Mop<T, S>>> {
        let mut big_s = OrderedSet::default();
        if self.is_epitome() {
            big_s.insert(Rc::clone(self));
        }
        for (j, j_mop) in self.children_r.borrow().iter() | self.children_v.borrow().iter() {
            if j == j_mop.elements().difference(self.elements()).next().unwrap() {
                big_s = big_s | j_mop.algorithm_b10_mod_epitomes_after(j);
            }
        }
        big_s
    }
}

#[derive(Debug)]
pub struct RedundantDiscriminationTree<T: Ord + Debug + Clone, S: Strength> {
    mop: Rc<Mop<T, S>>,
}

impl<T: Ord + Debug + Clone, S: Strength> RedundantDiscriminationTree<T, S> {
    pub fn new() -> Self {
        Self {
            mop: Mop::<T, S>::tabula_rasa(),
        }
    }

    // Algorithm 6.1
    pub fn include_excerpt(&mut self, excerpt: OrderedSet<T>) {
        let mut big_u = OrderedSet::<(Rc<Mop<T, S>>, Rc<Mop<T, S>>)>::new();
        let mut new_trace: Option<Rc<Mop<T, S>>> = None;
        self.mop
            .algorithm_6_4_reorganize(&excerpt, &self.mop, &mut big_u);
        self.mop
            .algorithm_6_7_reorganize(&excerpt, &self.mop, &mut big_u);
        assert!(self.mop.is_recursive_compatible_with(&excerpt));
        self.mop.algorithm_6_11_absorb(&excerpt, &mut new_trace);
        assert!(self.mop.verify_tree());
    }

    pub fn include_experience(&mut self, experience: &[T]) {
        let excerpt: OrderedSet<T> = experience.iter().collect();
        self.include_excerpt(excerpt);
    }

    pub fn decrement_strengths(&mut self) {
        self.mop.algorithm_6_12_decr_strengths();
    }

    pub fn complete_match(&self, query: &OrderedSet<T>) -> Option<Rc<Mop<T, S>>> {
        self.mop.algorithm_6_13_complete_match(&query)
    }

    pub fn partial_matches(&self, query: &OrderedSet<T>) -> OrderedSet<Rc<Mop<T, S>>> {
        self.mop.algorithm_6_14_patrial_match(query)
    }

    pub fn traces(&self) -> OrderedSet<Rc<Mop<T, S>>> {
        self.mop.traces()
    }

    pub fn epitomes(&self) -> OrderedSet<Rc<Mop<T, S>>> {
        self.mop.epitomes()
    }
}

// SIMPLE STRENGTH
//
// #[derive(Debug, Clone)]
// pub struct SimpleStrength {
//     value: Cell<f64>,
// }
//
// impl Strength for SimpleStrength {
//     fn new(incr_value: bool) -> Self {
//         let strength = Self {
//             value: Cell::new(0.0),
//         };
//         if incr_value {
//             strength.increase();
//         }
//         strength
//     }
//
//     fn value(&self) -> f64 {
//         self.value.get()
//     }
//
//     fn increase(&self) {
//         let old_value = self.value.get();
//         self.value.set(old_value + (1.0 - old_value) * 0.05);
//     }
//
//     fn decrease(&self) {
//         let old_value = self.value.get();
//         self.value.set(old_value * (1.0 - 0.05));
//     }
// }

// Debug Helpers
fn format_set<T: Ord + Debug>(set: &OrderedSet<T>) -> String {
    let v: Vec<&T> = set.iter().collect();
    format!("{:?}", v)
}

impl<T: Ord + Debug + Clone, S: Strength> Mop<T, S> {
    fn format_mop_short(&self) -> String {
        let big_c: Vec<&T> = self.elements.iter().collect();
        let childen_r = self.children_r.borrow();
        let big_i_r: Vec<&T> = childen_r.keys().collect();
        let childen_v = self.children_v.borrow();
        let big_i_v: Vec<&T> = childen_v.keys().collect();
        format!("C: {:?} I_r: {:?} I_v: {:?}", big_c, big_i_r, big_i_v)
    }

    fn format_mop(&self) -> String {
        if self.children_r.borrow().len() == 0 && self.children_v.borrow().len() == 0 {
            return format!("C: {} {{}}", format_set(&self.elements));
        }
        let mut fstr = format!("C: {} {{\n", format_set(&self.elements));
        let mut big_a = self.children_r.borrow().keys().to_set();
        while let Some(j) = big_a.first() {
            let (j_mop, j_mop_indices) = self.get_r_child_and_indices(j).unwrap();
            let tstr = format!(
                "\tR: {} -> {}\n",
                format_set(&j_mop_indices),
                j_mop.format_mop_short()
            );
            fstr.push_str(&tstr);
            big_a = big_a - j_mop_indices;
        }
        let mut big_a = self.children_v.borrow().keys().to_set();
        while let Some(j) = big_a.first() {
            let (j_mop, j_mop_indices) = self.get_v_child_and_indices(j).unwrap();
            let tstr = format!(
                "\tV: {} -> {}\n",
                format_set(&j_mop_indices),
                j_mop.format_mop_short()
            );
            fstr.push_str(&tstr);
            big_a = big_a - j_mop_indices;
        }
        fstr.push_str("}");
        fstr
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
            println!("child indices overlap {}", self.format_mop());
            result = false;
        };
        result
    }

    fn verify_tree(&self) -> bool {
        let mut result = self.verify_mop();
        let mut big_j = self.children_r.borrow().keys().to_set();
        while let Some(j) = big_j.first() {
            let (j_mop, j_mop_indices) = self.get_r_child_and_indices(j).unwrap();
            big_j = big_j.difference(&j_mop_indices).to_set();
            result = result && j_mop.verify_tree();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strength::*;

    #[test]
    fn it_works() {
        let mut rdt = RedundantDiscriminationTree::<&str, SimpleStrength>::new();
        let excerpt: OrderedSet<&str> = vec!["a", "b", "c", "d"].into();
        assert!(rdt.complete_match(&excerpt).is_none());
        rdt.include_excerpt(excerpt.clone());
        assert!(rdt.complete_match(&excerpt).is_some());
        rdt.include_experience(&["a", "b", "c"]);
        assert!(rdt.complete_match(&vec!["a", "b", "c"].into()).is_some());
        rdt.include_experience(&["a", "b", "d"]);
        assert!(rdt.complete_match(&vec!["a", "b", "d"].into()).is_some());
        rdt.include_experience(&["a", "d"]);
        assert!(rdt.complete_match(&vec!["a", "b", "c"].into()).is_some());
        assert!(rdt.complete_match(&vec!["a", "b", "d"].into()).is_some());
        assert!(rdt.complete_match(&vec!["a", "d"].into()).is_some());
        assert!(rdt.complete_match(&vec!["a", "b"].into()).is_some());
        assert!(rdt.complete_match(&vec!["d", "b"].into()).is_some());

        assert_eq!(
            rdt.complete_match(&vec!["a", "b", "c"].into())
                .unwrap()
                .elements(),
            &OrderedSet::<&str>::from(vec!["a", "b", "c"])
        );
        assert_eq!(
            rdt.complete_match(&vec!["a", "b", "d"].into())
                .unwrap()
                .elements(),
            &OrderedSet::<&str>::from(vec!["a", "b", "d"])
        );
        assert_eq!(
            rdt.complete_match(&vec!["a", "d"].into())
                .unwrap()
                .elements(),
            &OrderedSet::<&str>::from(vec!["a", "d"])
        );
        assert_eq!(
            rdt.complete_match(&vec!["a", "b"].into())
                .unwrap()
                .elements(),
            &OrderedSet::<&str>::from(vec!["a", "b"])
        );
        assert_eq!(
            rdt.complete_match(&vec!["d", "b"].into())
                .unwrap()
                .elements(),
            &OrderedSet::<&str>::from(vec!["a", "b", "d"])
        );
        assert_eq!(
            rdt.complete_match(&vec!["d", "b", "a", "c"].into())
                .unwrap()
                .elements(),
            &OrderedSet::<&str>::from(vec!["a", "b", "c", "d"])
        );

        assert_eq!(rdt.traces().len(), 4);
        assert_eq!(rdt.epitomes().len(), 6);

        rdt.include_experience(&vec!["e", "b", "d"]);
        assert!(rdt.complete_match(&vec!["a", "e"].into()).is_none());
        assert!(
            rdt.complete_match(&vec!["d", "b", "e"].into())
                .unwrap()
                .elements()
                .len()
                == 3
        );
        assert_eq!(rdt.partial_matches(&vec!["a", "d", "e"].into()).len(), 2);

        assert_eq!(rdt.traces().len(), 5);
        assert_eq!(rdt.epitomes().len(), 9);
        rdt.decrement_strengths();
    }
}
