use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use dam_domain::{Object, Task, merge_fields};

use crate::ports::ObjectRepository;
use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, RemoteMutation};
use crate::testing::{
    FaultyStore, FixedRandom, Fixture, MemoryStore, config, oid, pushing, remotes_launched_by, task,
};

#[derive(Default)]
struct Upstream {
    ran: BTreeMap<String, String>,
    held: BTreeMap<String, Object>,
}

impl Upstream {
    fn run_once(&mut self, m: &RemoteMutation) -> String {
        if let Some(id) = self.ran.get(&m.idempotency_key) {
            return id.clone();
        }
        let id = m.remote_id.clone();
        let id = match (m.op, &m.object) {
            (MutationOp::Create, Some(object)) => {
                let id = format!("r{}", self.ran.len() + 1);
                self.held.insert(id.clone(), object.clone());
                id
            }
            (MutationOp::Update, Some(sent)) => {
                let id = id.unwrap_or_default();
                let held = &self.held[&id];
                self.held
                    .insert(id.clone(), merge_fields(held, sent, &m.fields));
                id
            }
            _ => id.unwrap_or_default(),
        };
        self.ran.insert(m.idempotency_key.clone(), id.clone());
        id
    }
}

fn running_each_key_once(
    upstream: Rc<RefCell<Upstream>>,
) -> impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone + 'static {
    move |ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                verdict: MutationVerdict::Accepted {
                    remote_id: Some(upstream.borrow_mut().run_once(m)),
                },
            })
            .collect()
    }
}

fn commit(store: &MemoryStore, object: Object) {
    let arranging = Fixture {
        random: FixedRandom::new(200),
        ..Fixture::default()
    };
    store.put(&object).unwrap();
    arranging.dam(store).add_all().unwrap();
    arranging.dam(store).commit("m").unwrap();
}

#[test]
fn a_create_applied_whose_settlement_failed_lands_once_with_the_edit_made_after() {
    let store = FaultyStore::new(MemoryStore::new());
    let fixture = Fixture {
        config: config(),
        ..Fixture::default()
    };
    commit(&store.inner, task(1));
    let upstream = Rc::new(RefCell::new(Upstream::default()));
    let (l, _) = pushing(running_each_key_once(upstream.clone()));
    store.fail_at_write(1);
    let failed = fixture.dam(&store).push(remotes_launched_by(&l), None);
    assert!(failed.is_err(), "the settlement write failed: {failed:?}");
    store.fail_at_write(usize::MAX);

    commit(&store.inner, Object::Task(Task::new(oid(1), "edited")));
    fixture
        .dam(&store)
        .push(remotes_launched_by(&l), None)
        .unwrap();

    let upstream = upstream.borrow();
    let held: Vec<&str> = upstream
        .held
        .values()
        .map(|o| o.base().subject.as_str())
        .collect();
    assert_eq!(held, vec!["edited"]);
}
