mod app;
mod buffer;
mod camera;
mod ioctl_macros;
mod server_facing;

use app::App;

fn main() {
    let mut app = App::new("10.66.83.44:7878").expect("Failed to initialize App");
    app.run().expect("App encountered an error");
}
