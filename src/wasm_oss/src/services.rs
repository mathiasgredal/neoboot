use crate::executor::Executor;
use log::error;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub mod boot;
pub mod console;
pub mod server;

pub trait Service {
    fn name(&self) -> &'static str;
    fn init(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn run(&self, executor: &Executor) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Default)]
pub struct ServiceRegistry {
    services: Rc<RefCell<HashMap<&'static str, Rc<dyn Service>>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn register_all(&self, services: Vec<Box<dyn Service>>) {
        for service in services {
            let result = self.register(service);
            if result.is_err() {
                log::error!("Failed to register service: {}", result.err().unwrap());
            }
        }
    }

    pub fn register(&self, service: Box<dyn Service>) -> Result<(), Box<dyn std::error::Error>> {
        let name = service.name();
        let mut services = self.services.borrow_mut();

        if services.contains_key(name) {
            return Err(format!("Service '{}' is already registered", name).into());
        }

        service.init()?;
        services.insert(name, service.into());
        Ok(())
    }

    pub fn spawn_all(&self, executor: &Executor) -> Option<Box<dyn std::error::Error>> {
        let services = self.services.borrow();

        for (name, service) in services.iter() {
            if let Err(e) = service.run(executor) {
                error!("Service '{}' failed: {}", name, e);
                return Some(e);
            }
        }

        None
    }
}
