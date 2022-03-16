use crate::command::Command;
use crate::GameField;
use image::Rgba;
use std::sync::{Arc, RwLock};
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

pub struct PixelTcpServer {
    port: u16,
    field: Arc<RwLock<GameField>>,
}

impl PixelTcpServer {
    pub fn new(field: Arc<RwLock<GameField>>, port: u16) -> Self {
        PixelTcpServer { field, port }
    }

    pub async fn run(self) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))
            .await
            .unwrap_or_else(|_| panic!("Could not bind to port {}", self.port));

        loop {
            let (socket, _) = listener
                .accept()
                .await
                .expect("Unable to accept new connection");
            let field = Arc::clone(&self.field);

            tokio::spawn(async move {
                let mut stream = BufReader::new(socket);

                loop {
                    let mut line = String::new();
                    stream.read_line(&mut line).await.expect("Failed to read");
                    if line.is_empty() {
                        break;
                    }

                    let command = Command::parse(&line);

                    match command {
                        Command::Size => {
                            let (width, height) = field.read().unwrap().image.dimensions();
                            stream
                                .write(format!("SIZE {} {}\n", width, height).as_bytes())
                                .await
                                .ok();
                        }
                        Command::Failed => {
                            stream.write("ERROR parsing failed\n".as_bytes()).await.ok();
                        }
                        Command::Help => {
                            stream
                                .write("ERROR not implemented\n".as_bytes())
                                .await
                                .ok();
                        }
                        Command::Px(x, y, Some(color)) => {
                            let (width, height) = field.read().unwrap().image.dimensions();
                            if x < width && y < height {
                                let mut field = field.write().unwrap();
                                field.image.put_pixel(
                                    x,
                                    y,
                                    Rgba([color.red, color.green, color.blue, color.alpha]),
                                );
                                field.dirty = true;
                            } else {
                                stream.write("ERROR out of bounds\n".as_bytes()).await.ok();
                            }
                        }
                        Command::Px(x, y, None) => {
                            let (width, height) = field.read().unwrap().image.dimensions();
                            if x < width && y < height {
                                let color = *field.read().unwrap().image.get_pixel(x, y);
                                stream
                                    .write(
                                        format!(
                                            "PX {} {} {:02x}{:02x}{:02x}{:02x}\n",
                                            x, y, color[0], color[1], color[2], color[3]
                                        )
                                        .as_bytes(),
                                    )
                                    .await
                                    .ok();
                            } else {
                                stream.write("ERROR out of bounds\n".as_bytes()).await.ok();
                            }
                        }
                    }
                }
            });
        }
    }
}
