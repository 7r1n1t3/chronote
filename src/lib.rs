//! Chronote: animated study timelines with notes and attachments.
//!
//! The app itself is the menu ([`menu`]), the timeline editor ([`editor`],
//! [`track`]) and its storage ([`project`]). The built-in Embryo timeline is
//! only a demo of what a timeline can look like; it lives in [`demo`] with
//! its own data ([`data`]), illustration ([`fetus`]) and scrubber
//! ([`timeline`]).

pub mod anim;
pub mod app;
pub mod crash;
pub mod data;
pub mod demo;
pub mod editor;
pub mod fetus;
pub mod logging;
pub mod menu;
pub mod project;
pub mod startup;
pub mod theme;
pub mod timeline;
pub mod track;
pub mod ui;
