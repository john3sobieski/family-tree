//! Раскладка mindmap: от корня вниз по поколениям.
//! Единица раскладки (`Unit`) = человек по крови + его партнёры в одном ряду;
//! дети висят под единицей. Каждый человек размещается один раз.
//! Корень дерева (`Tree::root`) — сам пользователь («я»). Схема строится от верхних
//! предков корня вниз, так что видны родители, братья/сёстры, дяди, кузены и т.д.
//! Если у «меня» предки с двух сторон — деревья ставятся рядом.
//!
//! Семья «со стороны» (например, родители мамы, когда схема строится от родителей папы)
//! строится отдельным куском и ставится рядом с мамой: в ряду получается
//! [братья/сёстры папы] [папа мама] [братья/сёстры мамы] — «песочные часы». Над каждой
//! группой стоят её родители, группы не перемешиваются. См. `order_kids` и `attach_in_laws`.
//! Результат — только координаты (центры карточек); отрисовка — в `ui`.

use crate::model::{PersonId, Relation, Tree};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

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
    /// «Лесенка»: сдвиг вправо для каждого уровня потомков, подвешенных столбиком.
    pub hang_indent: f32,
    /// «Лесенка»: зазор между строками столбика.
    pub hang_gap: f32,
    /// Вручную: поддеревья из ≤ N единиц подвешиваются столбиком (0 — выкл.).
    /// Используется, только если `auto_compact == false`.
    pub hang_max: usize,
    /// Автоматически подбирать компактность, чтобы схема не была слишком широкой.
    pub auto_compact: bool,
    /// Автокомпактность включается только если людей не меньше этого числа
    /// (небольшие деревья остаются широкими и понятными, без «лесенок»).
    pub auto_compact_min: usize,
    /// Желаемое отношение ширины к высоте схемы (чем меньше — тем «выше» дерево).
    pub target_aspect: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        LayoutConfig {
            card_w: 120.0,
            card_h: 190.0,
            h_gap: 30.0,
            partner_gap: 12.0,
            v_gap: 90.0,
            hang_indent: 36.0,
            hang_gap: 28.0,
            hang_max: 0,
            auto_compact: true,
            auto_compact_min: 200,
            target_aspect: 1.6,
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
    /// `true` — связь «лесенки»: вниз по линии x = from.x, затем вбок к to.
    pub hang: bool,
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

/// Ширина поддерева, если подвесить его «лесенкой».
fn outline_width(cfg: &LayoutConfig, node: &BuildNode) -> f32 {
    let mut w = unit_width(cfg, node.members.len());
    for c in &node.children {
        w = w.max(cfg.hang_indent + outline_width(cfg, c));
    }
    w
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

/// С каких людей начинать построение (зерна). Есть корень («я») — его верхние предки;
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

/// Поколение каждого человека (номер ряда): ребёнок строго ниже всех родителей,
/// партнёры в одном ряду. Верхние предки без родителей опускаются к своим детям,
/// чтобы ветки разных сторон выравнивались по рядам.
fn compute_gens(tree: &Tree, adj: &Adj) -> HashMap<PersonId, i32> {
    let mut gn: HashMap<PersonId, i32> = tree.persons.keys().map(|&id| (id, 0)).collect();
    let limit = tree.persons.len() + 2;
    for _ in 0..limit {
        let mut changed = false;
        for r in &tree.relations {
            match r {
                Relation::ParentChild { parent, child } => {
                    if let (Some(&gp), Some(&gc)) = (gn.get(parent), gn.get(child)) {
                        if gc < gp + 1 {
                            gn.insert(*child, gp + 1);
                            changed = true;
                        }
                    }
                }
                Relation::Partners { a, b, .. } => {
                    if let (Some(&ga), Some(&gb)) = (gn.get(a), gn.get(b)) {
                        let m = ga.max(gb);
                        if ga != m {
                            gn.insert(*a, m);
                            changed = true;
                        }
                        if gb != m {
                            gn.insert(*b, m);
                            changed = true;
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }

    // Опускаем «свободные» пары/одиночек без родителей вплотную к их детям.
    let mut done: HashSet<PersonId> = HashSet::new();
    for &id in tree.persons.keys() {
        if done.contains(&id) || adj.parents.contains_key(&id) {
            continue;
        }
        let mut comp = vec![id];
        let mut seen: HashSet<PersonId> = HashSet::from([id]);
        let mut i = 0;
        while i < comp.len() {
            for &p in adj.partners(comp[i]) {
                if tree.persons.contains_key(&p) && seen.insert(p) {
                    comp.push(p);
                }
            }
            i += 1;
        }
        for m in &comp {
            done.insert(*m);
        }
        if comp.iter().any(|m| adj.parents.contains_key(m)) {
            continue;
        }
        let min_child = comp
            .iter()
            .flat_map(|m| adj.children(*m).iter())
            .filter_map(|c| gn.get(c))
            .min()
            .copied();
        if let Some(mc) = min_child {
            let g = mc - 1;
            if g > gn.get(&comp[0]).copied().unwrap_or(0) {
                for m in &comp {
                    gn.insert(*m, g);
                }
            }
        }
    }
    gn
}

fn gen_of(gens: &HashMap<PersonId, i32>, id: PersonId) -> usize {
    gens.get(&id).copied().unwrap_or(0).max(0) as usize
}

/// С какой стороны ряда братьев/сестёр стоит человек.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum End {
    Left,
    Right,
}

impl End {
    fn opposite(self) -> End {
        match self {
            End::Left => End::Right,
            End::Right => End::Left,
        }
    }
}

struct BuildNode {
    /// Люди единицы в порядке слева направо.
    members: Vec<PersonId>,
    /// Индекс «своего» (кровного) человека в `members`: к нему идёт линия от родителей.
    main: usize,
    children: Vec<BuildNode>,
    width: f32,
    /// Число единиц в поддереве (включая эту).
    count: usize,
    /// Поддерево рисуется «лесенкой» (столбиком), а не вширь.
    hang: bool,
    /// «Призрак» — пустое место под человека, который уже стоит в другом куске схемы
    /// (например, мама, чья семья строится отдельно). Карточка не рисуется, но куски
    /// потом совмещаются так, чтобы призрак лёг ровно на реального человека.
    ghost: Option<PersonId>,
    /// В поддереве есть призрак.
    anch: bool,
    /// В поддереве есть корень («я»).
    root: bool,
}

/// Общие данные для построения.
struct Ctx<'a> {
    tree: &'a Tree,
    adj: &'a Adj,
    cfg: &'a LayoutConfig,
    gens: &'a HashMap<PersonId, i32>,
    root: Option<PersonId>,
    hang_max: usize,
}

fn ghost_node(cfg: &LayoutConfig, id: PersonId) -> BuildNode {
    BuildNode {
        members: Vec::new(),
        main: 0,
        children: Vec::new(),
        width: cfg.card_w,
        count: 0,
        hang: false,
        ghost: Some(id),
        anch: true,
        root: false,
    }
}

/// У человека есть родители, которых ещё нет на схеме (его семья «со стороны»).
fn has_free_parents(cx: &Ctx, id: PersonId, visited: &HashSet<PersonId>) -> bool {
    cx.adj
        .parents(id)
        .iter()
        .any(|p| cx.tree.persons.contains_key(p) && !visited.contains(p))
}

/// В единице есть партнёр «со стороны», у которого на схеме ещё нет своей семьи.
fn marries_in(cx: &Ctx, node: &BuildNode, visited: &HashSet<PersonId>) -> bool {
    node.ghost.is_none()
        && node
            .members
            .iter()
            .enumerate()
            .any(|(i, m)| i != node.main && has_free_parents(cx, *m, visited))
}

/// Зеркалит единицу: партнёры меняются местами (кровный человек уходит вправо).
fn flip(k: &mut BuildNode) {
    k.members.reverse();
    k.main = k.members.len() - 1 - k.main;
}

/// Порядок детей одной семьи. Дети идут по возрасту, но:
/// - ребёнок, на чьём пути лежит призрак (или «я», если у его партнёра есть своя семья),
///   ставится на `path_end` — туда, где у него есть место для семьи партнёра;
/// - первый другой ребёнок с партнёром «со стороны» ставится на противоположный край.
/// Так у каждой семьи братья/сёстры стоят рядом друг с другом, а партнёр «со стороны»
/// получает свою семью с внешней стороны ряда («песочные часы»), и ветки не пересекаются.
fn order_kids(cx: &Ctx, kids: Vec<BuildNode>, visited: &HashSet<PersonId>, path_end: End) -> Vec<BuildNode> {
    if kids.len() < 2 {
        return kids;
    }
    let mut path_kid: Option<BuildNode> = None;
    let mut far_kid: Option<BuildNode> = None;
    let mut rest: Vec<BuildNode> = Vec::new();
    for k in kids {
        let is_path = k.anch || (k.root && marries_in(cx, &k, visited));
        if is_path && path_kid.is_none() {
            path_kid = Some(k);
        } else if far_kid.is_none() && marries_in(cx, &k, visited) {
            far_kid = Some(k);
        } else {
            rest.push(k);
        }
    }
    // Партнёр «со стороны» стоит с внешней стороны единицы, у края ряда.
    if let Some(k) = path_kid.as_mut() {
        if path_end == End::Left && marries_in(cx, k, visited) {
            flip(k);
        }
    }
    if let Some(k) = far_kid.as_mut() {
        if path_end.opposite() == End::Left {
            flip(k);
        }
    }
    let (left, right) = match path_end {
        End::Right => (far_kid, path_kid),
        End::Left => (path_kid, far_kid),
    };
    let mut out: Vec<BuildNode> = Vec::with_capacity(rest.len() + 2);
    out.extend(left);
    out.extend(rest);
    out.extend(right);
    out
}

/// `anchor` — человек, который уже стоит на схеме, а этот кусок строится как его семья
/// (родители и братья/сёстры): для него в нужной семье оставляется «призрак».
/// `path_end` — с какого края ряда ставить ветку, ведущую к якорю.
fn build(
    cx: &Ctx,
    id: PersonId,
    visited: &mut HashSet<PersonId>,
    anchor: Option<PersonId>,
    path_end: End,
) -> BuildNode {
    let (tree, adj, cfg) = (cx.tree, cx.adj, cx.cfg);
    visited.insert(id);
    // Единица: человек + все его партнёры (и партнёры партнёров) в одном ряду.
    let mut members = vec![id];
    let mut mi = 0;
    while mi < members.len() {
        let cur = members[mi];
        for &p in adj.partners(cur) {
            if tree.persons.contains_key(&p) && visited.insert(p) {
                members.push(p);
            }
        }
        mi += 1;
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

    let mut children: Vec<BuildNode> = Vec::new();
    for c in kids {
        if !visited.contains(&c) {
            children.push(build(cx, c, visited, anchor, path_end));
        }
    }
    if let Some(a) = anchor {
        if members.iter().any(|m| adj.children(*m).contains(&a)) {
            children.push(ghost_node(cfg, a));
        }
    }
    let children = order_kids(cx, children, &*visited, path_end);

    let count = 1 + children.iter().map(|c| c.count).sum::<usize>();
    let anch = children.iter().any(|c| c.anch);
    let root = cx.root.map_or(false, |r| members.contains(&r)) || children.iter().any(|c| c.root);
    let mut node = BuildNode {
        members,
        main: 0,
        children,
        width: 0.0,
        count,
        hang: false,
        ghost: None,
        anch,
        root,
    };
    // Ветку к якорю и ветку с «я» «лесенкой» не подвешиваем — они должны остаться «песочными часами».
    if cx.hang_max > 0 && !node.children.is_empty() && count <= cx.hang_max && !node.anch && !node.root {
        node.hang = true;
        node.width = outline_width(cfg, &node);
    } else {
        node.width = unit_width(cfg, node.members.len()).max(children_width(cfg, &node.children));
    }
    node
}

struct Out {
    nodes: Vec<PlacedPerson>,
    units: Vec<Unit>,
    edges: Vec<Edge>,
    /// Ребёнок (основной человек его единицы) → индекс единицы-родителя, под которой он стоит.
    placed_under: HashMap<PersonId, usize>,
    /// Где в этом куске стоит «призрак» (место якоря).
    anchor: Option<Pt>,
}

fn row_y(cfg: &LayoutConfig, depth: usize) -> f32 {
    depth as f32 * (cfg.card_h + cfg.v_gap) + cfg.card_h / 2.0
}

/// Размещает поддерево «лесенкой»: потомки идут столбиком под человеком,
/// каждый следующий уровень сдвинут вправо. `y` — центр карточки этого человека.
/// Возвращает y последней занятой строки.
fn place_hang(node: &BuildNode, left: f32, y: f32, cfg: &LayoutConfig, out: &mut Out) -> f32 {
    let uw = unit_width(cfg, node.members.len());
    let unit_idx = out.units.len();
    for (i, m) in node.members.iter().enumerate() {
        let x = left + cfg.card_w / 2.0 + i as f32 * (cfg.card_w + cfg.partner_gap);
        out.nodes.push(PlacedPerson { id: *m, pos: Pt { x, y } });
    }
    out.units.push(Unit {
        members: node.members.clone(),
        center: Pt { x: left + uw / 2.0, y },
        width: uw,
    });

    let pitch = cfg.card_h + cfg.hang_gap;
    let spine_x = left + cfg.hang_indent / 2.0;
    let child_left = left + cfg.hang_indent;
    let mut last_y = y;
    for child in &node.children {
        let cy = last_y + pitch;
        out.edges.push(Edge {
            from: Pt { x: spine_x, y: y + cfg.card_h / 2.0 },
            to: Pt { x: child_left, y: cy - cfg.card_h / 2.0 + cfg.card_w * 2.0 / 3.0 },
            hang: true,
        });
        out.placed_under.insert(child.members[child.main], unit_idx);
        last_y = place_hang(child, child_left, cy, cfg, out);
    }
    last_y
}

/// Размещает поддерево; возвращает x центра карточки основного (кровного) человека.
/// Ряд (y) определяется поколением человека, а не глубиной рекурсии.
fn place(
    node: &BuildNode,
    left: f32,
    cfg: &LayoutConfig,
    gens: &HashMap<PersonId, i32>,
    out: &mut Out,
) -> f32 {
    if let Some(g) = node.ghost {
        // Призрак: карточки нет, только запоминаем место.
        let gx = left + cfg.card_w / 2.0;
        out.anchor = Some(Pt { x: gx, y: row_y(cfg, gen_of(gens, g)) });
        return gx;
    }
    let depth = gen_of(gens, node.members[0]);
    if node.hang {
        place_hang(node, left, row_y(cfg, depth), cfg, out);
        return left + cfg.card_w / 2.0;
    }
    let cx = left + node.width / 2.0;
    let y = row_y(cfg, depth);
    let uw = unit_width(cfg, node.members.len());
    let unit_left = cx - uw / 2.0;
    let unit_idx = out.units.len();

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
    for child in &node.children {
        if child.ghost.is_some() {
            place(child, child_left, cfg, gens, out);
            child_left += child.width + cfg.h_gap;
            continue;
        }
        let child_top = row_y(cfg, gen_of(gens, child.members[0])) - cfg.card_h / 2.0;
        out.placed_under.insert(child.members[child.main], unit_idx);
        let anchor_x = place(child, child_left, cfg, gens, out);
        out.edges.push(Edge {
            from: Pt { x: cx, y: y + cfg.card_h / 2.0 },
            to: Pt { x: anchor_x, y: child_top },
            hang: false,
        });
        child_left += child.width + cfg.h_gap;
    }
    unit_left + cfg.card_w / 2.0 + node.main as f32 * (cfg.card_w + cfg.partner_gap)
}

fn new_out() -> Out {
    Out {
        nodes: Vec::new(),
        units: Vec::new(),
        edges: Vec::new(),
        placed_under: HashMap::new(),
        anchor: None,
    }
}

fn row_key(y: f32) -> i64 {
    (y * 10.0).round() as i64
}

/// Сдвиг куска вправо от всего, что уже размещено.
fn right_dx(piece: &Out, out: &Out, cfg: &LayoutConfig) -> f32 {
    let Some(min_x) = piece.nodes.iter().map(|n| n.pos.x).reduce(f32::min) else { return 0.0 };
    let Some(max_x) = out.nodes.iter().map(|n| n.pos.x).reduce(f32::max) else { return 0.0 };
    (max_x + cfg.card_w + cfg.h_gap * 3.0) - min_x
}

fn conflicts(piece: &Out, dx: f32, occ: &HashMap<i64, Vec<(f32, f32)>>, cfg: &LayoutConfig) -> bool {
    let half = cfg.card_w / 2.0 + cfg.h_gap / 2.0;
    piece.nodes.iter().any(|n| {
        occ.get(&row_key(n.pos.y)).map_or(false, |v| {
            let (lo, hi) = (n.pos.x + dx - half, n.pos.x + dx + half);
            v.iter().any(|&(a, b)| lo < b && a < hi)
        })
    })
}

/// Ближайший к `desired` сдвиг без наложений. `prefer` — куда двигать в первую очередь:
/// `1.0` вправо, `-1.0` влево (от родственника-якоря — прочь, а не сквозь соседей).
fn free_dx(
    piece: &Out,
    desired: f32,
    prefer: f32,
    occ: &HashMap<i64, Vec<(f32, f32)>>,
    cfg: &LayoutConfig,
) -> Option<f32> {
    if !conflicts(piece, desired, occ, cfg) {
        return Some(desired);
    }
    let step = cfg.card_w / 2.0;
    let dirs = if prefer >= 0.0 { [1.0_f32, -1.0] } else { [-1.0_f32, 1.0] };
    for dir in dirs {
        for k in 1..=800 {
            let d = desired + dir * step * k as f32;
            if !conflicts(piece, d, occ, cfg) {
                return Some(d);
            }
        }
    }
    None
}

fn merge_piece(
    out: &mut Out,
    occ: &mut HashMap<i64, Vec<(f32, f32)>>,
    placed: &mut HashMap<PersonId, Pt>,
    piece: Out,
    dx: f32,
    cfg: &LayoutConfig,
) {
    let base = out.units.len();
    let half = cfg.card_w / 2.0 + cfg.h_gap / 2.0;
    for n in piece.nodes {
        let pos = Pt { x: n.pos.x + dx, y: n.pos.y };
        occ.entry(row_key(pos.y)).or_default().push((pos.x - half, pos.x + half));
        placed.insert(n.id, pos);
        out.nodes.push(PlacedPerson { id: n.id, pos });
    }
    for mut u in piece.units {
        u.center.x += dx;
        out.units.push(u);
    }
    for mut e in piece.edges {
        e.from.x += dx;
        e.to.x += dx;
        out.edges.push(e);
    }
    for (k, v) in piece.placed_under {
        out.placed_under.insert(k, v + base);
    }
}

/// Накопленная схема: уже поставленные куски + служебные индексы.
struct Acc {
    out: Out,
    occ: HashMap<i64, Vec<(f32, f32)>>,
    placed: HashMap<PersonId, Pt>,
    /// Человек → x центра его единицы (чтобы понять, по какую сторону от неё он стоит).
    unit_cx: HashMap<PersonId, f32>,
    /// Люди, у которых ещё надо проверить, есть ли семья «со стороны».
    queue: VecDeque<PersonId>,
}

impl Acc {
    fn new() -> Self {
        Acc {
            out: new_out(),
            occ: HashMap::new(),
            placed: HashMap::new(),
            unit_cx: HashMap::new(),
            queue: VecDeque::new(),
        }
    }

    fn add(&mut self, piece: Out, dx: f32, cfg: &LayoutConfig) {
        let node_base = self.out.nodes.len();
        let unit_base = self.out.units.len();
        merge_piece(&mut self.out, &mut self.occ, &mut self.placed, piece, dx, cfg);
        for u in &self.out.units[unit_base..] {
            for m in &u.members {
                self.unit_cx.insert(*m, u.center.x);
            }
        }
        for n in &self.out.nodes[node_base..] {
            self.queue.push_back(n.id);
        }
    }
}

/// С какого верхнего предка строить семью родителей `x` (если их ещё нет на схеме).
fn next_in_law_top(cx: &Ctx, x: PersonId, visited: &HashSet<PersonId>) -> Option<PersonId> {
    for &p in cx.adj.parents(x) {
        if !cx.tree.persons.contains_key(&p) || visited.contains(&p) {
            continue;
        }
        let top = ancestor_tops(cx.tree, cx.adj, p).into_iter().find(|t| !visited.contains(t));
        return Some(top.unwrap_or(p));
    }
    None
}

/// Для каждого поставленного человека, у которого есть родители «со стороны» (мама,
/// пришедшая из другой семьи, и т.п.), строит эту семью отдельным куском и ставит
/// так, чтобы братья/сёстры стояли рядом с ним, а родители — над ними.
/// Кусок ставится с внешней стороны единицы и при столкновении отодвигается прочь.
fn attach_in_laws(cx: &Ctx, visited: &mut HashSet<PersonId>, acc: &mut Acc) {
    while let Some(x) = acc.queue.pop_front() {
        loop {
            let Some(top) = next_in_law_top(cx, x, &*visited) else { break };
            let (Some(xp), Some(ucx)) = (acc.placed.get(&x).copied(), acc.unit_cx.get(&x).copied()) else {
                break;
            };
            // Партнёр справа от центра единицы → его семья справа и наоборот.
            let side = if xp.x >= ucx { End::Right } else { End::Left };
            let node = build(cx, top, visited, Some(x), side.opposite());
            let mut piece = new_out();
            place(&node, 0.0, cx.cfg, cx.gens, &mut piece);
            let dx = match piece.anchor {
                Some(a) => {
                    let prefer = if side == End::Right { 1.0 } else { -1.0 };
                    free_dx(&piece, xp.x - a.x, prefer, &acc.occ, cx.cfg)
                        .unwrap_or_else(|| right_dx(&piece, &acc.out, cx.cfg))
                }
                None => right_dx(&piece, &acc.out, cx.cfg),
            };
            acc.add(piece, dx, cx.cfg);
        }
    }
}

/// Кандидаты порога «лесенки» при автоподборе (в единицах раскладки).
const HANG_STEPS: [usize; 12] = [0, 3, 5, 8, 13, 20, 30, 40, 60, 90, 121, 364];

pub fn compute(tree: &Tree, cfg: &LayoutConfig) -> Layout {
    let adj = Adj::new(tree);
    let tops = pick_tops(tree, &adj);
    let gens = compute_gens(tree, &adj);
    if !cfg.auto_compact {
        return compute_with(tree, &adj, &tops, &gens, cfg, cfg.hang_max);
    }
    // Небольшие деревья не «сжимаем»: пусть будут широкими, но понятными.
    if tree.persons.len() < cfg.auto_compact_min {
        return compute_with(tree, &adj, &tops, &gens, cfg, 0);
    }
    // Берём наименее компактную раскладку, у которой схема не шире целевых пропорций;
    // если такой нет — самую «высокую» из всех.
    let mut best: Option<(f32, Layout)> = None;
    for &h in HANG_STEPS.iter() {
        let l = compute_with(tree, &adj, &tops, &gens, cfg, h);
        let w = l.bounds.max.x - l.bounds.min.x;
        let ht = l.bounds.max.y - l.bounds.min.y;
        if ht <= 0.0 || l.nodes.is_empty() {
            return l;
        }
        let aspect = w / ht;
        if aspect <= cfg.target_aspect {
            return l;
        }
        let better = match &best {
            Some((a, _)) => aspect < *a,
            None => true,
        };
        if better {
            best = Some((aspect, l));
        }
    }
    best.map(|(_, l)| l).unwrap_or_default()
}

fn compute_with(
    tree: &Tree,
    adj: &Adj,
    tops: &[PersonId],
    gens: &HashMap<PersonId, i32>,
    cfg: &LayoutConfig,
    hang_max: usize,
) -> Layout {
    let cx = Ctx {
        tree,
        adj,
        cfg,
        gens,
        root: tree.root.filter(|r| tree.persons.contains_key(r)),
        hang_max,
    };
    let mut visited: HashSet<PersonId> = HashSet::new();
    let mut acc = Acc::new();

    // 1) Первое дерево строим от верхнего предка (в нём «я»). Затем для каждого человека,
    // у которого на схеме нет родителей (мама, пришедшая из другой семьи, и т.п.), строится
    // и ставится её семья — рядом с ним, на внешней стороне (см. `attach_in_laws`).
    // Остальные зёрна нужны только для несвязанных между собой групп людей (корня нет).
    for &top in tops {
        if visited.contains(&top) {
            continue;
        }
        let node = build(&cx, top, &mut visited, None, End::Right);
        let mut piece = new_out();
        place(&node, 0.0, cfg, gens, &mut piece);
        let dx = if acc.out.nodes.is_empty() { 0.0 } else { right_dx(&piece, &acc.out, cfg) };
        acc.add(piece, dx, cfg);
        attach_in_laws(&cx, &mut visited, &mut acc);
    }

    let mut out = acc.out;

    // Связи родитель → ребёнок, которые не вошли в «дерево под единицей»
    // (ребёнок стоит рядом с партнёром в чужой единице): рисуем отдельной линией.
    let mut unit_of: HashMap<PersonId, usize> = HashMap::new();
    for (i, u) in out.units.iter().enumerate() {
        for m in &u.members {
            unit_of.insert(*m, i);
        }
    }
    let node_idx: HashMap<PersonId, usize> =
        out.nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
    let mut cross: Vec<Edge> = Vec::new();
    let mut seen_cross: HashSet<(usize, PersonId)> = HashSet::new();
    for r in &tree.relations {
        if let Relation::ParentChild { parent, child } = r {
            let (Some(&pu), Some(&ci)) = (unit_of.get(parent), node_idx.get(child)) else {
                continue;
            };
            if out.placed_under.get(child) == Some(&pu) || !seen_cross.insert((pu, *child)) {
                continue;
            }
            let u = &out.units[pu];
            let c = out.nodes[ci].pos;
            cross.push(Edge {
                from: Pt { x: u.center.x, y: u.center.y + cfg.card_h / 2.0 },
                to: Pt { x: c.x, y: c.y - cfg.card_h / 2.0 },
                hang: false,
            });
        }
    }
    out.edges.extend(cross);

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

    fn plain() -> LayoutConfig {
        LayoutConfig { auto_compact: false, hang_max: 0, ..LayoutConfig::default() }
    }

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
        let l = compute(&Tree::new(), &plain());
        assert!(l.nodes.is_empty());
        assert!(l.edges.is_empty());
        assert!(l.unplaced.is_empty());
    }

    #[test]
    fn couple_with_two_kids() {
        let cfg = plain();
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
        let cfg = plain();
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
        let l = compute(&t, &plain());
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
        let l = compute(&t, &plain());
        let (pp, pk) = (l.pos_of(parent).unwrap(), l.pos_of(kid).unwrap());
        assert!(pk.y > pp.y);
    }

    #[test]
    fn root_is_me_shows_ancestors_siblings_and_cousins() {
        let cfg = plain();
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
        let cfg = plain();
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

    fn ternary(n: usize) -> Tree {
        let mut t = Tree::new();
        let root = add(&mut t, "Корень", Some(1800));
        t.root = Some(root);
        let mut queue = std::collections::VecDeque::from([root]);
        while t.persons.len() < n {
            let p = queue.pop_front().unwrap();
            for _ in 0..3 {
                if t.persons.len() >= n {
                    break;
                }
                let c = add(&mut t, "П", None);
                t.add_parent_child(p, c).unwrap();
                queue.push_back(c);
            }
        }
        t
    }

    #[test]
    fn hanging_makes_layout_narrower_and_taller() {
        let t = ternary(1000);
        let flat = compute(&t, &plain());
        let hung = compute(&t, &LayoutConfig { auto_compact: false, hang_max: 13, ..plain() });
        assert_eq!(hung.nodes.len(), 1000);
        assert!(hung.unplaced.is_empty());
        assert_eq!(hung.edges.len(), 999);
        assert_no_overlap(&hung, &plain());
        let fw = flat.bounds.max.x - flat.bounds.min.x;
        let hw = hung.bounds.max.x - hung.bounds.min.x;
        let fh = flat.bounds.max.y - flat.bounds.min.y;
        let hh = hung.bounds.max.y - hung.bounds.min.y;
        assert!(hw < fw / 3.0, "ширина должна сильно уменьшиться: {hw} vs {fw}");
        assert!(hh > fh, "высота должна вырасти");
    }

    #[test]
    fn auto_compact_keeps_small_tree_and_compacts_big_one() {
        let cfg = LayoutConfig::default();
        // Небольшое дерево остаётся обычным: нет ни одной связи «лесенкой».
        let small = compute(&ternary(4), &cfg);
        assert!(small.edges.iter().all(|e| !e.hang));
        // Большое — получает «лесенки» и становится заметно ближе к экранным пропорциям.
        let t = ternary(1000);
        let big = compute(&t, &cfg);
        let flat = compute(&t, &plain());
        assert_eq!(big.nodes.len(), 1000);
        assert!(big.edges.iter().any(|e| e.hang));
        assert_no_overlap(&big, &cfg);
        let aspect = |l: &Layout| (l.bounds.max.x - l.bounds.min.x) / (l.bounds.max.y - l.bounds.min.y);
        assert!(aspect(&big) < aspect(&flat) / 4.0);
    }

    #[test]
    fn large_tree_all_placed_without_overlap() {
        let cfg = plain();
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

    #[test]
    fn parents_of_a_parent_are_attached_above_in_rows() {
        // я <- A, B (партнёры); B <- C, D (партнёры). Родители B должны стоять над B.
        let cfg = plain();
        let mut t = Tree::new();
        let me = add(&mut t, "Я", Some(1990));
        let a = add(&mut t, "А", None);
        let b = add(&mut t, "Б", None);
        let c = add(&mut t, "В", None);
        let d = add(&mut t, "Г", None);
        t.add_partners(a, b).unwrap();
        t.add_partners(c, d).unwrap();
        t.add_parent_child(a, me).unwrap();
        t.add_parent_child(b, me).unwrap();
        t.add_parent_child(c, b).unwrap();
        t.add_parent_child(d, b).unwrap();
        t.root = Some(me);
        let l = compute(&t, &cfg);
        assert_eq!(l.nodes.len(), 5);
        assert!(l.unplaced.is_empty());
        let (pc, pd, pa, pb, pm) = (
            l.pos_of(c).unwrap(),
            l.pos_of(d).unwrap(),
            l.pos_of(a).unwrap(),
            l.pos_of(b).unwrap(),
            l.pos_of(me).unwrap(),
        );
        assert_eq!(pc.y, pd.y);
        assert_eq!(pa.y, pb.y);
        assert!(pc.y < pa.y && pa.y < pm.y);
        assert_eq!(l.edges.len(), 2); // (В,Г)→Б и (А,Б)→я
        assert_no_overlap(&l, &cfg);
    }

    #[test]
    fn both_partners_have_parents_cross_edge_and_aligned_rows() {
        // я <- A, B; A <- E, F; B <- C, D: обе линии предков над своим ребёнком, один ряд.
        let cfg = plain();
        let mut t = Tree::new();
        let me = add(&mut t, "Я", None);
        let a = add(&mut t, "А", None);
        let b = add(&mut t, "Б", None);
        let c = add(&mut t, "В", None);
        let d = add(&mut t, "Г", None);
        let e = add(&mut t, "Д", None);
        let f = add(&mut t, "Е", None);
        t.add_partners(a, b).unwrap();
        t.add_partners(c, d).unwrap();
        t.add_partners(e, f).unwrap();
        for p in [a, b] {
            t.add_parent_child(p, me).unwrap();
        }
        for p in [c, d] {
            t.add_parent_child(p, b).unwrap();
        }
        for p in [e, f] {
            t.add_parent_child(p, a).unwrap();
        }
        t.root = Some(me);
        let l = compute(&t, &cfg);
        assert_eq!(l.nodes.len(), 7);
        assert!(l.unplaced.is_empty());
        let y = |id| l.pos_of(id).unwrap().y;
        assert_eq!(y(c), y(e));
        assert_eq!(y(d), y(f));
        assert_eq!(y(a), y(b));
        assert!(y(c) < y(a) && y(a) < y(me));
        // Одна из двух линий предков попадает в «дерево», другая — отдельной линией.
        assert_eq!(l.edges.len(), 3);
        assert!(l.edges.iter().all(|ed| ed.to.y > ed.from.y));
        assert_no_overlap(&l, &cfg);
    }

    #[test]
    fn spouse_parents_are_shown_and_deep_side_aligns_rows() {
        // Дед-прадед только с одной стороны: ряды ветки без прадеда опускаются ближе к детям.
        let cfg = plain();
        let mut t = Tree::new();
        let me = add(&mut t, "Я", None);
        let a = add(&mut t, "А", None);
        let b = add(&mut t, "Б", None);
        let g = add(&mut t, "Прадед", None);
        let h = add(&mut t, "Дед", None);
        let m = add(&mut t, "Тесть", None);
        t.add_partners(a, b).unwrap();
        t.add_parent_child(a, me).unwrap();
        t.add_parent_child(b, me).unwrap();
        t.add_parent_child(g, h).unwrap();
        t.add_parent_child(h, a).unwrap();
        t.add_parent_child(m, b).unwrap();
        t.root = Some(me);
        let l = compute(&t, &cfg);
        assert_eq!(l.nodes.len(), 6);
        assert!(l.unplaced.is_empty());
        let y = |id| l.pos_of(id).unwrap().y;
        assert_eq!(y(h), y(m)); // Дед и Тесть — один ряд (родители А и Б)
        assert!(y(g) < y(h));
        assert_no_overlap(&l, &cfg);
    }

    #[test]
    fn in_laws_of_a_descendant_are_placed() {
        // У невестки (партнёр потомка) есть родители — они тоже на схеме.
        let cfg = plain();
        let mut t = Tree::new();
        let root = add(&mut t, "Корень", None);
        let kid = add(&mut t, "Ребёнок", None);
        let wife = add(&mut t, "Жена", None);
        let her_dad = add(&mut t, "Тесть", None);
        t.add_parent_child(root, kid).unwrap();
        t.add_partners(kid, wife).unwrap();
        t.add_parent_child(her_dad, wife).unwrap();
        t.root = Some(root);
        let l = compute(&t, &cfg);
        assert_eq!(l.nodes.len(), 4);
        assert!(l.unplaced.is_empty());
        assert!(l.pos_of(her_dad).unwrap().y < l.pos_of(wife).unwrap().y);
        assert_eq!(l.edges.len(), 2);
        assert_no_overlap(&l, &cfg);
    }

    /// Линии «шины» разных семей в одной паре рядов не должны перекрываться по x:
    /// иначе ветки визуально склеиваются в одну.
    fn assert_buses_separate(l: &Layout) {
        let mut groups: BTreeMap<(i64, i64), BTreeMap<i64, (f32, f32)>> = BTreeMap::new();
        for e in l.edges.iter().filter(|e| !e.hang) {
            let g = groups.entry((row_key(e.from.y), row_key(e.to.y))).or_default();
            let r = g.entry(row_key(e.from.x)).or_insert((e.from.x, e.from.x));
            r.0 = r.0.min(e.to.x).min(e.from.x);
            r.1 = r.1.max(e.to.x).max(e.from.x);
        }
        for g in groups.values() {
            let rs: Vec<(f32, f32)> = g.values().copied().collect();
            for i in 0..rs.len() {
                for j in (i + 1)..rs.len() {
                    assert!(
                        rs[i].1 < rs[j].0 || rs[j].1 < rs[i].0,
                        "шины семей перекрываются: {:?} и {:?}",
                        rs[i],
                        rs[j]
                    );
                }
            }
        }
    }

    /// Пара родителей и `n` детей (дети по возрасту). Возвращает (отец, мать, дети).
    fn family(t: &mut Tree, n: usize, year: i32) -> (PersonId, PersonId, Vec<PersonId>) {
        let f = add(t, "Отец", Some(year));
        let m = add(t, "Мать", Some(year + 2));
        t.add_partners(f, m).unwrap();
        let mut kids = Vec::new();
        for i in 0..n {
            let k = add(t, "Ребёнок", Some(year + 25 + i as i32));
            t.add_parent_child(f, k).unwrap();
            t.add_parent_child(m, k).unwrap();
            kids.push(k);
        }
        (f, m, kids)
    }

    fn unit_cx(l: &Layout, id: PersonId) -> f32 {
        l.units.iter().find(|u| u.members.contains(&id)).unwrap().center.x
    }

    #[test]
    fn five_siblings_on_both_parental_sides_form_hourglass() {
        // У папы и у мамы по пять братьев/сестёр: [семья папы] [папа мама] [семья мамы].
        let cfg = plain();
        let mut t = Tree::new();
        let (gf1, _gf2, dad_kids) = family(&mut t, 6, 1930); // ids 1.. — строится от этой стороны
        let (gm1, _gm2, mom_kids) = family(&mut t, 6, 1932);
        let dad = dad_kids[2];
        let mom = mom_kids[3];
        let dad_sibs: Vec<PersonId> = dad_kids.iter().copied().filter(|&k| k != dad).collect();
        let mom_sibs: Vec<PersonId> = mom_kids.iter().copied().filter(|&k| k != mom).collect();
        t.add_partners(dad, mom).unwrap();
        let me = add(&mut t, "Я", Some(1985));
        t.add_parent_child(dad, me).unwrap();
        t.add_parent_child(mom, me).unwrap();
        t.root = Some(me);

        let l = compute(&t, &cfg);
        assert_eq!(l.nodes.len(), t.persons.len());
        assert!(l.unplaced.is_empty());
        assert_no_overlap(&l, &cfg);
        assert_buses_separate(&l);

        let x = |id: PersonId| l.pos_of(id).unwrap().x;
        let y = |id: PersonId| l.pos_of(id).unwrap().y;
        for &k in dad_sibs.iter().chain(mom_sibs.iter()) {
            assert_eq!(y(k), y(dad), "братья/сёстры родителей в одном ряду с ними");
        }
        assert!(y(me) > y(dad));
        // Семьи не перемешаны: сначала папины братья/сёстры, потом папа и мама, потом мамины.
        for &k in &dad_sibs {
            assert!(x(k) < x(dad));
        }
        for &k in &mom_sibs {
            assert!(x(k) > x(mom));
        }
        assert!(x(dad) < x(mom));
        // Родители каждой семьи стоят над своими детьми.
        let (lo, hi) = dad_kids.iter().map(|&k| x(k)).fold((f32::MAX, f32::MIN), |a, v| (a.0.min(v), a.1.max(v)));
        let c = unit_cx(&l, gf1);
        assert!(lo <= c && c <= hi, "родители папы над его семьёй: {lo} <= {c} <= {hi}");
        let (lo, hi) = mom_kids.iter().map(|&k| x(k)).fold((f32::MAX, f32::MIN), |a, v| (a.0.min(v), a.1.max(v)));
        let c = unit_cx(&l, gm1);
        assert!(lo <= c && c <= hi, "родители мамы над её семьёй: {lo} <= {c} <= {hi}");
    }

    #[test]
    fn aunts_husband_family_goes_to_the_far_side() {
        // У тёти муж «со стороны» с родителями и братьями: тётя ставится на левый край,
        // муж — слева от неё, а его семья ещё левее. Папа с мамой — на правом краю.
        let cfg = plain();
        let mut t = Tree::new();
        let gf1 = add(&mut t, "Дед", Some(1930));
        let gf2 = add(&mut t, "Бабка", Some(1932));
        t.add_partners(gf1, gf2).unwrap();
        let aunt = add(&mut t, "Тётя", Some(1958));
        let dad = add(&mut t, "Папа", Some(1960));
        for k in [aunt, dad] {
            t.add_parent_child(gf1, k).unwrap();
            t.add_parent_child(gf2, k).unwrap();
        }
        // Семья мамы.
        let (_gm1, _gm2, mom_kids) = family(&mut t, 4, 1935);
        let mom = mom_kids[1];
        t.add_partners(dad, mom).unwrap();
        // Семья мужа тёти.
        let (_hp1, _hp2, h_kids) = family(&mut t, 4, 1932);
        let husband = h_kids[2];
        t.add_partners(aunt, husband).unwrap();
        let me = add(&mut t, "Я", Some(1985));
        t.add_parent_child(dad, me).unwrap();
        t.add_parent_child(mom, me).unwrap();
        t.root = Some(me);

        let l = compute(&t, &cfg);
        assert_eq!(l.nodes.len(), t.persons.len());
        assert!(l.unplaced.is_empty());
        assert_no_overlap(&l, &cfg);
        assert_buses_separate(&l);

        let x = |id: PersonId| l.pos_of(id).unwrap().x;
        let y = |id: PersonId| l.pos_of(id).unwrap().y;
        assert!(x(husband) < x(aunt), "муж слева от тёти");
        assert!(x(aunt) < x(dad) && x(dad) < x(mom));
        for &k in h_kids.iter().filter(|&&k| k != husband) {
            assert!(x(k) < x(husband), "семья мужа слева от него");
            assert_eq!(y(k), y(aunt));
        }
        for &k in mom_kids.iter().filter(|&&k| k != mom) {
            assert!(x(k) > x(mom), "семья мамы справа от неё");
            assert_eq!(y(k), y(dad));
        }
    }

    #[test]
    fn many_siblings_with_families_everywhere_do_not_overlap() {
        let cfg = plain();
        let mut t = Tree::new();
        let gf1 = add(&mut t, "Дед", Some(1930));
        let gf2 = add(&mut t, "Бабка", Some(1932));
        t.add_partners(gf1, gf2).unwrap();
        let gm1 = add(&mut t, "Дед2", Some(1931));
        let gm2 = add(&mut t, "Бабка2", Some(1933));
        t.add_partners(gm1, gm2).unwrap();
        let mut dad_side = Vec::new();
        let mut mom_side = Vec::new();
        for i in 0..6 {
            let a = add(&mut t, "Сиблинг", Some(1955 + i));
            t.add_parent_child(gf1, a).unwrap();
            t.add_parent_child(gf2, a).unwrap();
            dad_side.push(a);
            let b = add(&mut t, "Сиблинг", Some(1957 + i));
            t.add_parent_child(gm1, b).unwrap();
            t.add_parent_child(gm2, b).unwrap();
            mom_side.push(b);
        }
        let (dad, mom) = (dad_side[2], mom_side[3]);
        t.add_partners(dad, mom).unwrap();
        // У каждого брата/сестры родителей — партнёр и двое детей.
        for &s in dad_side.iter().chain(mom_side.iter()) {
            if s == dad || s == mom {
                continue;
            }
            let w = add(&mut t, "Партнёр", None);
            t.add_partners(s, w).unwrap();
            for j in 0..2 {
                let c = add(&mut t, "Кузен", Some(1985 + j));
                t.add_parent_child(s, c).unwrap();
                t.add_parent_child(w, c).unwrap();
            }
        }
        // «Я» и пятеро братьев/сестёр.
        let mut me_side = Vec::new();
        for i in 0..6 {
            let k = add(&mut t, "Я", Some(1985 + i));
            t.add_parent_child(dad, k).unwrap();
            t.add_parent_child(mom, k).unwrap();
            me_side.push(k);
        }
        t.root = Some(me_side[0]);

        let l = compute(&t, &cfg);
        assert_eq!(l.nodes.len(), t.persons.len());
        assert!(l.unplaced.is_empty());
        assert_no_overlap(&l, &cfg);
        assert!(l.edges.iter().all(|e| e.to.y > e.from.y));
        let y = |id: PersonId| l.pos_of(id).unwrap().y;
        for &s in dad_side.iter().chain(mom_side.iter()) {
            assert_eq!(y(s), y(dad));
        }
        for &k in &me_side {
            assert_eq!(y(k), y(me_side[0]));
            assert!(y(k) > y(dad));
        }
    }
}
