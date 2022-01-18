use crate::gamestates::title::Title;
use crate::gamestates::title_setup::TitleSetup;
use enum_dispatch::enum_dispatch;

#[derive(Debug, Clone, Copy)]
pub enum GameStateId {
    TitleSetup,
    Select,
    Title,
    Gameplay,
    Opening,
    FileChoose,
}

pub struct GameStateCommon {
    pub next_stateid: Option<GameStateId>,
    pub running: bool,
}

impl Default for GameStateCommon {
    fn default() -> Self {
        Self {
            next_stateid: None,
            running: true,
        }
    }
}

#[enum_dispatch(GameStateEnum)]
pub trait GameState {
    fn init(&mut self) -> anyhow::Result<()>;
    fn main(&mut self) -> anyhow::Result<()>;

    fn common(&self) -> &GameStateCommon;
    fn common_mut(&mut self) -> &mut GameStateCommon;

    fn set_next_state(&mut self, id: GameStateId) {
        self.common_mut().next_stateid = Some(id);
    }

    fn stop(&mut self) {
        self.common_mut().running = false;
    }
}

#[enum_dispatch]
pub enum GameStateEnum {
    TitleSetup,
    //Select,
    Title,
    /*Gameplay,
    Opening,
    FileChoose,*/
}

impl GameStateEnum {
    pub fn from_id(id: &GameStateId) -> Self {
        match id {
            GameStateId::TitleSetup => TitleSetup::default().into(),
            GameStateId::Title => Title::default().into(),
            _ => unimplemented!(),
        }
    }
}
