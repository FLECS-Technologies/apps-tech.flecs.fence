use std::collections::HashMap;

use crate::model::group::{Group, GroupId};

pub(super) fn default_groups() -> HashMap<GroupId, Group> {
    // TODO: Add descriptions
    let groups = vec![
        Group {
            id: GroupId::admin(),
            name: "Admin".to_string(),
            description: None,
            sub_groups: [GroupId::developer(), GroupId::core_admin()].into(),
        },
        Group {
            id: GroupId::developer(),
            name: "Developer".to_string(),
            description: None,
            sub_groups: [GroupId::technician(), GroupId::core_developer()].into(),
        },
        Group {
            id: GroupId::technician(),
            name: "Technician".to_string(),
            description: None,
            sub_groups: [GroupId::operator(), GroupId::core_technician()].into(),
        },
        Group {
            id: GroupId::operator(),
            name: "Operator".to_string(),
            description: None,
            sub_groups: [GroupId::core_operator()].into(),
        },
        Group {
            id: GroupId::core_admin(),
            name: "Core Admin".to_string(),
            description: Some("Backwards compatibility with flecs core 5.2 and below".to_string()),
            sub_groups: Default::default(),
        },
        Group {
            id: GroupId::core_developer(),
            name: "Core Developer".to_string(),
            description: Some("Backwards compatibility with flecs core 5.2 and below".to_string()),
            sub_groups: Default::default(),
        },
        Group {
            id: GroupId::core_technician(),
            name: "Core Technician".to_string(),
            description: Some("Backwards compatibility with flecs core 5.2 and below".to_string()),
            sub_groups: Default::default(),
        },
        Group {
            id: GroupId::core_operator(),
            name: "Core Operator".to_string(),
            description: Some("Backwards compatibility with flecs core 5.2 and below".to_string()),
            sub_groups: Default::default(),
        },
    ];
    groups
        .into_iter()
        .map(|group| (group.id.clone(), group))
        .collect()
}
