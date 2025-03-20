use crate::commands::CommandDispatcher;
use crate::executor::Executor;
use crate::util::sys_print;
use crate::{asyncio::get_keypress, errors::lwip_error::LwipError};
use futures::{
    future::{select, Either},
    FutureExt,
};
use futures_lite::future::yield_now;
use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc};

pub struct ConsoleService<'a> {
    dispatcher: Rc<RefCell<CommandDispatcher<'a>>>,
    input_buffer: String,
}

impl ConsoleService<'_> {
    async fn process_input(&self, input: &str) {
        if input.trim().is_empty() {
            return;
        }

        let response = self.dispatcher.borrow().execute(input).await;
        if response.is_ok() {
            sys_print(&format!("{}\n", response.unwrap()));
        } else {
            sys_print(&format!("Error: {:?}\n", response.err().unwrap()));
        }
    }
}

impl<'a> super::Service<'a> for ConsoleService<'a> {
    fn name(&self) -> &'static str {
        "console"
    }

    fn run(mut self: Box<Self>, executor: Executor<'a>) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async move {
            sys_print("\nConsole ready. Type 'help' for available commands.\n> ");
            loop {
                let key =
                    match select(get_keypress().boxed(), executor.wait_for_exit().boxed()).await {
                        Either::Left((key, _)) => Ok(key),
                        Either::Right((_, _)) => Err(LwipError::ConnectionAborted),
                    };

                if key.is_err() {
                    return;
                }

                let key = key.unwrap();

                match key {
                    13 => {
                        // Enter
                        sys_print("\n");
                        self.process_input(&self.input_buffer).await;
                        self.input_buffer.clear();
                        self.dispatcher
                            .borrow()
                            .finalize_shutdown_if_requested(&executor);
                        yield_now().await;
                        sys_print("> ");
                    }
                    32..=126 => {
                        // Printable ASCII
                        self.input_buffer.push(char::from_u32(key as u32).unwrap());
                        sys_print(&char::from_u32(key as u32).unwrap().to_string());
                    }
                    127 => {
                        // Backspace
                        if !self.input_buffer.is_empty() {
                            self.input_buffer.pop();
                            sys_print("\x08 \x08");
                        }
                    }
                    _ => {}
                }
            }
        })
    }
}

impl<'a> ConsoleService<'a> {
    pub fn new(dispatcher: Rc<RefCell<CommandDispatcher<'a>>>) -> Self {
        Self {
            dispatcher,
            input_buffer: String::new(),
        }
    }
}
