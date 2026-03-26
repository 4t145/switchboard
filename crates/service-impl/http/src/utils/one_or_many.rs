use std::iter::once;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    Many(Vec<T>),
    One(T),
}

impl<T: Default> Default for OneOrMany<T> {
    fn default() -> Self {
        OneOrMany::One(T::default())
    }
}

pub enum IntoIter<T> {
    Many(std::vec::IntoIter<T>),
    One(std::iter::Once<T>),
}
pub enum Iter<'t, T> {
    Many(std::slice::Iter<'t, T>),
    One(std::iter::Once<&'t T>),
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoIter::Many(it) => it.next(),
            IntoIter::One(it) => it.next(),
        }
    }
}
impl<'t, T> Iterator for Iter<'t, T> {
    type Item = &'t T;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Many(it) => it.next(),
            Iter::One(it) => it.next(),
        }
    }
}
impl<T> IntoIterator for OneOrMany<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            OneOrMany::Many(m) => IntoIter::Many(m.into_iter()),
            OneOrMany::One(o) => IntoIter::One(once(o)),
        }
    }
}
impl<'t, T> IntoIterator for &'t OneOrMany<T> {
    type IntoIter = Iter<'t, T>;
    type Item = &'t T;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            OneOrMany::Many(m) => Iter::Many(m.iter()),
            OneOrMany::One(o) => Iter::One(once(o)),
        }
    }
}