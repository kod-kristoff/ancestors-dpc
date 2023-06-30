#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Person(PersonEvent),
}

pub type PersonId = ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonEvent {
    PersonAdded { id: PersonId, name: String },
}
