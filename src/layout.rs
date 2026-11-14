//! Раскладка mindmap: от корня вниз по поколениям.
//! Единица раскладки (`Unit`) = человек по крови + его партнёры в одном ряду;
//! дети висят под единицей. Каждый человек размещается один раз.
//! Корень дерева (`Tree::root`) — сам пользователь («я»). Схема строится от верхних
//! предков корня вниз, так что видны родители, братья/сёстры, дяди, кузены и т.д.
//! Если у «меня» предки с двух сторон — деревья ставятся рядом.
//! Результат — только координаты (центры карточек); отрисовка — в `ui`.

use crate::model::{PersonId, Tree};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Pt {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
pub struct LayoutConfig {
    /// Размер карточки (фото + имя).
    pub card_w: f32,
    pub card_h: f32,
    /// Зазор между поддеревьями братьев/сестёр.
    pub h_gap: f32,
    /// Зазор между партнёрами в одном ряду.
    pub partner_gap: f32,
    /// Зазор между поколениями.
    pub v_gap: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        LayoutConfig {
            card_w: 120.0,
            card_h: 190.0,
            h_gap: 30.0,
            partner_gap: 12.0,
            v_gap: 90.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlacedPerson {
    pub id: PersonId,
    /// Центр карточки.
    pub pos: Pt,
}

/// Человек по крови + партнёры. `members[0]` — основной.
#[derive(Clone, Debug)]
pub struct Unit {
    pub members: Vec<PersonId>,
    pub center: Pt,
    pub width: f32,
}

/// Линия от единицы-родителя к карточке ребёнка.
#[derive(Clone, Debug)]
pub struct Edge {
    pub from: Pt,
    pub to: Pt,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Bounds {
    pub min: Pt,
    pub max: Pt,
}

#[derive(Clone, Debug, Default)]
pub struct Layout {
    pub nodes: Vec<PlacedPerson>,
    pub units: Vec<Unit>,
    pub edges: Vec<Edge>,
    pub bounds: Bounds,
    /// Люди, не достижимые от корня (показываются только в списке).
    pub unplaced: Vec<PersonId>,
    index: HashMap<PersonId, usize>,
}

impl Layout {
    pub fn pos_of(&self, id: PersonId) -> Option<Pt> {
        self.index.get(&id).map(|&i| self.nodes[i].pos)
    }
}

/// Индексы связей — один проход по relations (важно для больших деревьев).
struct Adj {
    children: HashMap<PersonId, Vec<PersonId>>,
    partners: HashMap<PersonId, Vec<PersonId>>,
    parents: HashMap<PersonId, Vec<PersonId>>,
}

impl Adj {
    fn new(tree: &Tree) -> Self {
        use crate::model::Relation;
        let mut children: HashMap<PersonId, Vec<PersonId>> = HashMap::new();
        let mut partners: HashMap<PersonId, Vec<PersonId>> = HashMap::new();
        let mut parents: HashMap<PersonId, Vec<PersonId>> = HashMap::new();
        for r in &tree.relations {
            match r {
                Relation::ParentChild { parent, child } => {
                    children.entry(*parent).or_default().push(*child);
                    parents.entry(*child).or_default().push(*parent);
                }
                Relation::Partners { a, b, .. } => {
                    partners.entry(*a).or_default().push(*b);
                    partners.entry(*b).or_default().push(*a);
                }
            }
        }
        Adj { children, partners, parents }
    }

    fn children(&self, id: PersonId) -> &[PersonId] {
        self.children.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    fn parents(&self, id: PersonId) -> &[PersonId] {
        self.parents.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    fn partners(&self, id: PersonId) -> &[PersonId] {
        self.partners.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

struct BuildNode {
    members: Vec<PersonId>,
    children: Vec<BuildNode>,
    width: f32,
}

fn unit_width(cfg: &LayoutConfig, k: usize) -> f32 {
    k as f32 * cfg.card_w + (k.saturating_sub(1)) as f32 * cfg.partner_gap
}

fn children_width(cfg: &LayoutConfig, kids: &[BuildNode]) -> f32 {
    if kids.is_empty() {
        return 0.0;
    }
    kids.iter().map(|c| c.width).sum::<f32>() + (kids.len() - 1) as f32 * cfg.h_gap
}

/// Верхние предки `me` (люди без родителей в дереве); если предков нет — сам `me`.
fn ancestor_tops(tree: &Tree, adj: &Adj, me: PersonId) -> Vec<PersonId> {
    let mut tops: BTreeSet<PersonId> = BTreeSet::new();
    let mut seen: HashSet<PersonId> = HashSet::new();
    let mut stack = vec![me];
    while let Some(cur) = stack.pop() {
        if !seen.insert(cur) {
            continue;
        }
        let ps: Vec<PersonId> = adj
            .parents(cur)
            .iter()
            .copied()
            .filter(|p| tree.persons.contains_key(p))
            .collect();
        if ps.is_empty() {
            tops.insert(cur);
        } else {
            stack.extend(ps);
        }
    }
    tops.into_iter().collect()
}

/// С каких людей начинать построение. Есть корень («я») — его верхние предки;
/// нет корня — все люди без родителей.
fn pick_tops(tree: &Tree, adj: &Adj) -> Vec<PersonId> {
    if let Some(r) = tree.root {
        if tree.persons.contains_key(&r) {
            return ancestor_tops(tree, adj, r);
        }
    }
    let mut tops: Vec<PersonId> = tree
        .persons
        .keys()
        .filter(|id| !adj.parents.contains_key(id))
        .copied()
        .collect();
    if tops.is_empty() {
        if let Some(&first) = tree.persons.keys().next() {
            tops.push(first);
        }
    }
    tops
}

fn build(
    tree: &Tree,
    adj: &Adj,
    cfg: &LayoutConfig,
    id: PersonId,
    visited: &mut HashSet<PersonId>,
) -> BuildNode {
    visited.insert(id);
    let mut members = vec![id];
    for &p in adj.partners(id) {
        if tree.persons.contains_key(&p) && visited.insert(p) {
            members.push(p);
        }
    }

    let mut kids: Vec<PersonId> = Vec::new();
    let mut seen: HashSet<PersonId> = HashSet::new();
    for m in &members {
        for &c in adj.children(*m) {
            if tree.persons.contains_key(&c) && !visited.contains(&c) && seen.insert(c) {
                kids.push(c);
            }
        }
    }
    // Старшие слева; без года рождения — в конец.
    kids.sort_by_key(|c| (tree.persons[c].birth.year.unwrap_or(i32::MAX), *c));

    let mut children = Vec::new();
    for c in kids {
        if !visited.contains(&c) {
            children.push(build(tree, adj, cfg, c, visited));
        }
    }
    let width = unit_width(cfg, members.len()).max(children_width(cfg, &children));
    BuildNode { members, children, width }
}

struct Out {
    nodes: Vec<PlacedPerson>,
    units: Vec<Unit>,
    edges: Vec<Edge>,
}

fn row_y(cfg: &LayoutConfig, depth: usize) -> f32 {
    depth as f32 * (cfg.card_h + cfg.v_gap) + cfg.card_h / 2.0
}

/// Размещает поддерево; возвращает x центра карточки основного человека.
fn place(node: &BuildNode, left: f32, depth: usize, cfg: &LayoutConfig, out: &mut Out) -> f32 {
    let cx = left + node.width / 2.0;
    let y = row_y(cfg, depth);
    let uw = unit_width(cfg, node.members.len());
    let unit_left = cx - uw / 2.0;

    for (i, m) in node.members.iter().enumerate() {
        let x = unit_left + cfg.card_w / 2.0 + i as f32 * (cfg.card_w + cfg.partner_gap);
        out.nodes.push(PlacedPerson { id: *m, pos: Pt { x, y } });
    }
    out.units.push(Unit {
        members: node.members.clone(),
        center: Pt { x: cx, y },
        width: uw,
    });

    let cw = children_width(cfg, &node.children);
    let mut child_left = left + (node.width - cw) / 2.0;
    let child_top = row_y(cfg, depth + 1) - cfg.card_h / 2.0;
    for child in &node.children {
        let anchor_x = place(child, child_left, depth + 1, cfg, out);
        out.edges.push(Edge {
            from: Pt { x: cx, y: y + cfg.card_h / 2.0 },
            to: Pt { x: anchor_x, y: child_top },
        });
        child_left += child.width + cfg.h_gap;
    }
    unit_left + cfg.card_w / 2.0
}

pub fn compute(tree: &Tree, cfg: &LayoutConfig) -> Layout {
    let adj = Adj::new(tree);
    let mut visited: HashSet<PersonId> = HashSet::new();
    let mut out = Out { nodes: Vec::new(), units: Vec::new(), edges: Vec::new() };

    // Деревья нескольких верхних предков стоят рядом друг с другом.
    let mut left = 0.0_f32;
    for top in pick_tops(tree, &adj) {
        if visited.contains(&top) {
            continue;
        }
        let node = build(tree, &adj, cfg, top, &mut visited);
        place(&node, left, 0, cfg, &mut out);
        left += node.width + cfg.h_gap * 3.0;
    }

    let unplaced: Vec<PersonId> = tree
        .persons
        .keys()
        .filter(|id| !visited.contains(id))
        .copied()
        .collect();

    let mut bounds = Bounds::default();
    if let Some(first) = out.nodes.first() {
        let (mut min_x, mut max_x) = (first.pos.x, first.pos.x);
        let (mut min_y, mut max_y) = (first.pos.y, first.pos.y);
        for n in &out.nodes {
            min_x = min_x.min(n.pos.x);
            max_x = max_x.max(n.pos.x);
            min_y = min_y.min(n.pos.y);
            max_y = max_y.max(n.pos.y);
        }
        bounds = Bounds {
            min: Pt { x: min_x - cfg.card_w / 2.0, y: min_y - cfg.card_h / 2.0 },
            max: Pt { x: max_x + cfg.card_w / 2.0, y: max_y + cfg.card_h / 2.0 },
        };
    }

    let index = out.nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
    Layout {
        nodes: out.nodes,
        units: out.units,
        edges: out.edges,
        bounds,
        unplaced,
        index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Person;
    use std::collections::BTreeMap;

    fn add(t: &mut Tree, name: &str, birth: Option<i32>) -> PersonId {
        let mut p = Person::new(name, "Т");
        p.birth.year = birth;
        t.add_person(p)
    }

    fn assert_no_overlap(l: &Layout, cfg: &LayoutConfig) {
        let mut rows: BTreeMap<i64, Vec<f32>> = BTreeMap::new();
        for n in &l.nodes {
            rows.entry((n.pos.y * 10.0).round() as i64).or_default().push(n.pos.x);
        }
        for xs in rows.values_mut() {
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            for w in xs.windows(2) {
                assert!(w[1] - w[0] >= cfg.card_w - 0.5, "карточки пересекаются: {} и {}", w[0], w[1]);
            }
        }
    }

    #[test]
    fn empty_tree() {
        let l = compute(&Tree::new(), &LayoutConfig::default());
        assert!(l.nodes.is_empty());
        assert!(l.edges.is_empty());
        assert!(l.unplaced.is_empty());
    }

    #[test]
    fn couple_with_two_kids() {
        let cfg = LayoutConfig::default();
        let mut t = Tree::new();
        let dad = add(&mut t, "Папа", Some(1960));
        let mom = add(&mut t, "Мама", Some(1962));
        let k1 = add(&mut t, "Старший", Some(1985));
        let k2 = add(&mut t, "Младший", Some(1990));
        t.add_partners(dad, mom).unwrap();
        for k in [k1, k2] {
            t.add_parent_child(dad, k).unwrap();
            t.add_parent_child(mom, k).unwrap();
        }
        t.root = Some(dad);
        let l = compute(&t, &cfg);

        assert_eq!(l.nodes.len(), 4);
        assert_eq!(l.units.len(), 3);
        assert_eq!(l.edges.len(), 2);
        assert!(l.unplaced.is_empty());

        let (p_dad, p_mom) = (l.pos_of(dad).unwrap(), l.pos_of(mom).unwrap());
        let (p1, p2) = (l.pos_of(k1).unwrap(), l.pos_of(k2).unwrap());
        assert_eq!(p_dad.y, p_mom.y);
        assert_eq!(p1.y, p2.y);
        assert!(p1.y > p_dad.y);
        assert!(p1.x < p2.x, "старший слева");
        assert!((p_mom.x - p_dad.x - (cfg.card_w + cfg.partner_gap)).abs() < 0.01);
        // Пара по центру над детьми.
        let couple_center = (p_dad.x + p_mom.x) / 2.0;
        assert!((couple_center - (p1.x + p2.x) / 2.0).abs() < 0.01);
        assert_no_overlap(&l, &cfg);
    }

    #[test]
    fn three_generations_no_overlap() {
        let cfg = LayoutConfig::default();
        let mut t = Tree::new();
        let root = add(&mut t, "Дед", Some(1930));
        let babushka = add(&mut t, "Бабка", Some(1932));
        t.add_partners(root, babushka).unwrap();
        let mut kids = Vec::new();
        for i in 0..3 {
            let k = add(&mut t, "Ребёнок", Some(1955 + i));
            t.add_parent_child(root, k).unwrap();
            kids.push(k);
        }
        let spouse = add(&mut t, "Невестка", None);
        t.add_partners(kids[0], spouse).unwrap();
        for i in 0..3 {
            let g = add(&mut t, "Внук", Some(1980 + i));
            t.add_parent_child(kids[0], g).unwrap();
            t.add_parent_child(spouse, g).unwrap();
        }
        let g2 = add(&mut t, "Внук2", Some(1982));
        t.add_parent_child(kids[1], g2).unwrap();
        t.root = Some(root);

        let l = compute(&t, &cfg);
        assert_eq!(l.nodes.len(), t.persons.len());
        assert!(l.unplaced.is_empty());
        assert_no_overlap(&l, &cfg);
        // Невестка стоит в одном ряду рядом с мужем.
        let (pk, ps) = (l.pos_of(kids[0]).unwrap(), l.pos_of(spouse).unwrap());
        assert_eq!(pk.y, ps.y);
        assert!((ps.x - pk.x - (cfg.card_w + cfg.partner_gap)).abs() < 0.01);
    }

    #[test]
    fn disconnected_person_is_unplaced() {
        let mut t = Tree::new();
        let a = add(&mut t, "А", None);
        let b = add(&mut t, "Б", None);
        let stranger = add(&mut t, "Чужой", None);
        t.add_parent_child(a, b).unwrap();
        t.root = Some(a);
        let l = compute(&t, &LayoutConfig::default());
        assert_eq!(l.nodes.len(), 2);
        assert_eq!(l.unplaced, vec![stranger]);
        assert!(l.pos_of(stranger).is_none());
    }

    #[test]
    fn root_defaults_to_parentless_person() {
        let mut t = Tree::new();
        let kid = add(&mut t, "Ребёнок", None);
        let parent = add(&mut t, "Родитель", None);
        t.add_parent_child(parent, kid).unwrap();
        // root не задан; у kid (id=1) есть родитель, значит корень — parent.
        let l = compute(&t, &LayoutConfig::default());
        let (pp, pk) = (l.pos_of(parent).unwrap(), l.pos_of(kid).unwrap());
        assert!(pk.y > pp.y);
    }

    #[test]
    fn root_is_me_shows_ancestors_siblings_and_cousins() {
        let cfg = LayoutConfig::default();
        let mut t = Tree::new();
        let gpa = add(&mut t, "Дед", Some(1930));
        let gma = add(&mut t, "Бабка", Some(1932));
        t.add_partners(gpa, gma).unwrap();
        let dad = add(&mut t, "Папа", Some(1958));
        let uncle = add(&mut t, "Дядя", Some(1960));
        for k in [dad, uncle] {
            t.add_parent_child(gpa, k).unwrap();
            t.add_parent_child(gma, k).unwrap();
        }
        let me = add(&mut t, "Я", Some(1985));
        let sis = add(&mut t, "Сестра", Some(1988));
        for k in [me, sis] {
            t.add_parent_child(dad, k).unwrap();
        }
        let cousin = add(&mut t, "Кузен", Some(1990));
        t.add_parent_child(uncle, cousin).unwrap();
        t.root = Some(me);

        let l = compute(&t, &cfg);
        assert_eq!(l.nodes.len(), t.persons.len());
        assert!(l.unplaced.is_empty());
        let (p_gpa, p_me, p_cousin) =
            (l.pos_of(gpa).unwrap(), l.pos_of(me).unwrap(), l.pos_of(cousin).unwrap());
        assert!(p_me.y > p_gpa.y);
        assert_eq!(p_me.y, p_cousin.y);
        assert_no_overlap(&l, &cfg);
    }

    #[test]
    fn both_parental_sides_are_placed_side_by_side() {
        let cfg = LayoutConfig::default();
        let mut t = Tree::new();
        let pg = add(&mut t, "Дед по папе", Some(1930));
        let mg = add(&mut t, "Дед по маме", Some(1932));
        let dad = add(&mut t, "Папа", Some(1958));
        let mom = add(&mut t, "Мама", Some(1960));
        t.add_parent_child(pg, dad).unwrap();
        t.add_parent_child(mg, mom).unwrap();
        t.add_partners(dad, mom).unwrap();
        let me = add(&mut t, "Я", Some(1985));
        t.add_parent_child(dad, me).unwrap();
        t.add_parent_child(mom, me).unwrap();
        t.root = Some(me);

        let l = compute(&t, &cfg);
        assert_eq!(l.nodes.len(), t.persons.len());
        assert!(l.unplaced.is_empty());
        assert_eq!(l.pos_of(pg).unwrap().y, l.pos_of(mg).unwrap().y);
        assert_no_overlap(&l, &cfg);
    }

    #[test]
    fn large_tree_all_placed_without_overlap() {
        let cfg = LayoutConfig::default();
        let mut t = Tree::new();
        let root = add(&mut t, "Корень", Some(1800));
        t.root = Some(root);
        let mut queue = std::collections::VecDeque::from([root]);
        while t.persons.len() < 3000 {
            let p = queue.pop_front().unwrap();
            for _ in 0..3 {
                if t.persons.len() >= 3000 {
                    break;
                }
                let c = add(&mut t, "П", None);
                t.add_parent_child(p, c).unwrap();
                queue.push_back(c);
            }
        }
        let l = compute(&t, &cfg);
        assert_eq!(l.nodes.len(), 3000);
        assert!(l.unplaced.is_empty());
        assert_eq!(l.edges.len(), 2999);
        assert_no_overlap(&l, &cfg);
    }
}
