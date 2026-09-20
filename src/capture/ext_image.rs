use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::fd::AsFd;
use std::time::{Duration, Instant};

use image::{
    RgbaImage,
    imageops::{self, FilterType},
};
use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::fs::{MemfdFlags, memfd_create};
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::{wl_buffer, wl_output, wl_registry, wl_shm, wl_shm_pool};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, WEnum, delegate_noop};
use wayland_protocols::ext::image_capture_source::v1::client::{
    ext_image_capture_source_v1, ext_output_image_capture_source_manager_v1,
};
use wayland_protocols::ext::image_copy_capture::v1::client::{
    ext_image_copy_capture_frame_v1, ext_image_copy_capture_manager_v1,
    ext_image_copy_capture_session_v1,
};
use wayland_protocols::xdg::xdg_output::zv1::client::{
    zxdg_output_manager_v1::ZxdgOutputManagerV1,
    zxdg_output_v1::{self, ZxdgOutputV1},
};

use ext_image_capture_source_v1::ExtImageCaptureSourceV1;
use ext_image_copy_capture_frame_v1::ExtImageCopyCaptureFrameV1;
use ext_image_copy_capture_manager_v1::{ExtImageCopyCaptureManagerV1, Options};
use ext_image_copy_capture_session_v1::ExtImageCopyCaptureSessionV1;
use ext_output_image_capture_source_manager_v1::ExtOutputImageCaptureSourceManagerV1;

use crate::capture::{CaptureError, CaptureResult, DesktopRect, composite};

const BYTES_PER_PIXEL: usize = 4;
const OPAQUE: u8 = 255;
const OUTPUT_MAX_VERSION: u32 = 4;
const XDG_OUTPUT_MANAGER_MAX_VERSION: u32 = 3;
const PROTOCOL_VERSION: u32 = 1;
const DISPATCH_TIMEOUT: Duration = Duration::from_secs(2);

struct OutputState {
    output: wl_output::WlOutput,
    xdg_output: Option<ZxdgOutputV1>,
    position: (i32, i32),
    logical_size: Option<(u32, u32)>,
    transform: wl_output::Transform,
    scale: i32,
    mode: Option<(i32, i32)>,
}

struct Managers {
    shm: wl_shm::WlShm,
    source_manager: ExtOutputImageCaptureSourceManagerV1,
    capture_manager: ExtImageCopyCaptureManagerV1,
}

struct State {
    managers: Managers,
    xdg_output_manager: Option<ZxdgOutputManagerV1>,
    outputs: Vec<OutputState>,
    session_width: u32,
    session_height: u32,
    session_formats: Vec<wl_shm::Format>,
    session_done: bool,
    session_stopped: bool,
    frame_ready: bool,
    frame_failed: Option<String>,
    frame_transform: wl_output::Transform,
}

impl State {
    fn new(managers: Managers, xdg_output_manager: Option<ZxdgOutputManagerV1>) -> Self {
        Self {
            managers,
            xdg_output_manager,
            outputs: Vec::new(),
            session_width: 0,
            session_height: 0,
            session_formats: Vec::new(),
            session_done: false,
            session_stopped: false,
            frame_ready: false,
            frame_failed: None,
            frame_transform: wl_output::Transform::Normal,
        }
    }
}

pub fn capture() -> CaptureResult<(RgbaImage, DesktopRect)> {
    let connection = Connection::connect_to_env().map_err(failure)?;
    let (globals, mut queue) = registry_queue_init::<State>(&connection).map_err(failure)?;
    let qh = queue.handle();
    let mut state = State::new(
        Managers {
            shm: bind::<wl_shm::WlShm>(&globals, &qh)?,
            source_manager: bind::<ExtOutputImageCaptureSourceManagerV1>(&globals, &qh)?,
            capture_manager: bind::<ExtImageCopyCaptureManagerV1>(&globals, &qh)?,
        },
        globals
            .bind::<ZxdgOutputManagerV1, _, _>(&qh, 1..=XDG_OUTPUT_MANAGER_MAX_VERSION, ())
            .ok(),
    );

    for global in globals.contents().clone_list() {
        if global.interface == "wl_output" {
            let output = globals.registry().bind::<wl_output::WlOutput, _, _>(
                global.name,
                global.version.min(OUTPUT_MAX_VERSION),
                &qh,
                (),
            );
            state.outputs.push(OutputState {
                output,
                xdg_output: None,
                position: (0, 0),
                logical_size: None,
                transform: wl_output::Transform::Normal,
                scale: 1,
                mode: None,
            });
        }
    }

    if let Some(manager) = state.xdg_output_manager.clone() {
        for output in &mut state.outputs {
            output.xdg_output = Some(manager.get_xdg_output(&output.output, &qh, ()));
        }
    }

    roundtrip(&mut queue, &mut state)?;

    if state.outputs.is_empty() {
        return Err(CaptureError::ExtImageUnavailable);
    }

    for output in &mut state.outputs {
        if output.logical_size.is_none() {
            let size = fallback_logical_size(output);
            output.logical_size = size;
        }
    }

    let mut captures = Vec::with_capacity(state.outputs.len());
    for index in 0..state.outputs.len() {
        captures.push(capture_output(&mut queue, &mut state, index)?);
    }

    composite(captures)
}

fn capture_output(
    queue: &mut EventQueue<State>,
    state: &mut State,
    index: usize,
) -> CaptureResult<(RgbaImage, i32, i32)> {
    let output = state.outputs[index].output.clone();

    let source = state
        .managers
        .source_manager
        .create_source(&output, &queue.handle(), ());
    let session = state.managers.capture_manager.create_session(
        &source,
        Options::empty(),
        &queue.handle(),
        (),
    );

    state.session_width = 0;
    state.session_height = 0;
    state.session_formats.clear();
    state.session_done = false;
    state.session_stopped = false;

    dispatch_until(queue, state, Instant::now() + DISPATCH_TIMEOUT, |state| {
        state.session_done || state.session_stopped
    })?;
    if state.session_stopped {
        return Err(failure("capture session stopped"));
    }
    if state.session_width == 0 || state.session_height == 0 {
        return Err(failure("capture session reported no buffer size"));
    }

    let format = state
        .session_formats
        .iter()
        .copied()
        .find(|format| matches!(format, wl_shm::Format::Argb8888 | wl_shm::Format::Xrgb8888))
        .ok_or_else(|| failure("capture session offered no supported shm format"))?;

    let width = state.session_width;
    let height = state.session_height;
    let stride = width as usize * BYTES_PER_PIXEL;
    let byte_len = stride * height as usize;

    let mut file = create_shm_file(byte_len)?;
    let pool = state
        .managers
        .shm
        .create_pool(file.as_fd(), byte_len as i32, &queue.handle(), ());
    let buffer = pool.create_buffer(
        0,
        width as i32,
        height as i32,
        stride as i32,
        format,
        &queue.handle(),
        (),
    );

    let frame = session.create_frame(&queue.handle(), ());
    frame.attach_buffer(&buffer);
    frame.damage_buffer(0, 0, width as i32, height as i32);
    frame.capture();

    state.frame_ready = false;
    state.frame_failed = None;
    state.frame_transform = wl_output::Transform::Normal;
    dispatch_until(queue, state, Instant::now() + DISPATCH_TIMEOUT, |state| {
        state.frame_ready || state.frame_failed.is_some()
    })?;
    let failed = state.frame_failed.take();

    frame.destroy();
    buffer.destroy();
    pool.destroy();
    session.destroy();
    source.destroy();

    if let Some(reason) = failed {
        return Err(failure(format!("capture frame failed: {reason}")));
    }
    if !state.frame_ready {
        return Err(failure("capture frame was never ready"));
    }

    let image = read_shm(&mut file, width, height, stride)?;
    let image = orient(image, state.frame_transform);
    let output = &state.outputs[index];
    let image = match output.logical_size {
        Some(size) if image.dimensions() != size => {
            imageops::resize(&image, size.0, size.1, FilterType::Triangle)
        }
        _ => image,
    };
    Ok((image, output.position.0, output.position.1))
}

fn bind<T>(
    globals: &wayland_client::globals::GlobalList,
    qh: &QueueHandle<State>,
) -> CaptureResult<T>
where
    T: wayland_client::Proxy + 'static,
    State: Dispatch<T, ()> + 'static,
{
    globals
        .bind::<T, _, _>(qh, PROTOCOL_VERSION..=PROTOCOL_VERSION, ())
        .map_err(|_| CaptureError::ExtImageUnavailable)
}

fn roundtrip(queue: &mut EventQueue<State>, state: &mut State) -> CaptureResult<()> {
    queue.roundtrip(state).map_err(failure)?;
    Ok(())
}

fn dispatch_until(
    queue: &mut EventQueue<State>,
    state: &mut State,
    deadline: Instant,
    done: impl Fn(&State) -> bool,
) -> CaptureResult<()> {
    while !done(state) {
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            break;
        };
        if !dispatch_within(queue, state, remaining)? {
            break;
        }
    }
    Ok(())
}

fn dispatch_within(
    queue: &mut EventQueue<State>,
    state: &mut State,
    timeout: Duration,
) -> CaptureResult<bool> {
    queue.dispatch_pending(state).map_err(failure)?;
    queue.flush().map_err(failure)?;

    let Some(guard) = queue.prepare_read() else {
        return Ok(true);
    };
    let ready = {
        let fd = guard.connection_fd();
        let mut fds = [PollFd::new(&fd, PollFlags::IN | PollFlags::ERR)];
        let timeout = Timespec::try_from(timeout).map_err(failure)?;
        match poll(&mut fds, Some(&timeout)) {
            Ok(0) => false,
            Ok(_) => true,
            Err(rustix::io::Errno::INTR) => return Ok(true),
            Err(error) => return Err(failure(error)),
        }
    };
    if !ready {
        return Ok(false);
    }

    guard.read().map_err(failure)?;
    queue.dispatch_pending(state).map_err(failure)?;
    Ok(true)
}

fn create_shm_file(byte_len: usize) -> CaptureResult<File> {
    let fd = memfd_create("colorpickle-shm", MemfdFlags::CLOEXEC).map_err(failure)?;
    let file = File::from(fd);
    file.set_len(byte_len as u64).map_err(failure)?;
    Ok(file)
}

fn read_shm(file: &mut File, width: u32, height: u32, stride: usize) -> CaptureResult<RgbaImage> {
    file.seek(SeekFrom::Start(0)).map_err(failure)?;
    let mut bytes = vec![0u8; stride * height as usize];
    file.read_exact(&mut bytes).map_err(failure)?;

    let row_bytes = width as usize * BYTES_PER_PIXEL;
    if stride != row_bytes {
        for row in 1..height as usize {
            let source = row * stride;
            bytes.copy_within(source..source + row_bytes, row * row_bytes);
        }
        bytes.truncate(height as usize * row_bytes);
    }

    for pixel in bytes.as_chunks_mut::<BYTES_PER_PIXEL>().0 {
        pixel.swap(0, 2);
        pixel[3] = OPAQUE;
    }

    RgbaImage::from_raw(width, height, bytes)
        .ok_or_else(|| failure("capture frame has invalid dimensions"))
}

fn fallback_logical_size(output: &OutputState) -> Option<(u32, u32)> {
    let (mode_width, mode_height) = output.mode?;
    let swapped = is_quarter_turn(output.transform);
    let (width, height) = if swapped {
        (mode_height, mode_width)
    } else {
        (mode_width, mode_height)
    };
    let scale = output.scale.max(1);
    Some((
        (width / scale).max(1) as u32,
        (height / scale).max(1) as u32,
    ))
}

fn orient(image: RgbaImage, transform: wl_output::Transform) -> RgbaImage {
    if matches!(transform, wl_output::Transform::Normal) {
        return image;
    }

    let (buffer_width, buffer_height) = image.dimensions();
    let (width, height) = if is_quarter_turn(transform) {
        (buffer_height, buffer_width)
    } else {
        (buffer_width, buffer_height)
    };

    let source_transform = invert_transform(transform);
    let mut oriented = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let (source_x, source_y) = source_pixel(source_transform, x, y, width, height);
            if let Some(pixel) = image.get_pixel_checked(source_x, source_y) {
                oriented.put_pixel(x, y, *pixel);
            }
        }
    }
    oriented
}

fn is_quarter_turn(transform: wl_output::Transform) -> bool {
    matches!(
        transform,
        wl_output::Transform::_90
            | wl_output::Transform::_270
            | wl_output::Transform::Flipped90
            | wl_output::Transform::Flipped270
    )
}

fn invert_transform(transform: wl_output::Transform) -> wl_output::Transform {
    match transform {
        wl_output::Transform::_90 => wl_output::Transform::_270,
        wl_output::Transform::_270 => wl_output::Transform::_90,
        wl_output::Transform::Flipped90 => wl_output::Transform::Flipped270,
        wl_output::Transform::Flipped270 => wl_output::Transform::Flipped90,
        other => other,
    }
}

fn source_pixel(
    transform: wl_output::Transform,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> (u32, u32) {
    match transform {
        wl_output::Transform::Normal => (x, y),
        wl_output::Transform::_90 => (height - y - 1, x),
        wl_output::Transform::_180 => (width - x - 1, height - y - 1),
        wl_output::Transform::_270 => (y, width - x - 1),
        wl_output::Transform::Flipped => (width - x - 1, y),
        wl_output::Transform::Flipped90 => (y, x),
        wl_output::Transform::Flipped180 => (x, height - y - 1),
        wl_output::Transform::Flipped270 => (height - y - 1, width - x - 1),
        _ => (x, y),
    }
}

fn failure(error: impl std::fmt::Display) -> CaptureError {
    CaptureError::ExtImage(error.to_string())
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut State,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

delegate_noop!(State: ignore wl_shm::WlShm);
delegate_noop!(State: wl_shm_pool::WlShmPool);
delegate_noop!(State: ignore wl_buffer::WlBuffer);

impl Dispatch<wl_output::WlOutput, ()> for State {
    fn event(
        state: &mut State,
        proxy: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        let Some(output) = state
            .outputs
            .iter_mut()
            .find(|output| &output.output == proxy)
        else {
            return;
        };
        match event {
            wl_output::Event::Geometry {
                x, y, transform, ..
            } => {
                output.position = (x, y);
                if let WEnum::Value(transform) = transform {
                    output.transform = transform;
                }
            }
            wl_output::Event::Mode { width, height, .. } => {
                output.mode = Some((width, height));
            }
            wl_output::Event::Scale { factor } => output.scale = factor,
            _ => {}
        }
    }
}

delegate_noop!(State: ExtOutputImageCaptureSourceManagerV1);
delegate_noop!(State: ExtImageCaptureSourceV1);
delegate_noop!(State: ExtImageCopyCaptureManagerV1);

impl Dispatch<ExtImageCopyCaptureSessionV1, ()> for State {
    fn event(
        state: &mut State,
        _: &ExtImageCopyCaptureSessionV1,
        event: ext_image_copy_capture_session_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        match event {
            ext_image_copy_capture_session_v1::Event::BufferSize { width, height } => {
                state.session_width = width;
                state.session_height = height;
            }
            ext_image_copy_capture_session_v1::Event::ShmFormat {
                format: WEnum::Value(format),
            } => {
                state.session_formats.push(format);
            }
            ext_image_copy_capture_session_v1::Event::Done => {
                state.session_done = true;
            }
            ext_image_copy_capture_session_v1::Event::Stopped => {
                state.session_stopped = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<ExtImageCopyCaptureFrameV1, ()> for State {
    fn event(
        state: &mut State,
        _: &ExtImageCopyCaptureFrameV1,
        event: ext_image_copy_capture_frame_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        match event {
            ext_image_copy_capture_frame_v1::Event::Transform {
                transform: WEnum::Value(transform),
            } => {
                state.frame_transform = transform;
            }
            ext_image_copy_capture_frame_v1::Event::Ready => state.frame_ready = true,
            ext_image_copy_capture_frame_v1::Event::Failed { reason } => {
                state.frame_failed = Some(format!("{reason:?}"));
            }
            _ => {}
        }
    }
}

delegate_noop!(State: ZxdgOutputManagerV1);

impl Dispatch<ZxdgOutputV1, ()> for State {
    fn event(
        state: &mut State,
        proxy: &ZxdgOutputV1,
        event: zxdg_output_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        let Some(output) = state
            .outputs
            .iter_mut()
            .find(|output| output.xdg_output.as_ref() == Some(proxy))
        else {
            return;
        };
        match event {
            zxdg_output_v1::Event::LogicalPosition { x, y } => output.position = (x, y),
            zxdg_output_v1::Event::LogicalSize { width, height } if width > 0 && height > 0 => {
                output.logical_size = Some((width as u32, height as u32));
            }
            _ => {}
        }
    }
}
