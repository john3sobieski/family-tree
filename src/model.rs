//! Модель данных семейного дерева. Сериализуется в `tree.json` внутри `.famtree`.

use crate::i18n::{tr, trf};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub month: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    /// Текущая фамилия (у женщины — после замужества, если менялась).
    #[serde(default)]
    pub last_name: String,
    /// Фамилия при рождении (чаще всего фамилия отца). Используется только у женщин.
    #[serde(default)]
    pub maiden_name: String,
    /// Отчество.
    #[serde(default)]
    pub patronymic: String,
    #[serde(default)]
    pub gender: Gender,
    #[serde(default)]
    pub birth: PartialDate,
    #[serde(default)]
    pub death: PartialDate,
    /// Место рождения (свободный текст).
    #[serde(default)]
    pub birth_place: String,
    /// Место смерти (свободный текст).
    #[serde(default)]
    pub death_place: String,
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
    /// Национальности: коды стран (`flags::COUNTRIES`), не больше двух.
    #[serde(default)]
    pub nationalities: Vec<String>,
}

impl Person {
    pub fn new(first_name: &str, last_name: &str) -> Self {
        Person {
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            ..Default::default()
        }
    }

    /// Фамилия для показа. У женщины с фамилией при рождении, отличной от текущей:
    /// «Иванова (Петрова)», где в скобках фамилия при рождении.
    pub fn surname_display(&self) -> String {
        let last = self.last_name.trim();
        let maiden = self.maiden_name.trim();
        if self.gender != Gender::Female || maiden.is_empty() || maiden == last {
            last.to_string()
        } else if last.is_empty() {
            maiden.to_string()
        } else {
            format!("{last} ({maiden})")
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
            ModelError::NoSuchPerson(id) => write!(f, "{}", trf("Человек {id} не найден", &[("id", id.to_string())])),
            ModelError::SelfRelation => write!(f, "{}", tr("Связь человека с самим собой")),
            ModelError::Duplicate => write!(f, "{}", tr("Такая связь уже есть")),
            ModelError::Cycle => write!(f, "{}", tr("Связь создаёт цикл в родстве")),
            ModelError::UnsupportedVersion(v) => {
                write!(f, "{}", trf("Версия файла {v} новее, чем поддерживает приложение", &[("v", v.to_string())]))
            }
            ModelError::Json(e) => write!(f, "{}", trf("Ошибка данных: {err}", &[("err", e.to_string())])),
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

    /// Убирает связь родитель → ребёнок. `false`, если такой связи не было.
    pub fn remove_parent_child(&mut self, parent: PersonId, child: PersonId) -> bool {
        let before = self.relations.len();
        self.relations.retain(|r| {
            !matches!(r, Relation::ParentChild { parent: p, child: c } if *p == parent && *c == child)
        });
        self.relations.len() != before
    }

    /// Убирает связь партнёров. `false`, если такой связи не было.
    pub fn remove_partners(&mut self, a: PersonId, b: PersonId) -> bool {
        let before = self.relations.len();
        self.relations.retain(|r| {
            !matches!(r, Relation::Partners { a: x, b: y, .. }
                if (*x == a && *y == b) || (*x == b && *y == a))
        });
        self.relations.len() != before
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

    /// Читаемый JSON: короткие записи в одну строку, связи подписаны именами.
    /// Поля `_readme` и `_comment` — только пояснения: при загрузке игнорируются,
    /// при сохранении создаются заново.
    pub fn to_json(&self) -> Result<String, ModelError> {
        let mut value = serde_json::to_value(self).map_err(|e| ModelError::Json(e.to_string()))?;
        self.add_comments(&mut value);
        let mut out = String::new();
        write_json(&value, 0, &mut out);
        out.push('\n');
        Ok(out)
    }

    /// Имя человека для подсказок в файле: «Имя Фамилия (#id)».
    fn label(&self, id: PersonId) -> String {
        match self.persons.get(&id) {
            Some(p) => {
                let name = format!("{} {}", p.first_name.trim(), p.last_name.trim());
                let name = name.trim();
                if name.is_empty() {
                    format!("#{id}")
                } else {
                    format!("{name} (#{id})")
                }
            }
            None => format!("#{id}?"),
        }
    }

    fn add_comments(&self, value: &mut Value) {
        let Some(root) = value.as_object_mut() else {
            return;
        };
        if let Some(Value::Array(rels)) = root.get_mut("relations") {
            // Связи сериализуются в том же порядке, что и self.relations.
            for (rel, v) in self.relations.iter().zip(rels.iter_mut()) {
                let text = match rel {
                    Relation::ParentChild { parent, child } => format!(
                        "родитель {} → ребёнок {}",
                        self.label(*parent),
                        self.label(*child)
                    ),
                    Relation::Partners { a, b, .. } => {
                        format!("пара: {} + {}", self.label(*a), self.label(*b))
                    }
                };
                if let Value::Object(obj) = v {
                    *obj = with_first(std::mem::take(obj), "_comment", Value::String(text));
                }
            }
        }
        let readme = [
            "Данные семейного дерева (FamTree). Файл .famtree — ZIP, это его tree.json.",
            "persons — люди по id; relations — связи, в них указаны id людей (имена в _comment для удобства).",
            "Поля _readme и _comment приложение игнорирует и пересоздаёт при каждом сохранении.",
            "Читать можно в любом редакторе, но править прямо в .famtree нельзя: ZIP сломается. Для правки распакуй архив, измени tree.json и запакуй обратно.",
        ];
        let lines: Vec<Value> = readme.iter().map(|l| Value::String(l.to_string())).collect();
        *root = with_first(std::mem::take(root), "_readme", Value::Array(lines));
    }

    pub fn from_json(s: &str) -> Result<Tree, ModelError> {
        let tree: Tree = serde_json::from_str(s).map_err(|e| ModelError::Json(e.to_string()))?;
        if tree.version > FORMAT_VERSION {
            return Err(ModelError::UnsupportedVersion(tree.version));
        }
        Ok(tree)
    }
}

/// Новая карта, где `key` стоит первым (порядок ключей сохраняет `preserve_order`).
fn with_first(old: Map<String, Value>, key: &str, val: Value) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert(key.to_string(), val);
    for (k, v) in old {
        m.insert(k, v);
    }
    m
}

/// Максимальная длина записи (в символах), которую ещё пишем одной строкой.
const INLINE_MAX: usize = 150;

fn is_flat(v: &Value) -> bool {
    match v {
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
        _ => true,
    }
}

/// Значение одной строкой (`{"year": 2009, "month": 10}`).
fn inline_json(v: &Value) -> String {
    match v {
        Value::Array(a) => {
            let parts: Vec<String> = a.iter().map(inline_json).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Object(o) => {
            let parts: Vec<String> = o
                .iter()
                .map(|(k, x)| format!("{}: {}", Value::String(k.clone()), inline_json(x)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        other => other.to_string(),
    }
}

/// Можно ли записать значение одной строкой: внутри нет вложенных записей и оно недлинное.
fn fits_inline(v: &Value) -> bool {
    let flat = match v {
        Value::Object(o) => o.values().all(is_flat),
        Value::Array(a) => a.iter().all(is_flat),
        _ => true,
    };
    flat && inline_json(v).chars().count() <= INLINE_MAX
}

fn push_indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

/// Печать JSON: отступ 2 пробела, мелкие записи (даты, связи, списки) — в одну строку.
fn write_json(v: &Value, level: usize, out: &mut String) {
    match v {
        Value::Object(o) if !o.is_empty() => {
            if fits_inline(v) {
                out.push_str(&inline_json(v));
                return;
            }
            out.push_str("{\n");
            let last = o.len() - 1;
            for (i, (k, x)) in o.iter().enumerate() {
                push_indent(out, level + 1);
                out.push_str(&Value::String(k.clone()).to_string());
                out.push_str(": ");
                write_json(x, level + 1, out);
                if i != last {
                    out.push(',');
                }
                out.push('\n');
            }
            push_indent(out, level);
            out.push('}');
        }
        Value::Array(a) if !a.is_empty() => {
            if fits_inline(v) {
                out.push_str(&inline_json(v));
                return;
            }
            out.push_str("[\n");
            let last = a.len() - 1;
            for (i, x) in a.iter().enumerate() {
                push_indent(out, level + 1);
                write_json(x, level + 1, out);
                if i != last {
                    out.push(',');
                }
                out.push('\n');
            }
            push_indent(out, level);
            out.push(']');
        }
        other => out.push_str(&inline_json(other)),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn nationalities_roundtrip_and_old_files_load() {
        let mut t = Tree::new();
        let mut p = Person::new("A", "B");
        p.nationalities = vec!["PL".to_string(), "DE".to_string()];
        let id = t.add_person(p);
        let back = Tree::from_json(&t.to_json().unwrap()).unwrap();
        assert_eq!(back.persons[&id].nationalities, vec!["PL", "DE"]);

        let old = r#"{"version":1,"persons":{"1":{"id":1,"first_name":"X"}}}"#;
        let t = Tree::from_json(old).unwrap();
        assert!(t.persons[&1].nationalities.is_empty());
        assert!(t.persons[&1].birth_place.is_empty());
        assert!(t.persons[&1].death_place.is_empty());
    }
    #[test]
    fn surname_display_and_old_files() {
        let mut p = Person::new("Анна", "Иванова");
        p.gender = Gender::Female;
        assert_eq!(p.surname_display(), "Иванова");
        p.maiden_name = "Петрова".into();
        assert_eq!(p.surname_display(), "Иванова (Петрова)");
        p.maiden_name = "Иванова".into();
        assert_eq!(p.surname_display(), "Иванова");
        p.maiden_name = "Петрова".into();
        p.last_name.clear();
        assert_eq!(p.surname_display(), "Петрова");
        p.last_name = "Иванова".into();
        p.gender = Gender::Male;
        assert_eq!(p.surname_display(), "Иванова");

        let old = r#"{"version":1,"persons":{"1":{"id":1,"first_name":"X"}}}"#;
        let t = Tree::from_json(old).unwrap();
        assert!(t.persons[&1].patronymic.is_empty());
        assert!(t.persons[&1].maiden_name.is_empty());
    }

    use super::*;

    #[test]
    fn remove_relations() {
        let (mut t, dad, mom, kid) = sample();
        assert!(t.remove_parent_child(dad, kid));
        assert!(!t.remove_parent_child(dad, kid));
        assert!(t.parents_of(kid).iter().all(|p| *p != dad));
        assert!(t.remove_partners(mom, dad));
        assert!(!t.remove_partners(dad, mom));
        assert!(t.partners_of(dad).is_empty());
    }

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
    fn json_is_readable_and_loads_back() {
        let (mut t, dad, _, _) = sample();
        t.persons.get_mut(&dad).unwrap().birth.year = Some(1960);
        let json = t.to_json().unwrap();
        assert!(json.contains("\"_readme\""));
        assert!(json.contains("\"_comment\": \"родитель Иван Петров (#1) → ребёнок Олег Петров (#3)\""));
        assert!(json.contains("\"birth\": {\"year\": 1960}"));
        assert!(json.contains("\"death\": {}"));
        // Пояснения игнорируются при загрузке: дерево совпадает с исходным.
        assert_eq!(Tree::from_json(&json).unwrap(), t);
        // Повторное сохранение даёт тот же текст.
        assert_eq!(Tree::from_json(&json).unwrap().to_json().unwrap(), json);
    }

    #[test]
    fn old_pretty_json_with_null_dates_still_loads() {
        let old = r#"{"version":1,"root":1,"next_id":2,"persons":{"1":{"id":1,"first_name":"А",
            "birth":{"year":2000,"month":null,"day":null},
            "death":{"year":null,"month":null,"day":null}}},
            "relations":[{"type":"Partners","a":1,"b":1,"start":{"year":null,"month":null,"day":null},"end":{"year":null,"month":null,"day":null}}]}"#;
        let t = Tree::from_json(old).unwrap();
        assert_eq!(t.persons[&1].birth.year, Some(2000));
        assert_eq!(t.persons[&1].death, PartialDate::default());
        assert_eq!(t.relations.len(), 1);
    }

    #[test]
    fn json_roundtrip() {
        let (mut t, dad, _, _) = sample();
        {
            let p = t.persons.get_mut(&dad).unwrap();
            p.profession = "Инженер".into();
            p.birth.year = Some(1960);
            p.patronymic = "Иванович".into();
            p.maiden_name = "Петров".into();
            p.birth_place = "Минск".into();
            p.death_place = "Гродно".into();
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
