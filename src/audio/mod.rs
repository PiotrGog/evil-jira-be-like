mod player_trait;
mod sin_player;

pub use player_trait::PlayerTrait;
pub use sin_player::SinPlayer;

#[cfg(any(test))]
pub mod testing {
    pub use super::player_trait::MockPlayerTrait;
}
