# Reducing Memory Access Across Kernel-User Boundary

Date: November 1st<br>
Author: Anton Melnychuk<br>

### General Structure

A server and a client operate asynchronously over TCP. The client establishes an interface with the server via `server_facing.rs` and creates a direct connection with the camera using `IOCTL`, thereby reducing unnecessary overhead associated with OpenCV. The application flow and display management are handled in `app.rs`.

```
anton@anton22:~/workspace/rust_movenet_client/src$ tree
.
├── app.rs
├── buffer.rs
├── camera.rs
├── ioctl_macros.rs
├── main.rs
└── server_facing.rs

0 directories, 6 files
```

```
anton@anton22:~/workspace/rust_movenet_server/src$ tree
.
├── main.rs
└── utils.rs

0 directories, 2 files
```

Use `cargo run` to start both microservices and set your own IP.

### Buffer Management in Kernel

This project utilizes memory mapping and buffer queues managed by the kernel to efficiently handle video capture. The camera driver allocates buffers that the user-space application can access directly, minimizing the overhead of copying data between kernel and user space. This approach allows the application to quickly enqueue and dequeue frames, facilitating smooth data flow and reducing latency.

Here, 20 buffer count has been chosen to ensure that there are enough frames available for continuous streaming without underflow, while also allowing for efficient memory management. This balance showed to help slightly minimize latency and maximizes throughput during video capture, enabling smoother performance.

### Valuable Resources Used

- [Nix Documentation](https://docs.rs/nix/latest/nix/sys/ioctl/index.html)
- [OpenCV Video Capture Code](https://github.com/opencv/opencv/blob/67fa8a2f4720404a15da7a723bc048b247c5d227/modules/videoio/src/cap_v4l.cpp)
- [Kernel Documentation on Memory Mapping](https://www.kernel.org/doc/html/v4.9/media/uapi/v4l/mmap.html)
- [Kernel Documentation on Video Capture](https://www.kernel.org/doc/html/v4.9/media/uapi/v4l/capture.c.html)

### Intermediate Results & Issues

Without threading, the frame capture process yields suboptimal performance:

```
anton@anton22:~/workspace/rust_movenet_client/src$ cargo run
   Compiling rust_movenet_client v0.1.0 (/home/anton/workspace/rust_movenet_client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.68s
     Running `/home/anton/workspace/rust_movenet_client/target/debug/rust_movenet_client`
Camera device fd: 4
Streaming started!
FPS: 30.40
FPS: 30.67
```

Note, the sequential execution of both machines would waste valuable time while waiting for responses from each other. Therefore, we employ a thread-safe concurrent environment for `sending` and `receiving` videos both for the client and server. Another potential improvement, albeit minor based on testing, involves adjusting the size of the images sent over the network.

INTERESTING IMPOROVEMENT: In a concurrent setup where the server handles images, we must consider unordered image handling. If a neural network returns one frame faster than another, using a camera sequence number will allow us to block one data display in favor of another or even drop frames as needed.

### Results

![PROOF Part3](./part3/PROOF.png)
