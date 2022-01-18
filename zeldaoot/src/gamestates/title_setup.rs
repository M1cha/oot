use crate::graph::{GameState, GameStateCommon, GameStateId};
use log::info;

#[derive(Default)]
pub struct TitleSetup {
    common: GameStateCommon,
}

impl GameState for TitleSetup {
    fn common(&self) -> &GameStateCommon {
        &self.common
    }

    fn common_mut(&mut self) -> &mut GameStateCommon {
        &mut self.common
    }

    fn init(&mut self) -> anyhow::Result<()> {
        info!("Zelda common data initalization");

        // TODO: init SaveContext
        self.stop();
        self.set_next_state(GameStateId::Title);

        Ok(())
    }

    fn main(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
