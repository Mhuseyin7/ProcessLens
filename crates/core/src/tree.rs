use crate::{ProcessInfo, ProcessTreeNode};
use std::collections::{HashMap, HashSet};

pub fn build_process_tree(processes: &[ProcessInfo], root_pid: u32) -> Option<ProcessTreeNode> {
    let map: HashMap<u32, &ProcessInfo> = processes.iter().map(|p| (p.pid, p)).collect();
    fn visit(
        pid: u32,
        map: &HashMap<u32, &ProcessInfo>,
        seen: &mut HashSet<u32>,
    ) -> Option<ProcessTreeNode> {
        if !seen.insert(pid) {
            return None;
        }
        let process = (*map.get(&pid)?).clone();
        let mut children: Vec<_> = map
            .values()
            .filter(|p| p.parent_pid == Some(pid))
            .filter_map(|p| visit(p.pid, map, seen))
            .collect();
        children.sort_by_key(|node| node.process.pid);
        Some(ProcessTreeNode { process, children })
    }
    visit(root_pid, &map, &mut HashSet::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Availability;
    fn p(pid: u32, parent: Option<u32>) -> ProcessInfo {
        ProcessInfo {
            pid,
            parent_pid: parent,
            name: pid.to_string(),
            executable: Availability::Unavailable { reason: "x".into() },
            command: Availability::Unavailable { reason: "x".into() },
            working_directory: Availability::Unavailable { reason: "x".into() },
            start_time_unix: 0,
            memory_bytes: 0,
            status: "run".into(),
        }
    }
    #[test]
    fn builds_and_breaks_cycles() {
        let ps = vec![p(1, None), p(2, Some(1)), p(3, Some(2)), p(1, Some(3))];
        let t = build_process_tree(&ps, 1).unwrap();
        assert_eq!(t.children[0].process.pid, 2);
    }
}
