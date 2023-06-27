mod error;
mod views;

pub use self::error::TuiError;
pub type TuiResult<T> = Result<T, TuiError>;

use super::LoopService;
use crate::{event, event_log};

#[derive(Debug)]
pub struct AddPersonRequest {
    name: String,
}
pub struct Tui {
    event_writer: event_log::SharedWriter,
}

impl Tui {
    pub fn new(event_writer: event_log::SharedWriter) -> Self {
        Self { event_writer }
    }

    fn add_person(&self, add_person_request: AddPersonRequest) -> TuiResult<String> {
        self.event_writer
            .write(&[event::Event::Person(event::PersonEvent::PersonAdded {
                name: add_person_request.name,
            })])
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
                println!("person with id '{}' added", id);
            }
            cmd => todo!("handle unknown command '{}'", cmd),
        }
        Ok(())
    }
}
