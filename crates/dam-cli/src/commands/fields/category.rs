use dam_domain::Categories;

use crate::error::CliError;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct CategoryLabels {
    pub(crate) add: Vec<String>,
    pub(crate) remove: Vec<String>,
}

pub(crate) fn category_labels(
    categories: &Categories,
    asked: &[(String, String)],
) -> Result<CategoryLabels, CliError> {
    let mut labels = CategoryLabels::default();
    for (group, label) in asked {
        let refused = |why: String| CliError::Usage(format!("--category {group}={label}: {why}"));
        let Some(category) = categories.category_of_name(group) else {
            let groups: Vec<&str> = categories.all().iter().map(|c| c.name.as_str()).collect();
            return Err(refused(match groups.is_empty() {
                true => "no label group is declared; dam category create declares one".into(),
                false => format!(
                    "that is not a label group; the groups are {}; dam category list shows their labels",
                    groups.join(", ")
                ),
            }));
        };
        match label.as_str() {
            "none" => labels.remove.extend(category.values.iter().cloned()),
            _ if category.values.contains(label) => labels.add.push(label.clone()),
            _ => {
                return Err(refused(format!(
                    "the group {group} holds only {}; give one of them, such as --category {group}={}",
                    category.values.join(", "),
                    category.values[0]
                )));
            }
        }
    }
    Ok(labels)
}
