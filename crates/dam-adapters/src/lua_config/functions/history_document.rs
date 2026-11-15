use dam_domain::{Authored, CommitRecord, Details, Field, Object, When, touched_fields};
use serde_json::{Value, json};

pub fn history_document(commits: &[CommitRecord], owner: &str) -> Value {
    commits
        .iter()
        .map(|commit| {
            json!({
                "id": commit.id.to_string(),
                "at": commit.at.to_string(),
                "message": commit.message,
                "changes": commit.changes.iter().map(|c| change_row(c, owner)).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn change_row(authored: &Authored, owner: &str) -> Value {
    let change = &authored.change;
    let object = change.object();
    let base = object.base();
    let token = authored.author.token();
    let mut row = json!({
        "oid": change.oid().to_string(),
        "op": change.op().as_str(),
        "fields": touched_fields(change).iter().map(Field::as_str).collect::<Vec<_>>(),
        "kind": object.kind().as_str(),
        "subject": base.subject,
        "path": base.path.as_str(),
        "labels": base.labels,
        "from": token.map(|t| t.name.as_str()),
        "key_id": token.map(|t| t.key_id.as_str()),
        "channel": token.map(|t| t.channel.as_str()),
        "author": token.map_or(owner, |t| t.name.as_str()),
        "details": authored.details.as_ref().map(details),
    });
    if let (Value::Object(row), Value::Object(state)) = (&mut row, state(object)) {
        row.extend(state);
    }
    row
}

fn state(object: &Object) -> Value {
    match object {
        Object::Task(t) => json!({
            "done": t.completion.is_done(),
            "completed_at": t.completion.at().map(|at| at.to_string()),
            "priority": t.priority.get(),
            "due": t.due.as_ref().map(When::to_rfc3339),
        }),
        Object::Event(e) => json!({
            "start": e.start.to_rfc3339(),
            "end": e.end.to_rfc3339(),
        }),
    }
}

fn details(details: &Details) -> Value {
    json!({
        "meta": details.meta,
        "host": details.host,
        "dir": details.dir,
        "repo": details.repo,
        "branch": details.branch,
        "at": details.at.to_string(),
        "channel": details.channel.as_str(),
    })
}
