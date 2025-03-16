use crate::asyncio::get_keypress;
use crate::commands::CommandDispatcher;
use crate::executor::Executor;
use crate::util::sys_print;
use futures_lite::future::yield_now;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct Console {
    dispatcher: CommandDispatcher,
    input_buffer: Rc<RefCell<String>>,
}

impl Console {
    pub fn new(dispatcher: CommandDispatcher) -> Self {
        Self {
            dispatcher,
            input_buffer: Rc::new(RefCell::new(String::new())),
        }
    }

    fn process_input(&self, input: &str) {
        if input.trim().is_empty() {
            return;
        }

        let response = self.dispatcher.execute(input);
        if response.is_ok() {
            sys_print(&format!("{}\n", response.unwrap()));
        } else {
            sys_print(&format!("Error: {:?}\n", response.err().unwrap()));
        }
    }

    pub async fn run_loop(&self) {
        sys_print("\nConsole ready. Type 'help' for available commands.\n> ");

        loop {
            let key = get_keypress().await;
            let mut input = self.input_buffer.borrow_mut();

            match key {
                13 => {
                    // Enter
                    sys_print("\n");
                    let command = input.clone();
                    input.clear();
                    self.process_input(&command);
                    yield_now().await;
                    sys_print("> ");
                }
                32..=126 => {
                    // Printable ASCII
                    input.push(char::from_u32(key as u32).unwrap());
                    sys_print(&char::from_u32(key as u32).unwrap().to_string());
                }
                127 => {
                    // Backspace
                    if !input.is_empty() {
                        input.pop();
                        sys_print("\x08 \x08");
                    }
                }
                _ => {}
            }
        }
    }
}

pub struct ConsoleService {
    console: Console,
}

impl super::Service for ConsoleService {
    fn name(&self) -> &'static str {
        "console"
    }

    fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn run(&self, executor: &Executor) -> Result<(), Box<dyn std::error::Error>> {
        let console = self.console.clone();
        executor.spawn(async move {
            console.run_loop().await;
        });
        Ok(())
    }
}

impl ConsoleService {
    pub fn new(dispatcher: CommandDispatcher) -> Self {
        Self {
            console: Console::new(dispatcher),
        }
    }
}
