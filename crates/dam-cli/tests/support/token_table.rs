pub struct Grant {
    allow: Vec<String>,
    limits: Vec<(&'static str, String)>,
}

impl Grant {
    pub fn allowing(permissions: &[&str]) -> Grant {
        Grant {
            allow: permissions.iter().map(|p| p.to_string()).collect(),
            limits: Vec::new(),
        }
    }

    pub fn paths(self, paths: &[&str]) -> Grant {
        self.limited("paths", list(paths))
    }

    pub fn labels(self, labels: &[&str]) -> Grant {
        self.limited("labels", list(labels))
    }

    pub fn remotes(self, remotes: &[&str]) -> Grant {
        self.limited("remotes", list(remotes))
    }

    pub fn own(self) -> Grant {
        self.limited("own", "true".to_string())
    }

    fn limited(mut self, key: &'static str, value: String) -> Grant {
        self.limits.push((key, value));
        self
    }

    fn lua(&self) -> String {
        let mut fields = vec![format!("allow = {}", list(&self.allow))];
        fields.extend(self.limits.iter().map(|(k, v)| format!("{k} = {v}")));
        format!("{{ {} }}", fields.join(", "))
    }
}

#[derive(Default)]
pub struct TokenTable {
    fields: Vec<(&'static str, String)>,
    grants: Vec<Grant>,
}

impl TokenTable {
    pub fn valued(value: &str) -> TokenTable {
        TokenTable::without_value().field("value", text(value))
    }

    pub fn without_value() -> TokenTable {
        TokenTable::default()
    }

    pub fn expires(self, at: &str) -> TokenTable {
        self.field("expires", text(at))
    }

    pub fn not_before(self, at: &str) -> TokenTable {
        self.field("not_before", text(at))
    }

    pub fn via(self, channels: &[&str]) -> TokenTable {
        self.field("via", list(channels))
    }

    pub fn require(self, details: &[&str]) -> TokenTable {
        self.field("require", list(details))
    }

    pub fn grant(mut self, grant: Grant) -> TokenTable {
        self.grants.push(grant);
        self
    }

    pub fn grants(self, grants: Vec<Grant>) -> TokenTable {
        grants.into_iter().fold(self, TokenTable::grant)
    }

    fn field(mut self, key: &'static str, value: String) -> TokenTable {
        self.fields.push((key, value));
        self
    }

    pub(super) fn lua(&self) -> String {
        let mut fields: Vec<String> = self
            .fields
            .iter()
            .map(|(k, v)| format!("{k} = {v}"))
            .collect();
        if !self.grants.is_empty() {
            let grants: Vec<String> = self.grants.iter().map(Grant::lua).collect();
            fields.push(format!("grant = {{ {} }}", grants.join(", ")));
        }
        format!("{{ {} }}", fields.join(", "))
    }
}

pub(super) fn text(value: &str) -> String {
    format!("{value:?}")
}

pub(super) fn list(items: &[impl AsRef<str>]) -> String {
    let items: Vec<String> = items.iter().map(|i| text(i.as_ref())).collect();
    format!("{{ {} }}", items.join(", "))
}
