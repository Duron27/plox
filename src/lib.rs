use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufRead;
use std::io::{self};
use std::path::Path;
use toposort_scc::IndexGraph;

pub mod expressions;
pub mod rules;

use rules::*;

////////////////////////////////////////////////////////////////////////
/// LOGIC
////////////////////////////////////////////////////////////////////////

pub fn stable_topo_sort_inner(
    n: usize,
    edges: &[(usize, usize)],
    index_dict: &HashMap<&str, usize>,
    result: &mut Vec<String>,
) -> bool {
    for i in 0..n {
        for j in 0..i {
            let x = index_dict[result[i].as_str()];
            let y = index_dict[result[j].as_str()];
            if edges.contains(&(x, y)) {
                let t = result[i].to_owned();
                result.remove(i);
                result.insert(j, t);
                return true;
            }
        }
    }
    false
}

pub fn topo_sort(
    mods: &Vec<String>,
    order: &Vec<(String, String)>,
) -> Result<Vec<String>, &'static str> {
    let mut g = IndexGraph::with_vertices(mods.len());
    let mut index_dict: HashMap<&str, usize> = HashMap::new();
    for (i, m) in mods.iter().enumerate() {
        index_dict.insert(m, i);
    }
    // add edges
    let mut edges: Vec<(usize, usize)> = vec![];
    for (a, b) in order {
        if mods.contains(a) && mods.contains(b) {
            let idx_a = index_dict[a.as_str()];
            let idx_b = index_dict[b.as_str()];
            g.add_edge(idx_a, idx_b);
            edges.push((idx_a, idx_b));
        }
    }
    // cycle check
    let sort = g.toposort();
    if sort.is_none() {
        return Err("Graph contains a cycle");
    }

    // sort
    let mut result: Vec<String> = mods.iter().map(|e| (*e).to_owned()).collect();
    println!("{result:?}");
    loop {
        if !stable_topo_sort_inner(mods.len(), &edges, &index_dict, &mut result) {
            break;
        }
    }

    // Return the sorted vector
    Ok(result)
}

pub fn parse_rules_from_dir<P>(rules_dir: P) -> io::Result<Vec<RuleKind>>
where
    P: AsRef<Path>,
{
    let rules_path = rules_dir.as_ref().join("cmop_rules_base.txt");
    parse_rules(rules_path)
}

/// custom rules parser
///
/// # Errors
///
/// This function will return an error if .
pub fn parse_rules<P>(rules_path: P) -> io::Result<Vec<RuleKind>>
where
    P: AsRef<Path>,
{
    let mut rules: Vec<RuleKind> = vec![];

    // helpers for order rule
    let mut orders: Vec<Vec<String>> = vec![];
    let mut current_order: Vec<String> = vec![];

    // todo scan directory for user files
    let lines = read_lines(rules_path)?;
    let mut parsing = false;
    let mut current_rule: Option<RuleKind> = None;

    // parse each line
    for line in lines.flatten() {
        // comments
        if line.starts_with(';') {
            continue;
        }

        // HANDLE RULE END
        // new empty lines end a rule block
        if parsing && line.is_empty() {
            parsing = false;
            if let Some(rule) = current_rule.take() {
                // Order rule is handled separately
                if let RuleKind::Order(_o) = rule {
                    orders.push(current_order.to_owned());
                    current_order.clear();
                } else {
                    rules.push(rule);
                }
            } else {
                // error and abort
                panic!("Parsing error: unknown empty new line");
            }
            continue;
        }

        // HANDLE RULE START
        // start order parsing
        let mut r_line = line;
        if !parsing {
            if r_line.starts_with("[Order") {
                current_rule = Some(RuleKind::Order(Order::default()));
                r_line = r_line["[Order".len()..].to_owned();
            } else if r_line.starts_with("[Note") {
                current_rule = Some(RuleKind::Note(Note::default()));
                r_line = r_line["[Note".len()..].to_owned();
            } else if r_line.starts_with("[Conflict") {
                current_rule = Some(RuleKind::Conflict(Conflict::default()));
                r_line = r_line["[Conflict".len()..].to_owned();
            } else if r_line.starts_with("[Requires") {
                current_rule = Some(RuleKind::Requires(Requires::default()));
                r_line = r_line["[Requires".len()..].to_owned();
            } else {
                // unknown rule
                panic!("Parsing error: unknown rule");
            }
            parsing = true;
        }

        // HANDLE RULE PARSE
        // parse current rule
        if parsing {
            if let Some(current_rule) = &current_rule {
                match current_rule {
                    RuleKind::Order(_o) => {
                        // order is just a list of names
                        // TODO in-line names?
                        current_order.push(r_line)
                    }
                    RuleKind::Note(_n) => {
                        // parse rule
                        // Syntax: [Note optional-message] expr-1 expr-2 ... expr-N
                        // TODO alternative:
                        // [Note]
                        //  message
                        // A.esp

                        // subsequent lines are archive names

                        // parse expressions

                        todo!()
                    }
                    RuleKind::Conflict(_c) => {
                        todo!()
                    }
                    RuleKind::Requires(_r) => {
                        todo!()
                    }
                }
            }
        }
    }
    orders.push(current_order.to_owned());

    // process order rules
    for o in orders {
        match o.len().cmp(&2) {
            Ordering::Less => continue,
            Ordering::Equal => rules.push(RuleKind::Order(Order::new(
                o[0].to_owned(),
                o[1].to_owned(),
            ))),
            Ordering::Greater => {
                // add all pairs
                for i in 0..o.len() - 1 {
                    rules.push(RuleKind::Order(Order::new(
                        o[i].to_owned(),
                        o[i + 1].to_owned(),
                    )));
                }
            }
        }
    }

    Ok(rules)
}

pub fn get_mods_from_rules(order: &[(String, String)]) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for r in order.iter() {
        let mut a = r.0.to_owned();
        if !result.contains(&a) {
            result.push(a);
        }
        a = r.1.to_owned();
        if !result.contains(&a) {
            result.push(a);
        }
    }
    result
}

pub fn gather_mods<P>(root: &P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    // gather mods from archive/pc/mod
    let archive_path = root.as_ref().join("archive").join("pc").join("mod");
    let mut entries = fs::read_dir(archive_path)?
        .map(|res| res.map(|e| e.path()))
        .filter_map(Result::ok)
        .filter_map(|e| {
            if !e.is_dir() {
                if let Some(os_ext) = e.extension() {
                    if let Some(ext) = os_ext.to_ascii_lowercase().to_str() {
                        if ext.contains("archive") {
                            if let Some(file_name) = e.file_name().and_then(|n| n.to_str()) {
                                return Some(file_name.to_owned());
                            }
                        }
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>();

    // TODO gather REDmods from mods/<NAME>
    entries.sort();

    Ok(entries)
}

////////////////////////////////////////////////////////////////////////
/// HELPERS
////////////////////////////////////////////////////////////////////////

pub fn get_order_from_rules(rules: &Vec<RuleKind>) -> Vec<(String, String)> {
    let mut order: Vec<(String, String)> = vec![];
    for r in rules {
        if let RuleKind::Order(o) = r {
            order.push((o.name_a.to_owned(), o.name_b.to_owned()));
        }
    }

    order
}

// Returns an Iterator to the Reader of the lines of the file.
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

// read file line by line into vector
pub fn read_file_as_list<P>(modlist_path: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let mut result: Vec<String> = vec![];
    if let Ok(lines) = read_lines(modlist_path) {
        for line in lines.flatten() {
            result.push(line);
        }
    }
    result
}
