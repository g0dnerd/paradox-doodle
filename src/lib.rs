use clap::Parser;

pub mod camera;
pub mod framework;
pub mod scene;

pub struct Config {
    pub image_size: u32,
    pub camera_distance: f32,
}

#[derive(Parser, Debug)]
#[command(version = "0.1")]
#[command(about = "renders a black hole in a skybox")]
#[command(long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    image_size: u32,

    #[arg(short, long)]
    camera_distance: Option<f32>,
}
