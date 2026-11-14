use dam_protocol::Mutation;

#[derive(Default)]
pub struct OffLimits {
    unanswered: Vec<Unanswered>,
    call_ended_by: Option<String>,
}

struct Unanswered {
    oid: String,
    remote_id: Option<String>,
    folder: Option<String>,
    name: String,
}

impl OffLimits {
    pub fn why_left_out(&self, m: &Mutation) -> Option<String> {
        if let Some(why) = &self.call_ended_by {
            return Some(why.clone());
        }
        let waited_on = self.unanswered.iter().find(|u| u.holds(m))?;
        Some(format!(
            "waits on {}, which Todoist left unanswered",
            waited_on.name
        ))
    }

    pub fn unanswered(&mut self, m: &Mutation) {
        let object = m.object.as_ref();
        self.unanswered.push(Unanswered {
            oid: m.oid.clone(),
            remote_id: m.remote_id.clone(),
            folder: object.map(|o| format!("{}{}/", o.path, o.subject)),
            name: object
                .map(|o| format!("{}{}", o.path, o.subject))
                .or_else(|| m.remote_id.clone())
                .unwrap_or_else(|| m.oid.clone()),
        });
    }

    pub fn end_the_call(&mut self, why: &str) {
        self.call_ended_by = Some(why.to_string());
    }
}

impl Unanswered {
    fn holds(&self, m: &Mutation) -> bool {
        let removes_it = m.op == "delete" && m.remote_id.is_some() && m.remote_id == self.remote_id;
        let same_object = m.oid == self.oid || removes_it;
        let inside = match (&self.folder, &m.object) {
            (Some(folder), Some(object)) => object.path.starts_with(folder.as_str()),
            _ => false,
        };
        same_object || inside
    }
}
