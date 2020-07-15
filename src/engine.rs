use crate::command::{CommandExecutor, Command};

pub trait ProxyEngine {
    type Executor;
    type Config;

    fn get_executor(&self) -> &Self::Executor;
    fn get_config(&self) -> &Self::Config;
    fn get_commands(self) -> Vec<Box<dyn Command + Send + Sync>>;
}

pub trait IntoProxyEngine<E> where E: ProxyEngine {
    fn into_engine(self) -> E;
}

impl<E> IntoProxyEngine<E> for E where E: ProxyEngine {
    fn into_engine(self) -> E {
        self
    }
}

pub struct Engine<E, C> where E: CommandExecutor {
    executor: Option<E>,
    config: Option<C>,
    commands: Vec<Box<dyn Command + Send + Sync>>
}

impl<E, C> Engine<E, C> where E: CommandExecutor {
    pub fn new() -> Self {
        Engine {
            executor: None,
            config: None,
            commands: Vec::new()
        }
    }

    pub fn command<T: 'static + Command + Sized + Send + Sync> (mut self, command: T) -> Self {
        let boxed = Box::new(command);
        self.commands.push(boxed);

        self
    }
}

impl<E, C> ProxyEngine for Engine<E, C> where E: CommandExecutor {
    type Executor = E;
    type Config = C;

    fn get_executor(&self) -> &E {
        self.executor.as_ref().unwrap()
    }

    fn get_config(&self) -> &C {
        self.config.as_ref().unwrap()
    }

    fn get_commands(self) -> Vec<Box<dyn Command + Send + Sync>> {
        self.commands
    }
}

pub fn into_engine<E: ProxyEngine, T: IntoProxyEngine<E>>(into: T) -> E {
    into.into_engine()
}