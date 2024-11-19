use crate::ioctl_macros::*;
use crate::buffer::Buffer;
use std::fs::{OpenOptions, File};
use std::io::{Error, ErrorKind};
use std::num::NonZeroUsize;
use std::os::unix::prelude::AsRawFd;
use nix::sys::mman::{mmap, munmap, MapFlags, ProtFlags};
use std::ptr::NonNull;
use std::mem::zeroed;
use v4l2_sys_mit::*;

const V4L2_CAP_VIDEO_CAPTURE: u32 = 1 << 0;

pub struct Camera {
    pub media_fd: File,
    pub buffers: Vec<Buffer>,
    pub reqbufs: v4l2_requestbuffers,
}

impl Camera {
    pub fn new(device_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let media_fd = OpenOptions::new()
            .read(true)
            .write(true)
            .open(device_path)?;

        println!("Camera device fd: {}", media_fd.as_raw_fd());

        let mut capabilities = v4l2_capability { ..unsafe { zeroed() } };
        if unsafe { query_capabilities(media_fd.as_raw_fd(), &mut capabilities).is_err() } {
            return Err(Box::new(Error::last_os_error()));
        }

        if capabilities.capabilities & V4L2_CAP_VIDEO_CAPTURE == 0 {
            return Err(Box::new(Error::new(ErrorKind::Other, "Device does not support video capture")));
        }

        let mut reqbufs = v4l2_requestbuffers {
            count: 20,
            type_: v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE,
            memory: v4l2_memory_V4L2_MEMORY_MMAP,
            capabilities: 0,
            flags: 0,
            reserved: [0; 3],
        };

        if unsafe { request_buffers(media_fd.as_raw_fd(), &mut reqbufs).is_err() } {
            return Err(Box::new(Error::last_os_error()));
        }

        let mut buffers: Vec<Buffer> = Vec::with_capacity(reqbufs.count as usize);
        for i in 0..reqbufs.count {
            let mut buffer_info = v4l2_buffer {
                index: i,
                type_: reqbufs.type_,
                memory: v4l2_memory_V4L2_MEMORY_MMAP,
                ..unsafe { zeroed() }
            };

            if unsafe { query_buffers(media_fd.as_raw_fd(), &mut buffer_info).is_err() } {
                return Err(Box::new(Error::last_os_error()));
            }

            let buffer_length = buffer_info.length;
            let non_zero_length = NonZeroUsize::new(buffer_length as usize)
                .ok_or_else(|| Error::new(ErrorKind::Other, "Invalid buffer length"))?;

            let buffer_start = unsafe {
                mmap::<&File>(
                    None,
                    non_zero_length,
                    ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                    MapFlags::MAP_SHARED,
                    &media_fd,
                    buffer_info.m.offset.into(),
                )
            };

            match buffer_start {
                Ok(start) => {
                    buffers.push(Buffer {
                        start: NonNull::new(start.as_ptr() as *mut u8).expect("Failed to create NonNull pointer"),
                        length: buffer_length as usize,
                    });
                }
                Err(e) => {
                    eprintln!("mmap [FAILED]: {}", e);
                    for mapped_buffer in &buffers {
                        unsafe {
                            munmap(mapped_buffer.start.cast::<libc::c_void>(), mapped_buffer.length).ok();
                        }
                    }
                    return Err(Box::new(Error::last_os_error()));
                }
            }
        }

        Ok(Camera { media_fd, buffers, reqbufs })
    }

    pub fn start_streaming(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for i in 0..self.reqbufs.count {
            let mut buffer_info = v4l2_buffer {
                index: i,
                type_: self.reqbufs.type_,
                memory: self.reqbufs.memory,
                ..unsafe { zeroed() }
            };
            if unsafe { q_buffer(self.media_fd.as_raw_fd(), &mut buffer_info).is_err() } {
                return Err(Box::new(Error::last_os_error()));
            }
        }

        let buf_type = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
        if unsafe { vidioc_streamon(self.media_fd.as_raw_fd(), &buf_type as *const _ as *const i32).is_err() } {
            return Err(Box::new(Error::last_os_error()));
        }

        println!("Streaming started!");
        Ok(())
    }

    pub fn stop_streaming(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let buf_type = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
        if unsafe { vidioc_streamoff(self.media_fd.as_raw_fd(), &buf_type as *const _ as *const i32).is_err() } {
            eprintln!("Failed to stop streaming: {}", Error::last_os_error());
        }

        for mapped_buffer in &self.buffers {
            unsafe {
                munmap(mapped_buffer.start.cast::<libc::c_void>(), mapped_buffer.length).ok();
            }
        }

        Ok(())
    }

    pub fn get_frame(&mut self) -> Result<&[u8], Box<dyn std::error::Error>> {
        let mut buffer_info = v4l2_buffer {
            type_: self.reqbufs.type_,
            memory: self.reqbufs.memory,
            ..unsafe { zeroed() }
        };

        if unsafe { dq_buffer(self.media_fd.as_raw_fd(), &mut buffer_info).is_err() } {
            return Err(Box::new(Error::last_os_error()));
        }

        let frame_data = unsafe {
            std::slice::from_raw_parts(
                self.buffers[buffer_info.index as usize].start.as_ptr(),
                buffer_info.bytesused as usize,
            )
        };

        if unsafe { q_buffer(self.media_fd.as_raw_fd(), &mut buffer_info).is_err() } {
            return Err(Box::new(Error::last_os_error()));
        }

        Ok(frame_data)
    }
}
