use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::os::fd::AsFd;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use image::{Rgba, RgbaImage, imageops};
use rustix::event::{PollFd, PollFlags, Timespec, poll};
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::{wl_buffer, wl_output, wl_registry, wl_shm, wl_shm_pool};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, WEnum};
use wayland_protocols::ext::image_capture_source::v1::client::{
    ext_image_capture_source_v1, ext_output_image_capture_source_manager_v1,
};
use wayland_protocols::ext::image_copy_capture::v1::client::{
    ext_image_copy_capture_frame_v1, ext_image_copy_capture_manager_v1,
    ext_image_copy_capture_session_v1,
};

use ext_image_capture_source_v1::ExtImageCaptureSourceV1;
use ext_image_copy_capture_frame_v1::ExtImageCopyCaptureFrameV1;
use ext_image_copy_capture_manager_v1::{ExtImageCopyCaptureManagerV1, Options};
use ext_image_copy_capture_session_v1::ExtImageCopyCaptureSessionV1;
use ext_output_image_capture_source_manager_v1::ExtOutputImageCaptureSourceManagerV1;

use crate::capture::{CaptureBackend, CaptureError, CaptureResult};

const BYTES_PER_PIXEL: usize = 4;
const OPAQUE: u8 = 255;
const OUTPUT_MAX_VERSION: u32 = 4;
const PROTOCOL_VERSION: u32 = 1;
const DISPATCH_TIMEOUT: Duration = Duration::from_secs(2);

static SHM_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct ExtImageBackend {
    frame: RgbaImage,
}

impl ExtImageBackend {
    pub fn new() -> CaptureResult<Self> {
        Ok(Self { frame: capture()? })
    }
}

impl CaptureBackend for ExtImageBackend {
    fn capture_fullscreen(&self) -> CaptureResult<RgbaImage> {
        Ok(self.frame.clone())
    }
}

struct OutputState {
    output: wl_output::WlOutput,
    x: i32,
    y: i32,
}

#[derive(Default)]
struct State {
    shm: Option<wl_shm::WlShm>,
    source_manager: Option<ExtOutputImageCaptureSourceManagerV1>,
    capture_manager: Option<ExtImageCopyCaptureManagerV1>,
    outputs: Vec<OutputState>,
    session_width: u32,
    session_height: u32,
    session_formats: Vec<wl_shm::Format>,
    session_done: bool,
    session_stopped: bool,
    frame_ready: bool,
    frame_failed: Option<String>,
}

fn capture() -> CaptureResult<RgbaImage> {
    let connection = Connection::connect_to_env().map_err(failure)?;
    let (globals, mut queue) = registry_queue_init::<State>(&connection).map_err(failure)?;
    let qh = queue.handle();
    let mut state = State::default();

    state.shm = Some(bind::<wl_shm::WlShm>(&globals, &qh)?);
    state.source_manager = Some(bind::<ExtOutputImageCaptureSourceManagerV1>(&globals, &qh)?);
    state.capture_manager = Some(bind::<ExtImageCopyCaptureManagerV1>(&globals, &qh)?);

    for global in globals.contents().clone_list() {
        if global.interface == "wl_output" {
            let output = globals.registry().bind::<wl_output::WlOutput, _, _>(
                global.name,
                global.version.min(OUTPUT_MAX_VERSION),
                &qh,
                (),
            );
            state.outputs.push(OutputState { output, x: 0, y: 0 });
        }
    }
    roundtrip(&mut queue, &mut state)?;

    if state.outputs.is_empty() {
        return Err(CaptureError::ExtImageUnavailable);
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
    let (output, x, y) = {
        let output = &state.outputs[index];
        (output.output.clone(), output.x, output.y)
    };

    let source = state
        .source_manager
        .as_ref()
        .expect("source manager bound")
        .create_source(&output, &queue.handle(), ());
    let session = state
        .capture_manager
        .as_ref()
        .expect("capture manager bound")
        .create_session(&source, Options::empty(), &queue.handle(), ());

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
    let pool = state.shm.as_ref().expect("shm bound").create_pool(
        file.as_fd(),
        byte_len as i32,
        &queue.handle(),
        (),
    );
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
    Ok((image, x, y))
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
    let unique = SHM_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let mut path = std::env::temp_dir();
    path.push(format!(
        "colorpickle-shm-{}-{nanos}-{unique}",
        std::process::id()
    ));

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(failure)?;
    std::fs::remove_file(&path).map_err(failure)?;
    file.set_len(byte_len as u64).map_err(failure)?;
    Ok(file)
}

fn read_shm(file: &mut File, width: u32, height: u32, stride: usize) -> CaptureResult<RgbaImage> {
    file.seek(SeekFrom::Start(0)).map_err(failure)?;
    let mut bytes = vec![0u8; stride * height as usize];
    file.read_exact(&mut bytes).map_err(failure)?;

    let mut image = RgbaImage::new(width, height);
    for y in 0..height {
        let row = &bytes[y as usize * stride..];
        for x in 0..width {
            let offset = x as usize * BYTES_PER_PIXEL;
            image.put_pixel(
                x,
                y,
                Rgba([row[offset + 2], row[offset + 1], row[offset], OPAQUE]),
            );
        }
    }
    Ok(image)
}

fn composite(captures: Vec<(RgbaImage, i32, i32)>) -> CaptureResult<RgbaImage> {
    let min_x = captures.iter().map(|(_, x, _)| *x).min().unwrap_or(0);
    let min_y = captures.iter().map(|(_, _, y)| *y).min().unwrap_or(0);
    let max_x = captures
        .iter()
        .map(|(image, x, _)| x + image.width() as i32)
        .max()
        .unwrap_or(0);
    let max_y = captures
        .iter()
        .map(|(image, _, y)| y + image.height() as i32)
        .max()
        .unwrap_or(0);

    let width = (max_x - min_x).max(0) as u32;
    let height = (max_y - min_y).max(0) as u32;
    if width == 0 || height == 0 {
        return Err(failure("captured no pixels"));
    }

    let mut canvas = RgbaImage::new(width, height);
    for (image, x, y) in captures {
        imageops::overlay(
            &mut canvas,
            &image,
            i64::from(x - min_x),
            i64::from(y - min_y),
        );
    }
    Ok(canvas)
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

impl Dispatch<wl_shm::WlShm, ()> for State {
    fn event(
        _: &mut State,
        _: &wl_shm::WlShm,
        _: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for State {
    fn event(
        _: &mut State,
        _: &wl_shm_pool::WlShmPool,
        _: <wl_shm_pool::WlShmPool as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for State {
    fn event(
        _: &mut State,
        _: &wl_buffer::WlBuffer,
        _: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

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
        if let wl_output::Event::Geometry { x, y, .. } = event {
            output.x = x;
            output.y = y;
        }
    }
}

impl Dispatch<ExtOutputImageCaptureSourceManagerV1, ()> for State {
    fn event(
        _: &mut State,
        _: &ExtOutputImageCaptureSourceManagerV1,
        _: ext_output_image_capture_source_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

impl Dispatch<ExtImageCaptureSourceV1, ()> for State {
    fn event(
        _: &mut State,
        _: &ExtImageCaptureSourceV1,
        _: ext_image_capture_source_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

impl Dispatch<ExtImageCopyCaptureManagerV1, ()> for State {
    fn event(
        _: &mut State,
        _: &ExtImageCopyCaptureManagerV1,
        _: ext_image_copy_capture_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

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
            ext_image_copy_capture_session_v1::Event::ShmFormat { format } => {
                if let WEnum::Value(format) = format {
                    state.session_formats.push(format);
                }
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
            ext_image_copy_capture_frame_v1::Event::Ready => state.frame_ready = true,
            ext_image_copy_capture_frame_v1::Event::Failed { reason } => {
                state.frame_failed = Some(format!("{reason:?}"));
            }
            _ => {}
        }
    }
}
