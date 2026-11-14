//! Модель данных семейного дерева. Сериализуется в `tree.json` внутри `.famtree`.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fmt;

pub type PersonId = u64;

/// Версия формата `tree.json`. Увеличивать при несовместимых изменениях.
pub const FORMAT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub enum Gender {
    Male,
    Female,
    #[default]
    Unknown,
}

/// Неполная дата: можно знать только год.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct PartialDate {
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub month: Option<u8>,
    #[serde(default)]
    pub day: Option<u8>,
}

/// Фото из контейнера: `hash` — sha256 (hex), файл `media/<hash>.jpg`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Photo {
    pub hash: String,
    #[serde(default)]
    pub caption: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct Person {
    #[serde(default)]
    pub id: PersonId,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub gender: Gender,
    #[serde(default)]
    pub birth: PartialDate,
    #[serde(default)]
    pub death: PartialDate,
    #[serde(default)]
    pub profession: String,
    #[serde(default)]
    pub notes: String,
    /// Фото для карточки (как на паспорт), hash.
    #[serde(default)]
    pub main_photo: Option<String>,
    /// Остальные фото (в заметках), без лимита.
    #[serde(default)]
    pub photos: Vec<Photo>,
}

impl Person {
    pub fn new(first_name: &str, last_name: &str) -> Self {
        Person {
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum Relation {
    ParentChild {
        parent: PersonId,
        child: PersonId,
    },
    Partners {
        a: PersonId,
        b: PersonId,
        #[serde(default)]
        start: PartialDate,
        #[serde(default)]
        end: PartialDate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    NoSuchPerson(PersonId),
    SelfRelation,
    Duplicate,
    Cycle,
    UnsupportedVersion(u32),
    Json(String),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::NoSuchPerson(id) => write!(f, "Человек {id} не найден"),
            ModelError::SelfRelation => write!(f, "Связь человека с самим собой"),
            ModelError::Duplicate => write!(f, "Такая связь уже есть"),
            ModelError::Cycle => write!(f, "Связь создаёт цикл в родстве"),
            ModelError::UnsupportedVersion(v) => {
                write!(f, "Версия файла {v} новее, чем поддерживает приложение")
            }
            ModelError::Json(e) => write!(f, "Ошибка данных: {e}"),
        }
    }
}

impl std::error::Error for ModelError {}

fn first_id() -> PersonId {
    1
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Tree {
    pub version: u32,
    /// «Я» — сам пользователь. Схема строится от верхних предков этого человека
    /// (родители, дяди, кузены и т.д. попадают на неё автоматически).
    #[serde(default)]
    pub root: Option<PersonId>,
    #[serde(default = "first_id")]
    pub next_id: PersonId,
    #[serde(default)]
    pub persons: BTreeMap<PersonId, Person>,
    #[serde(default)]
    pub relations: Vec<Relation>,
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            version: FORMAT_VERSION,
            root: None,
            next_id: 1,
            persons: BTreeMap::new(),
            relations: Vec::new(),
        }
    }

    /// Добавляет человека, назначает id (присланный id игнорируется).
    pub fn add_person(&mut self, mut p: Person) -> PersonId {
        let id = self.next_id;
        self.next_id += 1;
        p.id = id;
        self.persons.insert(id, p);
        id
    }

    /// Удаляет человека и все его связи.
    pub fn remove_person(&mut self, id: PersonId) -> bool {
        if self.persons.remove(&id).is_none() {
            return false;
        }
        self.relations.retain(|r| match r {
            Relation::ParentChild { parent, child } => *parent != id && *child != id,
            Relation::Partners { a, b, .. } => *a != id && *b != id,
        });
        if self.root == Some(id) {
            self.root = None;
        }
        true
    }

    fn check_exists(&self, id: PersonId) -> Result<(), ModelError> {
        if self.persons.contains_key(&id) {
            Ok(())
        } else {
            Err(ModelError::NoSuchPerson(id))
        }
    }

    pub fn add_parent_child(&mut self, parent: PersonId, child: PersonId) -> Result<(), ModelError> {
        self.check_exists(parent)?;
        self.check_exists(child)?;
        if parent == child {
            return Err(ModelError::SelfRelation);
        }
        let exists = self.relations.iter().any(|r| {
            matches!(r, Relation::ParentChild { parent: p, child: c } if *p == parent && *c == child)
        });
        if exists {
            return Err(ModelError::Duplicate);
        }
        // child не должен быть предком parent, иначе цикл.
        if self.is_ancestor(child, parent) {
            return Err(ModelError::Cycle);
        }
        self.relations.push(Relation::ParentChild { parent, child });
        Ok(())
    }

    pub fn add_partners(&mut self, a: PersonId, b: PersonId) -> Result<(), ModelError> {
        self.check_exists(a)?;
        self.check_exists(b)?;
        if a == b {
            return Err(ModelError::SelfRelation);
        }
        let exists = self.relations.iter().any(|r| {
            matches!(r, Relation::Partners { a: x, b: y, .. }
                if (*x == a && *y == b) || (*x == b && *y == a))
        });
        if exists {
            return Err(ModelError::Duplicate);
        }
        self.relations.push(Relation::Partners {
            a,
            b,
            start: PartialDate::default(),
            end: PartialDate::default(),
        });
        Ok(())
    }

    pub fn parents_of(&self, id: PersonId) -> Vec<PersonId> {
        self.relations
            .iter()
            .filter_map(|r| match r {
                Relation::ParentChild { parent, child } if *child == id => Some(*parent),
                _ => None,
            })
            .collect()
    }

    pub fn children_of(&self, id: PersonId) -> Vec<PersonId> {
        self.relations
            .iter()
            .filter_map(|r| match r {
                Relation::ParentChild { parent, child } if *parent == id => Some(*child),
                _ => None,
            })
            .collect()
    }

    pub fn partners_of(&self, id: PersonId) -> Vec<PersonId> {
        self.relations
            .iter()
            .filter_map(|r| match r {
                Relation::Partners { a, b, .. } if *a == id => Some(*b),
                Relation::Partners { a, b, .. } if *b == id => Some(*a),
                _ => None,
            })
            .collect()
    }

    /// Является ли `anc` предком `of` (по линиям родитель→ребёнок).
    pub fn is_ancestor(&self, anc: PersonId, of: PersonId) -> bool {
        let mut stack = vec![of];
        let mut seen: HashSet<PersonId> = HashSet::new();
        while let Some(cur) = stack.pop() {
            if !seen.insert(cur) {
                continue;
            }
            for p in self.parents_of(cur) {
                if p == anc {
                    return true;
                }
                stack.push(p);
            }
        }
        false
    }

    pub fn to_json(&self) -> Result<String, ModelError> {
        serde_json::to_string_pretty(self).map_err(|e| ModelError::Json(e.to_string()))
    }

    pub fn from_json(s: &str) -> Result<Tree, ModelError> {
        let tree: Tree = serde_json::from_str(s).map_err(|e| ModelError::Json(e.to_string()))?;
        if tree.version > FORMAT_VERSION {
            return Err(ModelError::UnsupportedVersion(tree.version));
        }
        Ok(tree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (Tree, PersonId, PersonId, PersonId) {
        let mut t = Tree::new();
        let dad = t.add_person(Person::new("Иван", "Петров"));
        let mom = t.add_person(Person::new("Анна", "Петрова"));
        let kid = t.add_person(Person::new("Олег", "Петров"));
        t.add_partners(dad, mom).unwrap();
        t.add_parent_child(dad, kid).unwrap();
        t.add_parent_child(mom, kid).unwrap();
        t.root = Some(dad);
        (t, dad, mom, kid)
    }

    #[test]
    fn json_roundtrip() {
        let (mut t, dad, _, _) = sample();
        {
            let p = t.persons.get_mut(&dad).unwrap();
            p.profession = "Инженер".into();
            p.birth.year = Some(1960);
            p.main_photo = Some("abc".into());
            p.photos.push(Photo { hash: "def".into(), caption: "Дача".into() });
        }
        let json = t.to_json().unwrap();
        let back = Tree::from_json(&json).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn relations_queries() {
        let (t, dad, mom, kid) = sample();
        assert_eq!(t.children_of(dad), vec![kid]);
        let mut parents = t.parents_of(kid);
        parents.sort();
        assert_eq!(parents, vec![dad, mom]);
        assert_eq!(t.partners_of(dad), vec![mom]);
        assert_eq!(t.partners_of(mom), vec![dad]);
    }

    #[test]
    fn remove_person_cleans_relations() {
        let (mut t, dad, _, kid) = sample();
        assert!(t.remove_person(dad));
        assert!(t.parents_of(kid).len() == 1);
        assert!(t.partners_of(kid).is_empty());
        assert_eq!(t.root, None);
        assert!(!t.remove_person(dad));
    }

    #[test]
    fn rejects_bad_relations() {
        let (mut t, dad, mom, kid) = sample();
        assert_eq!(t.add_parent_child(dad, dad), Err(ModelError::SelfRelation));
        assert_eq!(t.add_parent_child(dad, kid), Err(ModelError::Duplicate));
        assert_eq!(t.add_partners(mom, dad), Err(ModelError::Duplicate));
        assert_eq!(t.add_parent_child(kid, dad), Err(ModelError::Cycle));
        assert_eq!(t.add_parent_child(999, kid), Err(ModelError::NoSuchPerson(999)));
    }

    #[test]
    fn rejects_newer_version() {
        let json = r#"{"version": 999}"#;
        assert_eq!(Tree::from_json(json), Err(ModelError::UnsupportedVersion(999)));
    }

    #[test]
    fn missing_fields_use_defaults() {
        let json = r#"{"version": 1, "persons": {"1": {"id": 1, "first_name": "А"}}}"#;
        let t = Tree::from_json(json).unwrap();
        let p = &t.persons[&1];
        assert_eq!(p.first_name, "А");
        assert_eq!(p.last_name, "");
        assert_eq!(p.gender, Gender::Unknown);
        assert!(p.photos.is_empty());
    }

    #[test]
    fn ids_are_unique_after_removal() {
        let mut t = Tree::new();
        let a = t.add_person(Person::new("A", ""));
        t.remove_person(a);
        let b = t.add_person(Person::new("B", ""));
        assert_ne!(a, b);
    }
}
