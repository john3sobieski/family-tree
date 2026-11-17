//! Демо-деревья для проверки отображения (без фото).

use crate::model::{Gender, Person, PersonId, Tree};

fn add(t: &mut Tree, first: &str, last: &str, g: Gender, year: i32) -> PersonId {
    let mut p = Person::new(first, last);
    p.gender = g;
    p.birth.year = Some(year);
    t.add_person(p)
}

fn couple(t: &mut Tree, a: PersonId, b: PersonId, kids: &[PersonId]) {
    t.add_partners(a, b).unwrap();
    for &k in kids {
        t.add_parent_child(a, k).unwrap();
        t.add_parent_child(b, k).unwrap();
    }
}

/// Небольшая семья на 4 поколения: «я» с родителями, дедушками, дядями, кузенами, детьми.
pub fn sample_tree() -> Tree {
    use Gender::{Female as F, Male as M};
    let mut t = Tree::new();
    let pg = add(&mut t, "Пётр", "Иванов", M, 1930);
    let pgw = add(&mut t, "Мария", "Иванова", F, 1932);
    let dad = add(&mut t, "Сергей", "Иванов", M, 1958);
    let uncle = add(&mut t, "Алексей", "Иванов", M, 1961);
    let aunt = add(&mut t, "Татьяна", "Белова", F, 1964);
    couple(&mut t, pg, pgw, &[dad, uncle, aunt]);

    let mg = add(&mut t, "Николай", "Смирнов", M, 1935);
    let mgw = add(&mut t, "Ольга", "Смирнова", F, 1937);
    let mom = add(&mut t, "Елена", "Иванова", F, 1960);
    let uncle2 = add(&mut t, "Виктор", "Смирнов", M, 1963);
    couple(&mut t, mg, mgw, &[mom, uncle2]);

    let me = add(&mut t, "Дмитрий", "Иванов", M, 1985);
    let sis = add(&mut t, "Анна", "Иванова", F, 1988);
    let bro = add(&mut t, "Максим", "Иванов", M, 1992);
    couple(&mut t, dad, mom, &[me, sis, bro]);

    let uw = add(&mut t, "Ирина", "Иванова", F, 1962);
    let c1 = add(&mut t, "Павел", "Иванов", M, 1987);
    let c2 = add(&mut t, "Кирилл", "Иванов", M, 1990);
    couple(&mut t, uncle, uw, &[c1, c2]);

    let ah = add(&mut t, "Игорь", "Белов", M, 1963);
    let c3 = add(&mut t, "Софья", "Белова", F, 1991);
    couple(&mut t, aunt, ah, &[c3]);

    let u2w = add(&mut t, "Наталья", "Смирнова", F, 1965);
    let c4 = add(&mut t, "Артём", "Смирнов", M, 1989);
    couple(&mut t, uncle2, u2w, &[c4]);

    let wife = add(&mut t, "Юлия", "Иванова", F, 1986);
    let k1 = add(&mut t, "Лев", "Иванов", M, 2015);
    let k2 = add(&mut t, "Ева", "Иванова", F, 2018);
    couple(&mut t, me, wife, &[k1, k2]);

    // Национальности для проверки отображения флагов (1 и 2 флага).
    for (id, codes) in [(dad, &["PL"][..]), (mom, &["PL", "DE"][..]), (me, &["PL", "DE"][..]), (sis, &["DE"][..])] {
        if let Some(p) = t.persons.get_mut(&id) {
            p.nationalities = codes.iter().map(|c| c.to_string()).collect();
        }
    }
    // Отчества и фамилия при рождении (мама — Смирнова, тётя — Иванова).
    for (id, patr, maiden) in [
        (dad, "Петрович", ""),
        (mom, "Николаевна", "Смирнова"),
        (me, "Сергеевич", ""),
        (sis, "Сергеевна", ""),
        (aunt, "Петровна", "Иванова"),
    ] {
        if let Some(p) = t.persons.get_mut(&id) {
            p.patronymic = patr.to_string();
            p.maiden_name = maiden.to_string();
        }
    }
    if let Some(p) = t.persons.get_mut(&bro) {
        p.nationalities = vec!["NG".to_string()];
    }
    t.root = Some(me);
    t
}

/// Большое дерево для проверки производительности (по 3 ребёнка у каждого).
pub fn big_tree(n: usize) -> Tree {
    const FIRST: [&str; 12] = [
        "Иван", "Пётр", "Анна", "Мария", "Олег", "Елена", "Сергей", "Ольга", "Павел", "Наталья",
        "Андрей", "Татьяна",
    ];
    const LAST: [&str; 10] = [
        "Иванов", "Петров", "Сидоров", "Смирнов", "Кузнецов", "Попов", "Орлов", "Волков", "Фёдоров",
        "Морозов",
    ];
    let n = n.max(2);
    let mut t = Tree::new();
    let mut depth: Vec<(PersonId, i32)> = Vec::new();
    let root = add(&mut t, FIRST[0], LAST[0], Gender::Male, 1850);
    depth.push((root, 0));
    let mut head = 0;
    let mut last = root;
    while t.persons.len() < n {
        let (p, d) = depth[head];
        head += 1;
        for _ in 0..3 {
            if t.persons.len() >= n {
                break;
            }
            let i = t.persons.len();
            let c = add(&mut t, FIRST[i % FIRST.len()], LAST[i % LAST.len()], Gender::Unknown, 1850 + (d + 1) * 25);
            t.add_parent_child(p, c).unwrap();
            depth.push((c, d + 1));
            last = c;
        }
    }
    t.root = Some(last);
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{compute, LayoutConfig};

    #[test]
    fn sample_tree_is_fully_placed() {
        let t = sample_tree();
        let l = compute(&t, &LayoutConfig::default());
        assert_eq!(l.nodes.len(), t.persons.len());
        assert!(l.unplaced.is_empty());
    }

    #[test]
    fn big_tree_has_requested_size() {
        let t = big_tree(500);
        assert_eq!(t.persons.len(), 500);
        let l = compute(&t, &LayoutConfig::default());
        assert_eq!(l.nodes.len(), 500);
    }
}
