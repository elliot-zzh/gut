use std::process::exit;

pub fn infer_subcommand(input: &str, target: &str) -> bool {
    if target.starts_with(input) { return true; }
    levenshtein(input, target) <= 1
}

fn levenshtein(a: &str, b: &str) -> usize {
    let mut costs = vec![0; b.len() + 1];
    for j in 0..=b.len() { costs[j] = j; }
    for (i, ca) in a.chars().enumerate() {
        let mut last = i;
        costs[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let old = costs[j + 1];
            costs[j + 1] = std::cmp::min(
                std::cmp::min(costs[j] + 1, costs[j + 1] + 1),
                last + if ca == cb { 0 } else { 1 }
            );
            last = old;
        }
    }
    costs[b.len()]
}

pub fn pass_to_git(args: &[String]) {
    let status = std::process::Command::new("git").args(args).status().expect("failed to run git");
    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
}
