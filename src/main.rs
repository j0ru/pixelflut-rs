use crate::server::PixelTcpServer;
use calloop::EventLoop;
use clap::Parser;
use image::{Rgba, RgbaImage};
use smithay_client_toolkit::window::Event as WEvent;
use std::sync::{Arc, RwLock};

mod command;
mod gui;
mod server;

pub struct GameField {
    pub image: RgbaImage,
    pub dirty: bool,
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, default_value_t = 1000)]
    height: u32,
    #[clap(short, long, default_value_t = 1000)]
    width: u32,

    #[clap(short, long, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let field = Arc::new(RwLock::new(GameField {
        image: RgbaImage::from_pixel(args.width, args.height, Rgba([0, 0, 0, 255])),
        dirty: false,
    }));

    let event_loop = EventLoop::<Option<WEvent>>::try_new().unwrap();

    let server = PixelTcpServer::new(Arc::clone(&field), args.port);
    tokio::spawn(server.run()); // server is consumed here

    let mut surface = gui::Surface::new(Arc::clone(&field), 60, Some(event_loop.handle()));
    surface.run(event_loop)?;
    Ok(())
}
