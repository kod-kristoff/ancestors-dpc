mod error;
mod views;

pub use self::error::TuiError;
pub type TuiResult<T> = Result<T, TuiError>;

use super::LoopService;
use crate::{
    event::{self, PersonId},
    event_log,
    persistence::SharedPersistence,
};

#[derive(Debug)]
pub struct AddPersonRequest {
    name: String,
}
pub struct Tui {
    persistence: SharedPersistence,
    event_writer: event_log::SharedWriter,
}

impl Tui {
    pub fn new(persistence: SharedPersistence, event_writer: event_log::SharedWriter) -> Self {
        Self {
            persistence,
            event_writer,
        }
    }

    fn add_person(&self, add_person_request: AddPersonRequest) -> TuiResult<PersonId> {
        let new_id = PersonId::new();
        self.event_writer
            .write(
                &mut *self.persistence.get_connection().expect("a connection"),
                &[event::Event::Person(event::PersonEvent::PersonAdded {
                    id: new_id.clone(),
                    name: add_person_request.name,
                })],
            )
            .expect("write succeed");
        Ok(new_id)
    }
}

impl LoopService for Tui {
    fn run_iteration(&mut self) -> eyre::Result<()> {
        // don't hog the cpu
        // std::thread::sleep(std::time::Duration::from_millis(100));
        let user_input = views::readline_with_prompt("ancestors> ")?;
        match user_input.as_str() {
            "add" => {
                let name = views::readline_with_prompt("name? ")?;
                let new_id = self.add_person(AddPersonRequest { name })?;
                println!("person with id '{}' added", new_id);
            }
            cmd => todo!("handle unknown command '{}'", cmd),
        }
        Ok(())
    }
}
