//Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::cmp::{Eq, PartialEq, Ordering};
use std::hash::{Hash, Hasher};

use ordered_collections::OrderedMap;
use ordered_collections::OrderedSet;

pub trait MopIfce<T: Ord + Clone> {
    fn elements(&self) -> &OrderedSet<T>;

    fn trace_strength(&self) -> f64;
    fn epitome_strength(&self) -> f64;

    fn is_trace(&self) -> bool;
    fn is_epitome(&self) -> bool;
}

pub trait MopQueries<T: Ord + Clone, R: MopIfce<T> + Ord> {
    fn complete_match(&self, query: &OrderedSet<T>) -> Option<&R>;
    fn partial_matches_rv(&self, query: &OrderedSet<T>) -> OrderedSet<&R>;
    fn partial_matches(&self, query: &OrderedSet<T>) -> OrderedSet<&R>;

    fn traces(&self) -> OrderedSet<&R>;
    fn epitomes(&self) -> OrderedSet<&R>;
}

pub trait Strength: Copy {
    fn new(incr_value: bool) -> Self;
    fn value(&self) -> f64;
    fn increase(&mut self);
    fn decrease(&mut self);
}

#[derive(Clone, Debug)]
pub struct Mop<T: Ord + Clone + Hash, S: Strength> {
    elements: OrderedSet<T>,
    children: OrderedMap<T, Self>,
    trace_strength: S,
    epitome_strength: S,
    undif_strength: S,
}

impl<T: Ord + Clone + Hash, S: Strength> PartialEq for Mop<T, S> {
    fn eq(&self, other: &Self) -> bool {
        self.elements() == other.elements()
    }
}

impl<T: Ord + Clone + Hash, S: Strength> Eq for Mop<T, S> {}

impl<T: Ord + Clone + Hash, S: Strength> PartialOrd for Mop<T, S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord + Clone + Hash, S: Strength> Ord for Mop<T, S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.elements.cmp(&other.elements)
    }
}

impl<T: Ord + Clone + Hash, S: Strength> Hash for Mop<T, S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.elements().hash(state);
    }
}

impl<T: Ord + Clone + Hash, S: Strength> Mop<T, S> {
    fn tabula_rasa() -> Self {
        Self {
            elements: OrderedSet::<T>::new(),
            children: OrderedMap::<T, Self>::new(),
            trace_strength: S::new(false),
            epitome_strength: S::new(false),
            undif_strength: S::new(false),
        }
    }
}

impl<T: Ord + Clone + Hash, S: Strength> MopIfce<T> for Mop<T, S> {
    fn elements(&self) -> &OrderedSet<T> {
        &self.elements
    }

    fn trace_strength(&self) -> f64 {
        self.trace_strength.value()
    }

    fn epitome_strength(&self) -> f64 {
        self.epitome_strength.value()
    }

    fn is_trace(&self) -> bool {
        self.trace_strength.value() > 0.0
    }

    fn is_epitome(&self) -> bool {
        self.children.len() > 0
    }
}

impl<T, S> MopQueries<T, Mop<T, S>> for Mop<T, S>
where
    T: Ord + Clone + Hash,
    S: Strength,
{
    // Algorithn 3.2
    fn complete_match(&self, query: &OrderedSet<T>) -> Option<&Self> {
        let mut p: &Self = self;
        let mut set_j = query - self.elements();
        loop {
            if let Some(j) = set_j.first() {
                if let Some(j_child) = p.children.get(j) {
                    p = j_child;
                    set_j = &set_j - p.elements();
                } else {
                    return None;
                }
            } else {
                break;
            }
        }
        Some(p)
    }

    // Algorithm 3.3
    fn partial_matches_rv(&self, query: &OrderedSet<T>) -> OrderedSet<&Self> {
        let mut matches = OrderedSet::new();
        if query.is_map_disjoint(&self.children) {
            if !self.elements().is_disjoint(query) {
                matches.insert(self);
            }
        } else {
            for j in query.iter() {
                if let Some(rdt) = self.children.get(j) {
                    for m in rdt.partial_matches_rv(query).drain() {
                        matches.insert(m);
                    }
                }
            }
        }
        matches
    }

    // Algorithm 3.4
    fn partial_matches(&self, query: &OrderedSet<T>) -> OrderedSet<&Self> {
        let mut matches = OrderedSet::new();
        if query.is_map_disjoint(&self.children) {
            if !self.elements().is_disjoint(query) {
                matches.insert(self);
            }
        } else {
            for j in query.iter() {
                if let Some(rdt) = self.children.get(j) {
                    if let Some(first) = (&(rdt.elements() - self.elements()) & query).first() {
                        if first == j {
                            for m in rdt.partial_matches_after(query, j).drain() {
                                matches.insert(m);
                            }
                        }
                    }
                }
            }
        }
        matches
    }

    // Algorithm 3.6
    fn traces(&self) -> OrderedSet<&Self> {
        let mut matches = OrderedSet::new();
        if self.is_trace() {
            matches.insert(self);
        }
        for (j, rdt) in self.children.iter() {
            if let Some(first) = (rdt.elements() - self.elements()).first() {
                if first == j {
                    for m in rdt.traces_after(j).drain() {
                        matches.insert(m);
                    }
                }
            }
        }
        matches
    }

    // Algorithm B.1
    fn epitomes(&self) -> OrderedSet<&Self> {
        let mut matches = OrderedSet::new();
        if self.is_epitome() {
            matches.insert(self);
        }
        for (j, rdt) in self.children.iter() {
            if let Some(first) = (rdt.elements() - self.elements()).first() {
                if first == j {
                    for m in rdt.epitomes_after(j).drain() {
                        matches.insert(m);
                    }
                }
            }
        }
        matches
    }
}

impl<T: Ord + Clone + Hash, S: Strength> Mop<T, S> {
    fn new_trace(elements: OrderedSet<T>) -> Self {
        Self {
            elements: elements,
            children: OrderedMap::<T, Self>::new(),
            trace_strength: S::new(true),
            epitome_strength: S::new(false),
            undif_strength: S::new(true),
        }
    }

    fn new_epitome(elements: OrderedSet<T>, strength: &S) -> Self {
        Self {
            elements: elements,
            children: OrderedMap::<T, Self>::new(),
            trace_strength: S::new(false),
            epitome_strength: strength.clone(),
            undif_strength: strength.clone(),
        }
    }

    // Algorithm 3.5
    fn partial_matches_after(&self, query: &OrderedSet<T>, k: &T) -> OrderedSet<&Self> {
        let mut matches = OrderedSet::new();
        if query.is_map_disjoint(&self.children) {
            if !self.elements.is_disjoint(query) {
                matches.insert(self);
            }
        } else {
            for j in query.iter_after(k) {
                if let Some(rdt) = self.children.get(j) {
                    if let Some(first) = (&(rdt.elements() - self.elements()) & query).first() {
                        if first == j {
                            for m in rdt.partial_matches_after(query, j).drain() {
                                matches.insert(m);
                            }
                        }
                    }
                }
            }
        }
        matches
    }

    // Algorithm 3.7
    fn traces_after(&self, k: &T) -> OrderedSet<&Self> {
        let mut matches = OrderedSet::new();
        if self.is_trace() {
            matches.insert(self);
        }
        for (j, rdt) in self.children.iter_after(k) {
            if let Some(first) = (rdt.elements() - self.elements()).first() {
                if first == j {
                    for m in rdt.traces_after(j).drain() {
                        matches.insert(m);
                    }
                }
            }
        }
        matches
    }

    // Algorithm B.2
    fn epitomes_after(&self, k: &T) -> OrderedSet<&Self> {
        let mut matches = OrderedSet::new();
        if self.is_epitome() {
            matches.insert(self);
        }
        for (j, rdt) in self.children.iter_after(k) {
            if let Some(first) = (rdt.elements() - self.elements()).first() {
                if first == j {
                    for m in rdt.traces_after(j).drain() {
                        matches.insert(m);
                    }
                }
            }
        }
        matches
    }

    fn is_compatible_with(&self, excerpt: &OrderedSet<T>) -> bool {
        self.elements.is_subset(excerpt)
    }

    fn is_recursive_compatible_with(&self, excerpt: &OrderedSet<T>) -> bool {
        if self.elements().is_subset(excerpt) {
            for key in excerpt.iter() {
                if let Some(rdt) = self.children.get(key) {
                    if !rdt.is_recursive_compatible_with(excerpt) {
                        return false;
                    }
                }
            }
        } else {
            return false;
        }
        true
    }

    // Algorithm 4.2
    fn replicate(&self) -> Self {
        let mut replica = Self {
            elements: self.elements().clone(),
            children: OrderedMap::new(),
            trace_strength: self.trace_strength.clone(),
            epitome_strength: self.epitome_strength.clone(),
            undif_strength: self.undif_strength.clone(),
        };
        for (j, rdt) in self.children.iter() {
            replica.children.insert(j.clone(), rdt.replicate());
        }
        replica
    }

    // Algorithm 4.3
    fn interpose_for_compatability(&mut self, key: &T, excerpt: &OrderedSet<T>) {
        assert!(self.is_compatible_with(excerpt));
        assert!(excerpt.contains(key));
        let key_mop = self.children.remove(key).expect("invalid key in interpose");
        assert!(!key_mop.is_compatible_with(excerpt));
        let mut new_key_mop =
            Self::new_epitome(key_mop.elements() & excerpt, &key_mop.undif_strength);
        for (k, k_rdt) in key_mop.children.iter() {
            new_key_mop.children.insert(k.clone(), k_rdt.replicate());
        }
        let mut k_iter = key_mop.elements.difference(excerpt);
        if let Some(first) = k_iter.next() {
            for k in k_iter {
                new_key_mop.children.insert(k.clone(), key_mop.replicate());
            }
            new_key_mop.children.insert(first.clone(), key_mop);
        }
        self.children.insert(key.clone(), new_key_mop);
    }

    // Algorithm 4.4
    fn reorganize_for_compatability(&mut self, excerpt: &OrderedSet<T>) {
        assert!(self.is_compatible_with(excerpt));
        // clone() needed here to break mut borrow impasse
        for j in excerpt.difference(&self.elements().clone()) {
            if let Some(j_rdt) = self.children.get(j) {
                if !excerpt.is_superset(&j_rdt.elements()) {
                    self.interpose_for_compatability(j, excerpt);
                }
            }
            if let Some(j_rdt) = self.children.get_mut(j) {
                j_rdt.reorganize_for_compatability(excerpt);
            }
        }
    }

    // Algorithm 4.5
    fn include_excerpt(&mut self, excerpt: &OrderedSet<T>) {
        if excerpt == self.elements() {
            self.trace_strength.increase();
        } else {
            let keys = excerpt - self.elements();
            for key in keys.iter() {
                if let Some(rdt) = self.children.get_mut(key) {
                    rdt.include_excerpt(excerpt);
                }
            }
            // Collect needed to break borrow impasse
            let keys: OrderedSet<T> = keys.map_difference(&self.children).collect();
            for key in keys.iter() {
                let grand_child = Self::new_trace(excerpt.clone());
                self.children.insert(key.clone(), grand_child);
            }
            self.epitome_strength.increase();
        }
        self.undif_strength.increase();
    }

    // Algorithm 4.7
    fn decrease_all_strengths(&mut self) {
        self.trace_strength.decrease();
        self.epitome_strength.decrease();
        self.undif_strength.decrease();
        for child in self.children.values_mut() {
            child.decrease_all_strengths();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SimpleStrength {
    value: f64,
}

impl Strength for SimpleStrength {
    fn new(incr_value: bool) -> Self {
        let mut strength = Self { value: 0.0 };
        if incr_value {
            strength.increase();
        }
        strength
    }

    fn value(&self) -> f64 {
        self.value
    }

    fn increase(&mut self) {
        self.value += (1.0 - self.value) * 0.05;
    }

    fn decrease(&mut self) {
        self.value *= 1.0 - 0.05;
    }
}

#[derive(Debug)]
pub struct YardstickRDT<T: Ord + Clone + Hash, S: Strength> {
    mop: Mop<T, S>,
}

impl<T: Ord + Clone + Hash, S: Strength> YardstickRDT<T, S> {
    pub fn new() -> Self {
        Self {
            mop: Mop::<T, S>::tabula_rasa(),
        }
    }

    // Algorithm 4.6
    pub fn include_excerpt(&mut self, excerpt: OrderedSet<T>) {
        self.mop.reorganize_for_compatability(&excerpt);
        assert!(self.mop.is_recursive_compatible_with(&excerpt));
        self.mop.include_excerpt(&excerpt);
    }

    pub fn include_experience(&mut self, experience: &[T]) {
        let excerpt: OrderedSet<T> = experience.iter().collect();
        self.include_excerpt(excerpt);
    }

    pub fn decrement_strengths(&mut self) {
        self.mop.decrease_all_strengths();
    }

    pub fn complete_match(&self, query: &OrderedSet<T>) -> Option<&Mop<T, S>> {
        self.mop.complete_match(query)
    }

    pub fn partial_matches_rv(&self, query: &OrderedSet<T>) -> OrderedSet<&Mop<T, S>> {
        self.mop.partial_matches_rv(query)
    }

    pub fn partial_matches(&self, query: &OrderedSet<T>) -> OrderedSet<&Mop<T, S>> {
        self.mop.partial_matches(query)
    }

    pub fn traces(&self) -> OrderedSet<&Mop<T, S>> {
        self.mop.traces()
    }

    pub fn epitomes(&self) -> OrderedSet<&Mop<T, S>> {
        self.mop.epitomes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut rdt = YardstickRDT::<&str, SimpleStrength>::new();
        let excerpt: OrderedSet<&str> = vec!["a", "b", "c", "d"].into();
        println!("{:?}", rdt.complete_match(&excerpt));
        assert!(rdt.complete_match(&excerpt).is_none());
        rdt.include_excerpt(excerpt.clone());
        assert!(rdt.complete_match(&excerpt).is_some());
        rdt.include_experience(&["a", "b", "c"]);
        rdt.include_experience(&["a", "b", "d"]);
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
        assert_eq!(rdt.partial_matches_rv(&vec!["a", "d", "e"].into()).len(), 2);
        assert_eq!(rdt.partial_matches(&vec!["a", "d", "e"].into()).len(), 2);

        assert_eq!(rdt.traces().len(), 5);
        assert_eq!(rdt.epitomes().len(), 9);
        rdt.decrement_strengths();
    }
}
