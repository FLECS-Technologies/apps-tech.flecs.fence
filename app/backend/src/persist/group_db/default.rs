use std::collections::HashMap;

use crate::model::group::{Group, GroupId};

pub(super) fn default_groups() -> HashMap<GroupId, Group> {
    // TODO: Add descriptions
    let groups = vec![
        Group {
            id: GroupId::admin(),
            name: "Admin".to_string(),
            description: None,
            sub_groups: [GroupId::developer()].into(),
        },
        Group {
            id: GroupId::developer(),
            name: "Developer".to_string(),
            description: None,
            sub_groups: [GroupId::technician()].into(),
        },
        Group {
            id: GroupId::technician(),
            name: "Technician".to_string(),
            description: None,
            sub_groups: [GroupId::operator()].into(),
        },
        Group {
            id: GroupId::operator(),
            name: "Operator".to_string(),
            description: None,
            sub_groups: Default::default(),
        },
    ];
    groups
        .into_iter()
        .map(|group| (group.id.clone(), group))
        .collect()
}
