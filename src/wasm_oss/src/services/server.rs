use crate::commands::CommandDispatcher;
use crate::executor::Executor;

#[derive(Clone)]
pub struct Server {
    dispatcher: CommandDispatcher,
}

impl Server {
    pub fn new(dispatcher: CommandDispatcher) -> Self {
        Self { dispatcher }
    }

    pub async fn run_loop(&self) {}
}

pub struct ServerService {
    server: Server,
}

impl super::Service for ServerService {
    fn name(&self) -> &'static str {
        "server"
    }

    fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn run(&self, executor: &Executor) -> Result<(), Box<dyn std::error::Error>> {
        let server = self.server.clone();
        executor.spawn(async move {
            server.run_loop().await;
        });
        Ok(())
    }
}

impl ServerService {
    pub fn new(dispatcher: CommandDispatcher) -> Self {
        Self {
            server: Server::new(dispatcher),
        }
    }
}
