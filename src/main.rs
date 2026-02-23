#![deny(clippy::pedantic, clippy::unwrap_used, clippy::clone_on_ref_ptr)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod api;
mod app;
mod audio;
mod config;
mod message;
mod server;
mod transcribe;
mod views;

use app::Qvox;

fn main() -> anyhow::Result<()> {
    iced::application(Qvox::new, Qvox::update, Qvox::view)
        .title(Qvox::title)
        .subscription(Qvox::subscription)
        .theme(Qvox::theme)
        .run()?;
    Ok(())
}
