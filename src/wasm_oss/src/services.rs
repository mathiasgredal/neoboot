use crate::executor::Executor;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

pub mod boot;
pub mod console;
pub mod server;

pub trait Service<'a> {
    fn name(&self) -> &'static str;
    fn run(self: Box<Self>, executor: Executor<'a>) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
}

#[derive(Default)]
pub struct ServiceRegistry<'a> {
    services: HashMap<&'static str, Box<dyn Service<'a> + 'a>>,
}

impl<'a> ServiceRegistry<'a> {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register(&mut self, service: impl Service<'a> + 'a) {
        let name = service.name();

        if self.services.contains_key(name) {
            log::error!("Service '{}' is already registered", name);
        }

        self.services.insert(name, Box::new(service));
    }

    pub fn spawn_all(self, executor: &Executor<'a>) {
        let services = self.services;
        let services = services.into_values();

        for service in services {
            let run_fut = service.run(executor.clone());
            executor.spawn(run_fut);
        }
    }
}
