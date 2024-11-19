# Reducing Memory Access Across Kernel-User Boundary

**Date:** November 1st  
**Author:** Anton Melnychuk

### Overview

This project involves optimizing the flow of video capture and DNN (Deep Neural Network) inference in a resource-constrained environment. The system is split into two components running over a TCP connection: the local client component captures images from the camera, sends them to a remote server for inference, and displays the results; the remote server component performs the DNN inference using TensorFlow Lite, processes the images, and sends back the results.

The key goals of this project are to:
1. Offload DNN inference to a more powerful remote server.
2. Minimize memory access overheads by reducing the number of kernel-user boundary crossings during image capture.
3. Optimize the system for performance, ensuring the client can handle high frame rates.

### General Structure

The system consists of a **client** and a **server**. The client captures video frames using the Video for Linux (V4L2) API and sends them to the server. The server processes the frames using TensorFlow Lite and returns inference results to the client. Communication between the two is handled asynchronously over TCP.

- The **client** is structured with two main components:
  - `App` struct: Handles the camera capture and marks inference results on the display.
  - `Server-facing` struct: Manages the TCP connection to the server, serializes images, and retrieves inference results.
  
- The **server** listens for incoming image data, performs inference, and sends the results back to the client.

The architecture separates the app logic from network handling, ensuring modularity and ease of future updates.

```
anton@anton22:~/workspace/rust_movenet_client/src$ tree
.
├── app.rs
├── buffer.rs
├── camera.rs
├── ioctl_macros.rs
├── main.rs
└── server_facing.rs
```

```
anton@anton22:~/workspace/rust_movenet_server/src$ tree
.
├── main.rs
└── utils.rs
```

Run the client and server components using `cargo run`, ensuring you configure the appropriate IP address for server communication.

### Buffer Management in Kernel

This project leverages kernel-managed buffer queues to efficiently handle video capture. The video driver allocates memory buffers that the user-space application accesses directly, minimizing the overhead of copying data between kernel and user space.

- 20 buffer slots are used to ensure sufficient frames are available for continuous streaming while optimizing memory usage.
- This approach helps reduce latency and improve throughput for smooth video capture and processing.

### Key Features

- **Separation of Logic**: The client application uses two distinct structs for managing the app logic (`App`) and server communication (`Server-facing`), promoting maintainability and scalability.
- **Low-Latency Communication**: The client and server communicate over a TCP connection, with the server performing DNN inference and the client displaying the results.
- **Efficient Buffer Management**: Memory-mapped buffers are used to minimize the overhead of copying image data between kernel and user spaces, ensuring smooth streaming and high-performance processing.

### Performance Optimizations

- **Threading**: A concurrent, thread-safe environment is used to handle sending and receiving video frames, minimizing wait times and maximizing throughput.
- **Network Optimizations**: The system is designed to handle network delays and potential packet loss. A sequence number for each frame allows the system to handle unordered image processing, ensuring the correct frame is displayed even when results arrive out of order.

### Upcoming Improvements

We further focus on eliminating unnecessary memory access across the kernel-user boundary during image capture. The current implementation relies on OpenCV to convert YUV422 frames into RGB, but this involves costly memory copying. 

The following changes will be made:
1. Modify the **client** to directly capture YUV422 frames using the V4L2 API, eliminating the need for OpenCV.
2. Move the YUV-to-RGB conversion to the **server** to offload processing and minimize overhead on the client side.

By optimizing the client-side image capture and reducing memory copying, we aim to achieve even better performance, allowing for smoother and more efficient video streaming and DNN inference.

### Valuable Resources Used

- [Nix Documentation](https://docs.rs/nix/latest/nix/sys/ioctl/index.html)
- [OpenCV Video Capture Code](https://github.com/opencv/opencv/blob/67fa8a2f4720404a15da7a723bc048b247c5d227/modules/videoio/src/cap_v4l.cpp)
- [Kernel Documentation on Memory Mapping](https://www.kernel.org/doc/html/v4.9/media/uapi/v4l/mmap.html)
- [Kernel Documentation on Video Capture](https://www.kernel.org/doc/html/v4.9/media/uapi/v4l/capture.c.html)

### Intermediate Results & Issues

The current implementation demonstrates stable performance without threading but can benefit from a concurrent architecture for handling both video sending and receiving.

Example output without threading:

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

In the absence of threading, performance suffers due to the sequential execution of the client and server tasks. The system is being optimized for concurrent processing and better handling of image frames.

### Results

![PROOF Part3](./PROOF.png)
