use dam_domain::{Event, Object, Oid, Path, Task, When};
use jiff::SignedDuration;
use jiff::civil::date;

use crate::config::{Config, CredentialSpec, RemoteConfig};
use crate::ports::{
    Clock, CommitRepository, ObjectRepository, RemoteName, RemoteTrackingRepository,
};
use crate::service::Dam;
use crate::testing::{FixedRandom, Fixture, MemoryStore, oid};

pub fn todoist() -> RemoteName {
    RemoteName("todoist".into())
}

pub fn config() -> Config {
    Config {
        remotes: vec![RemoteConfig {
            name: todoist(),
            helper: "todoist".into(),
            url: "todoist::".into(),
            credentials: vec![CredentialSpec::Literal {
                name: "api_token".into(),
                value: "t".into(),
            }],
            stale: None,
            deadline: None,
            path: None,
        }],
        ..Config::default()
    }
}

pub fn task(byte: u8) -> Object {
    Object::Task(Task::new(oid(byte), format!("t{byte}")))
}

pub struct Scenario {
    pub store: MemoryStore,
    pub fixture: Fixture,
    arranging: Fixture,
}

impl Default for Scenario {
    fn default() -> Scenario {
        Scenario {
            store: MemoryStore::new(),
            fixture: Fixture {
                config: config(),
                ..Fixture::default()
            },
            arranging: Fixture {
                random: FixedRandom::new(200),
                ..Fixture::default()
            },
        }
    }
}

impl Scenario {
    pub fn drawing_from(mut self, byte: u8) -> Scenario {
        self.fixture.random = FixedRandom::new(byte);
        self
    }

    pub fn configured(mut self, config: Config) -> Scenario {
        self.fixture.config = config;
        self
    }

    pub fn dam(&self) -> Dam<'_> {
        self.fixture.dam(&self.store)
    }

    pub fn working(&self, object: Object) -> &Scenario {
        self.store.put(&object).unwrap();
        self
    }

    pub fn committed(&self, object: Object) -> &Scenario {
        self.working(object);
        self.commit_everything()
    }

    pub fn deleted(&self, oid: &Oid) -> &Scenario {
        self.store.delete(oid).unwrap();
        self.commit_everything()
    }

    pub fn pushed(&self) -> &Scenario {
        for commit in self.store.unpushed(&todoist()).unwrap() {
            self.store.mark_pushed(&todoist(), &commit.id).unwrap();
        }
        self
    }

    pub fn tracked_as(&self, oid: &Oid, remote_id: &str) -> &Scenario {
        self.store
            .map_remote_id(&todoist(), oid, remote_id)
            .unwrap();
        let snapshot = self.store.committed(oid).unwrap().unwrap();
        self.store
            .set_remote_snapshot(&todoist(), &snapshot)
            .unwrap();
        self
    }

    pub fn pulled_ago(&self, ago: SignedDuration) -> &Scenario {
        let at = self.fixture.clock.now().checked_sub(ago).unwrap();
        let remote = &self.fixture.config.remotes[0].name;
        self.store.set_last_pull(remote, at).unwrap();
        self
    }

    pub fn mapped_as(&self, oid: &Oid, remote_id: &str) -> &Scenario {
        self.store
            .map_remote_id(&todoist(), oid, remote_id)
            .unwrap();
        self
    }

    fn commit_everything(&self) -> &Scenario {
        let dam = self.arranging.dam(&self.store);
        dam.add_all().unwrap();
        dam.commit("m").unwrap();
        self
    }
}

pub fn event(byte: u8) -> Object {
    Object::Event(Event::new(
        oid(byte),
        format!("e{byte}"),
        When::Day(date(2026, 1, 1)),
        When::Day(date(2026, 1, 2)),
    ))
}

pub fn task_in(byte: u8, subject: &str, path: &str) -> Task {
    let mut t = Task::new(oid(byte), subject);
    t.base.path = Path::parse(path).unwrap();
    t
}

impl Scenario {
    pub fn filed(&self, task: Task) -> &Scenario {
        self.working(Object::Task(task))
    }

    pub fn path_of(&self, oid: &Oid) -> String {
        let object = self.store.get(oid).unwrap().unwrap();
        object.base().path.as_str().to_string()
    }
}
