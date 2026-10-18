//! The desktop programs of the dotfiles (GTK windows, the OSD daemon and its Claude session registry,
//! small system checks) and what they share.

pub mod fifo;
pub mod mpris;
pub mod registry;
pub mod temperature;

#[cfg(test)]
mod tests;
