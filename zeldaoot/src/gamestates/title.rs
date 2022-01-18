use crate::graph::{GameState, GameStateCommon};

#[derive(Default)]
pub struct Title {
    common: GameStateCommon,
}

impl GameState for Title {
    fn common(&self) -> &GameStateCommon {
        &self.common
    }

    fn common_mut(&mut self) -> &mut GameStateCommon {
        &mut self.common
    }

    fn init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn main(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
