use super::*;

pub use self::{directory::DirectoryHtml, package::PackageHtml, page::PageHtml};

pub(crate) use self::{
  audio::AudioHtml, directory_table::DirectoryTableHtml, error::ErrorHtml, files::FilesHtml,
  home::HomeHtml, image::ImageHtml, info::InfoHtml, media::MediaHtml, packages::PackagesHtml,
  video::VideoHtml,
};

mod audio;
mod directory;
mod directory_table;
mod error;
mod files;
mod home;
mod image;
mod info;
mod media;
mod package;
mod packages;
mod page;
mod video;
