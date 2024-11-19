use nix::{ioctl_read, ioctl_write_ptr, ioctl_readwrite};
use std::os::raw::c_int;
use v4l2_sys_mit::*;

ioctl_read!(query_capabilities, b'V', 0, v4l2_capability);
ioctl_readwrite!(request_buffers, b'V', 8, v4l2_requestbuffers);
ioctl_readwrite!(query_buffers, b'V', 9, v4l2_buffer);
ioctl_readwrite!(q_buffer, b'V', 15, v4l2_buffer);
ioctl_readwrite!(dq_buffer, b'V', 17, v4l2_buffer);
ioctl_write_ptr!(vidioc_streamon, b'V', 18, c_int);
ioctl_write_ptr!(vidioc_streamoff, b'V', 19, c_int);
