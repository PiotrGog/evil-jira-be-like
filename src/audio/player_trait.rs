#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait PlayerTrait {
    fn play(&self) -> anyhow::Result<()>;
}
