//! The desktop programs of the dotfiles that use GTK (the OSD daemon and its Claude session registry,
//! the now-playing card, the idle screen, the web placeholder) and what they share.

pub mod fifo;
pub mod mpris;
pub mod registry;
pub mod teams;

#[cfg(test)]
mod tests;
