

#[derive(States, Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Sequence)]
pub enum AppState {
    #[default]
    StartMenu,
    GameCreate,
    GameRunning,
    GamePaused,
    GameOver,
}

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        for state in all::<AppState>() {
            app.add_systems(OnEnter(state), state_enter_despawn::<AppState>);
        }
    }
}