
mod util;

use std::{ cmp, env, fs };
use std::io::{ self, Read };
use std::path::{ Path, PathBuf };
use argh::FromArgs;
use anyhow::Context;