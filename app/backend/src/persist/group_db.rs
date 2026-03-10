use crate::model::group::{Group, GroupId};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use tracing::error;

mod default;
mod versioning;

pub struct GroupDB {
    path: PathBuf,
    groups: HashMap<GroupId, Group>,
}

impl GroupDB {
    pub(super) fn new(path: PathBuf) -> anyhow::Result<Self> {
        let groups: versioning::GroupStorage = super::load_from_file(path.as_path())?;
        let groups = groups.into();
        Ok(GroupDB { path, groups })
    }

    pub fn query_groups_with_subgroups(&self, groups: &[GroupId]) -> HashSet<GroupId> {
        let mut stack: Vec<_> = groups
            .iter()
            .filter_map(|group| self.groups.get(group).map(|group| group.id.clone()))
            .collect();
        let mut groups = HashSet::from_iter(stack.iter().cloned());
        while let Some(group_id) = stack.pop() {
            let Some(group) = self.groups.get(&group_id) else {
                continue;
            };
            for sub_group in &group.sub_groups {
                if self.groups.contains_key(sub_group) && groups.insert(sub_group.clone()) {
                    stack.push(sub_group.clone());
                }
            }
        }

        groups
    }
}

impl Drop for GroupDB {
    fn drop(&mut self) {
        super::save_to_file(&self.path, &versioning::StorageRef::new(&self.groups))
            .unwrap_or_else(|e| error!("Could not persist group database: {e}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::group::Group;

    fn make_group(id: GroupId, sub_groups: Vec<GroupId>) -> Group {
        Group {
            id: id.clone(),
            name: id.as_ref().to_string(),
            description: None,
            sub_groups: HashSet::from_iter(sub_groups),
        }
    }

    fn make_db(groups: Vec<Group>) -> GroupDB {
        let map = groups.into_iter().map(|g| (g.id.clone(), g)).collect();
        GroupDB {
            path: PathBuf::new(),
            groups: map,
        }
    }

    #[test]
    fn empty_input_returns_empty() {
        let db = make_db(vec![make_group(GroupId::admin(), vec![])]);
        let result = db.query_groups_with_subgroups(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn unknown_group_is_excluded() {
        let db = make_db(vec![make_group(GroupId::admin(), vec![])]);
        let result = db.query_groups_with_subgroups(&[GroupId::from("nonexistent".to_string())]);
        assert!(result.is_empty());
    }

    #[test]
    fn single_group_no_subgroups() {
        let db = make_db(vec![make_group(GroupId::admin(), vec![])]);
        let result = db.query_groups_with_subgroups(&[GroupId::admin()]);
        assert_eq!(result, HashSet::from([GroupId::admin()]));
    }

    #[test]
    fn single_group_with_subgroups() {
        let db = make_db(vec![
            make_group(GroupId::admin(), vec![GroupId::developer()]),
            make_group(GroupId::developer(), vec![]),
        ]);
        let result = db.query_groups_with_subgroups(&[GroupId::admin()]);
        assert_eq!(
            result,
            HashSet::from([GroupId::admin(), GroupId::developer()])
        );
    }

    #[test]
    fn nested_subgroups_are_resolved() {
        let db = make_db(vec![
            make_group(GroupId::admin(), vec![GroupId::developer()]),
            make_group(GroupId::developer(), vec![GroupId::technician()]),
            make_group(GroupId::technician(), vec![]),
        ]);
        let result = db.query_groups_with_subgroups(&[GroupId::admin()]);
        assert_eq!(
            result,
            HashSet::from([
                GroupId::admin(),
                GroupId::developer(),
                GroupId::technician()
            ])
        );
    }

    #[test]
    fn cyclic_subgroups_do_not_loop() {
        let db = make_db(vec![
            make_group(GroupId::admin(), vec![GroupId::developer()]),
            make_group(GroupId::developer(), vec![GroupId::admin()]),
        ]);
        let result = db.query_groups_with_subgroups(&[GroupId::admin()]);
        assert_eq!(
            result,
            HashSet::from([GroupId::admin(), GroupId::developer()])
        );
    }

    #[test]
    fn multiple_input_groups() {
        let db = make_db(vec![
            make_group(GroupId::admin(), vec![GroupId::developer()]),
            make_group(GroupId::developer(), vec![]),
            make_group(GroupId::operator(), vec![GroupId::technician()]),
            make_group(GroupId::technician(), vec![]),
        ]);
        let result = db.query_groups_with_subgroups(&[GroupId::admin(), GroupId::operator()]);
        assert_eq!(
            result,
            HashSet::from([
                GroupId::admin(),
                GroupId::developer(),
                GroupId::operator(),
                GroupId::technician()
            ])
        );
    }

    #[test]
    fn overlapping_subgroups_across_inputs() {
        let db = make_db(vec![
            make_group(GroupId::admin(), vec![GroupId::technician()]),
            make_group(GroupId::developer(), vec![GroupId::technician()]),
            make_group(GroupId::technician(), vec![]),
        ]);
        let result = db.query_groups_with_subgroups(&[GroupId::admin(), GroupId::developer()]);
        assert_eq!(
            result,
            HashSet::from([
                GroupId::admin(),
                GroupId::developer(),
                GroupId::technician()
            ])
        );
    }

    #[test]
    fn unknown_subgroup_not_in_db_is_not_included() {
        let unknown = GroupId::from("unknown".to_string());
        let db = make_db(vec![make_group(GroupId::admin(), vec![unknown])]);
        let result = db.query_groups_with_subgroups(&[GroupId::admin()]);
        assert_eq!(result, HashSet::from([GroupId::admin()]));
    }
}
