use std::sync::{Arc, RwLock};

extern crate smithay_client_toolkit as sctk;
use sctk::reexports::client::protocol::wl_shm;
use sctk::shm::DoubleMemPool;
use sctk::window::{Event as WEvent, FallbackFrame, Window};
use sctk::WaylandSource;

use core::time::Duration;

use std::io::{BufWriter, ErrorKind, Seek, SeekFrom, Write};
use calloop::{LoopHandle, EventLoop};

use crate::GameField;

sctk::default_environment!(ImViewer, desktop);

pub struct Surface {
    field: Arc<RwLock<GameField>>,
    window: Window<FallbackFrame>,
    pools: DoubleMemPool,
    dimensions: (u32, u32),
    ms_between_frames: u64,
}

impl Surface {
    pub fn new(field: Arc<RwLock<GameField>>, max_fps: u64, loop_handle: Option<LoopHandle<Option<WEvent>>>) -> Self {
        let dimensions = field.read().unwrap().image.dimensions();
        let (env, _display, queue) = sctk::new_default_environment!(ImViewer, desktop)
            .expect("Unable to connect to a Wayland compositor");
        let surface = env
            .create_surface_with_scale_callback(|dpi, _surface, _dispatch_data| {
                println!("dpi changed to {}", dpi);
            })
            .detach();
        let pools = env
            .create_double_pool(|_| {})
            .expect("Failed to create memory pool!");

        let window = env
            .create_window::<FallbackFrame, _>(
                surface,
                None, // None for theme_manager, since we don't theme pointer outself
                dimensions,
                move |evt, mut dispatch_data| {
                    let next_action = dispatch_data.get::<Option<WEvent>>().unwrap();
                    // Check if we need to replace the old event by the new one
                    let replace = matches!(
                        (&evt, &*next_action),
                        // replace if there is no old event
                        (_, &None)
                        // or the old event is refresh
                        | (_, &Some(WEvent::Refresh))
                        // or we had a configure and received a new one
                        | (&WEvent::Configure { .. }, &Some(WEvent::Configure { .. }))
                        // or the new event is close
                        | (&WEvent::Close, _)
                    );
                    if replace {
                        *next_action = Some(evt);
                    }
                },
            )
            .expect("Failed to create a window !");

        window.set_title("Pixelflut".to_string());

        if let Some(handle) = loop_handle {
            WaylandSource::new(queue).quick_insert(handle).expect("Could not register to EventLoop");
        }

        let mut res = Surface {
            dimensions,
            window,
            field,
            pools,
            ms_between_frames: 1000 / max_fps
        };

        if !env.get_shell().unwrap().needs_configure() {
            res.draw().expect("Failed to draw");
            res.window.refresh();
        }

        res
    }

    pub fn run(&mut self, mut event_loop: EventLoop<Option<WEvent>>) -> Result<(), Box<dyn std::error::Error>> {
        let mut next_action = None::<WEvent>;
        let mut need_redraw = false;
        loop {
            match next_action.take() {
                Some(WEvent::Close) => break,
                Some(WEvent::Refresh) => {
                    self.window.refresh();
                    self.window.surface().commit();
                }
                Some(WEvent::Configure {
                    new_size,
                    states: _,
                }) => {
                    if let Some((w, h)) = new_size {
                        if self.dimensions != (w, h) {
                            self.dimensions = (w, h);
                            // let mut base_image = self.image.write().unwrap();
                            // *base_image = image::imageops::resize(&base_image, w, h, image::imageops::FilterType::Nearest);
                        }
                    }
                    self.window.resize(self.dimensions.0, self.dimensions.1);
                    self.window.refresh();

                    need_redraw = true;
                }
                None => {}
            }
            if need_redraw || self.field.read().unwrap().dirty {
                if let Err(_) = self.draw() {
                   eprintln!("All pools are used by wayland") 
                }  else {
                    need_redraw = false;
                    self.field.write().unwrap().dirty = false;
                }
            }
            event_loop.dispatch(Duration::from_millis(self.ms_between_frames), &mut next_action).unwrap();
        }

        Ok(())
    }

    fn draw(&mut self) -> Result<(), std::io::Error> {
        if let Some(pool) = self.pools.pool() {
            let image = &self.field.read().unwrap().image;
            let surface = self.window.surface();
            let stride = 4 * self.dimensions.0 as i32;
            let width = self.dimensions.0 as i32;
            let height = self.dimensions.1 as i32;

            // First make sure the pool is the right size
            pool.resize((stride * height) as usize)?;

            // Create a new buffer from the pool
            let buffer = pool.buffer(0, width, height, stride, wl_shm::Format::Abgr8888);

            // Write the color to all bytes of the pool
            pool.seek(SeekFrom::Start(0))?;
            {
                let mut writer = BufWriter::new(&mut *pool);
                writer.write_all(image.as_raw())?;
                writer.flush()?;
            }

            // Attach the buffer to the surface and mark the entire surface as damaged
            surface.attach(Some(&buffer), 0, 0);
            surface.damage_buffer(0, 0, width as i32, height as i32);

            // Finally, commit the surface
            surface.commit();
            Ok(())
        } else {
            Err(std::io::Error::new(
                ErrorKind::Other,
                "All pools are in use by Wayland",
            ))
        }
    }
}
