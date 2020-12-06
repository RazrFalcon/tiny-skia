use nix::sys::uio::IoVec;
use tiny_skia::*;
use nix::sys::socket::{
    sendmsg,
    ControlMessage,
    MsgFlags,
};
use std::os::unix::{
    io::AsRawFd,
    net::UnixStream,
};

/// This example demonstrates drawing with skia using a shared memory region.
///
/// It consists of two parts:
/// 1. `shm_client`: Draws an image onto a shared memory region.
/// 2. `shm_server`: Writes the image from the shared memory region to a PNG file ("image-shm.png").
///
/// To run the example, you'll need two terminal windows.
///
/// In one of the windows, run the shm_server example:
///
///     cargo run --example shm_server
///
/// Keep that example running, and run shm_client in the second window:
///
///     cargo run --example shm_client
///
/// After the command returns, you should see a message in the server window, and image-shm.png contains the image
/// that was drawn by the client.
///
/// You can press CTRL+C to terminate the server.
fn main() {
    // If you change this, make sure to modify the corresponding values in `shm_server.rs` as well:
    let width = 1000;
    let height = 1000;

    let file = tempfile::tempfile().unwrap();
    file.set_len((width * height * 4) as u64).unwrap();

    let mut mmap = unsafe { memmap::MmapOptions::new().map_mut(&file) }.unwrap();

    let pixmap = Pixmap::from_data(width, height, &mut mmap).unwrap();
    let mut canvas: Canvas = pixmap.into();

    let mut paint1 = Paint::default();
    paint1.set_color_rgba8(50, 127, 150, 200);
    paint1.anti_alias = true;

    let mut paint2 = Paint::default();
    paint2.set_color_rgba8(220, 140, 75, 180);

    let path1 = {
        let mut pb = PathBuilder::new();
        pb.move_to(60.0, 60.0);
        pb.line_to(160.0, 940.0);
        pb.cubic_to(380.0, 840.0, 660.0, 800.0, 940.0, 800.0);
        pb.cubic_to(740.0, 460.0, 440.0, 160.0, 60.0, 60.0);
        pb.close();
        pb.finish().unwrap()
    };

    let path2 = {
        let mut pb = PathBuilder::new();
        pb.move_to(940.0, 60.0);
        pb.line_to(840.0, 940.0);
        pb.cubic_to(620.0, 840.0, 340.0, 800.0, 60.0, 800.0);
        pb.cubic_to(260.0, 460.0, 560.0, 160.0, 940.0, 60.0);
        pb.close();
        pb.finish().unwrap()
    };

    canvas.fill_path(&path1, &paint1, FillRule::Winding);
    canvas.fill_path(&path2, &paint2, FillRule::Winding);

    let socket = UnixStream::connect("./shm-example.sock")
        .expect("Failed to connect to ./shm-example.sock. Is shm_server example running?");

    sendmsg(
        socket.as_raw_fd(),
        // Even though we only want to send a control message, the payload must have at least 1 byte.
        &[IoVec::from_slice(&[0; 1])],
        &[ControlMessage::ScmRights(&[file.as_raw_fd()])],
        MsgFlags::empty(),
        None
    ).expect("Failed to send message to socket");
}
