use std::os::unix::{
    io::{AsRawFd, RawFd, FromRawFd},
    net::UnixListener,
};
use tiny_skia::*;
use nix::sys::socket::{recvmsg, ControlMessageOwned, MsgFlags};
use nix::cmsg_space;

/// This is part of a two-part example. See `shm_client.rs` for documentation.
fn main() -> std::io::Result<()> {
    // If you change this, make sure to modify the corresponding values in `shm_client.rs` as well:
    let width = 1000;
    let height = 1000;

    let socket_path = std::path::Path::new("./shm-example.sock");
    if socket_path.exists() {
        std::fs::remove_file(&socket_path).unwrap();
    }
    let listener = UnixListener::bind(&socket_path)?;

    println!("Waiting for client");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut cmsg_buffer = cmsg_space!([RawFd; 1]);
                let received = recvmsg(stream.as_raw_fd(), &[], Some(&mut cmsg_buffer), MsgFlags::empty()).unwrap();
                for cmsg in received.cmsgs() {
                    match cmsg {
                        ControlMessageOwned::ScmRights(fds) => {
                            let mut mmap = unsafe {
                                let file = std::fs::File::from_raw_fd(fds[0]);
                                memmap::MmapOptions::new().map_mut(&file)
                            }.unwrap();
                            let pixmap = Pixmap::from_data(width, height, &mut mmap).unwrap();
                            pixmap.save_png("image-shm.png").unwrap();
                            println!("Wrote to image-shm.png");
                        },
                        _ => {}
                    }
                }
            }
            Err(err) => {
                println!("Failed to accept stream: {:?}", err);
            }
        }
    }

    Ok(())
}
