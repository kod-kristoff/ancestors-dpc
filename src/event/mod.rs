#[derive(Debug, Clone)]
pub enum Event {
    Person(PersonEvent),
}
#[derive(Debug, Clone)]
pub enum PersonEvent {
    PersonAdded { name: String },
}
