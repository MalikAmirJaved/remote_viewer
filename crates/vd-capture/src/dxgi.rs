//! DXGI Desktop Duplication capturer (Windows, P1 MVP).
//!
//! Scope of T-0107: pick a monitor on the adapter that owns the D3D11 device, acquire
//! frames without blocking, hand out tightly packed BGRA8 pixels plus dirty rects, and
//! rebuild the duplication interface when Windows reports `DXGI_ERROR_ACCESS_LOST`.
//!
//! Deliberately out of scope here:
//! - rotated outputs (`DXGI_MODE_ROTATION_*`, T-0605) — rejected with a clear error
//!   instead of returning a sideways frame;
//! - the hardware cursor, which DXGI reports separately (T-0409);
//! - outputs on secondary adapters (T-0605);
//! - zero-copy GPU hand-off (P5). This backend reads back through a staging texture,
//!   which is the documented P1 software-encoder path.
//!
//! Thread affinity: COM apartment, D3D11 immediate context and the duplication object
//! belong to one OS thread. Create, capture and drop on that same thread.

use vd_common::{Error, MonotonicClock, Result};

use windows::core::Interface;
use windows::Win32::Foundation::{HMODULE, RECT, RPC_E_CHANGED_MODE};
use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_10_0, D3D_FEATURE_LEVEL_10_1,
    D3D_FEATURE_LEVEL_11_0,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D,
    D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAP_READ, D3D11_SDK_VERSION,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_MODE_ROTATION_IDENTITY, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
    IDXGIAdapter, IDXGIDevice, IDXGIOutput, IDXGIOutput1, IDXGIOutputDuplication, IDXGIResource,
    DXGI_ERROR_ACCESS_LOST, DXGI_ERROR_MORE_DATA, DXGI_ERROR_NOT_CURRENTLY_AVAILABLE,
    DXGI_ERROR_WAIT_TIMEOUT, DXGI_OUTDUPL_FRAME_INFO,
};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

/// Bytes per RGBA/BGRA pixel in the format this backend emits.
const BYTES_PER_PIXEL: u32 = 4;

/// COM apartment scoped to one capturer instance.
///
/// DXGI Desktop Duplication requires COM on the rendering thread. If the thread already
/// joined an apartment (an egui/eframe UI thread is usually STA), that is fine; the
/// instance only remembers whether *it* must balance the reference count.
struct ComApartment {
    owned: bool,
}

impl ComApartment {
    fn init() -> Result<Self> {
        // SAFETY: `pvReserved` must be null; no COM objects are used across threads.
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr.is_ok() {
            return Ok(Self { owned: true });
        }
        if hr == RPC_E_CHANGED_MODE {
            // Another apartment mode is already active on this thread. COM works, but this
            // instance must not call CoUninitialize or it would unbalance the other owner.
            return Ok(Self { owned: false });
        }
        Err(Error::Unavailable(format!(
            "DXGI: CoInitializeEx failed: {hr:?}"
        )))
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.owned {
            // SAFETY: balanced with the successful CoInitializeEx that set `owned`.
            unsafe { CoUninitialize() };
        }
    }
}

/// A staging texture plus its `ID3D11Resource` view, cached to avoid per-frame casts.
struct StagingTexture {
    resource: ID3D11Resource,
}

impl StagingTexture {
    fn new(device: &ID3D11Device, width: u32, height: u32) -> Result<Self> {
        let desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };
        let mut texture: Option<ID3D11Texture2D> = None;
        // SAFETY: `desc` is a fully initialized, correctly sized descriptor for a 2D
        // texture; `texture` receives the created object.
        unsafe { device.CreateTexture2D(&raw const desc, None, Some(&raw mut texture)) }
            .map_err(|e| Error::Unavailable(format!("DXGI: CreateTexture2D: {e}")))?;
        let texture = texture
            .ok_or_else(|| Error::Other("DXGI: CreateTexture2D returned no texture".into()))?;
        let resource: ID3D11Resource = texture
            .cast()
            .map_err(|e| Error::Other(format!("DXGI: cast texture to resource: {e}")))?;
        Ok(Self { resource })
    }
}

/// DXGI Desktop Duplication capturer.
///
/// Create, use and drop on a single OS thread (see the module docs).
pub struct DxgiCapturer {
    com: Option<ComApartment>,
    device: Option<ID3D11Device>,
    context: Option<ID3D11DeviceContext>,
    duplication: Option<IDXGIOutputDuplication>,
    staging: Option<StagingTexture>,
    monitor_index: u32,
    width: u32,
    height: u32,
    clock: MonotonicClock,
    needs_keyframe: bool,
}

impl Default for DxgiCapturer {
    fn default() -> Self {
        Self {
            com: None,
            device: None,
            context: None,
            duplication: None,
            staging: None,
            monitor_index: 0,
            width: 0,
            height: 0,
            clock: MonotonicClock::new(),
            needs_keyframe: false,
        }
    }
}

impl DxgiCapturer {
    /// Create an idle capturer. Call [`Capturer::start`] before capturing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Current frame dimensions, or `(0, 0)` before `start`.
    #[must_use]
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Consume the pending keyframe hint.
    ///
    /// Set when the duplication interface had to be rebuilt after `ACCESS_LOST`: the
    /// decoder lost reference state, so the encoder must emit an IDR before P frames.
    pub fn take_needs_keyframe(&mut self) -> bool {
        std::mem::take(&mut self.needs_keyframe)
    }

    /// Build device, duplication and staging texture for `monitor_index`.
    fn attach(&mut self, monitor_index: u32) -> Result<()> {
        let device = self
            .device
            .as_ref()
            .ok_or_else(|| Error::Other("DXGI capturer has no device".into()))?;
        // SAFETY: `device` is a live D3D11 device from `create_device`; the cast only
        // queries a supported interface.
        let dxgi_device: IDXGIDevice = device
            .cast()
            .map_err(|e| Error::Unavailable(format!("DXGI: QI IDXGIDevice: {e}")))?;
        // SAFETY: interface call on a live DXGI device.
        let adapter = unsafe { dxgi_device.GetAdapter() }
            .map_err(|e| Error::Unavailable(format!("DXGI: GetAdapter: {e}")))?;

        let output = find_output(&adapter, monitor_index)?;
        // SAFETY: interface call on a live output.
        let desc = unsafe { output.GetDesc() }
            .map_err(|e| Error::Unavailable(format!("DXGI: output GetDesc: {e}")))?;
        if !desc.AttachedToDesktop.as_bool() {
            return Err(Error::Unavailable(format!(
                "DXGI: monitor {monitor_index} is not attached to the desktop"
            )));
        }
        if desc.Rotation != DXGI_MODE_ROTATION_IDENTITY {
            return Err(Error::Unavailable(format!(
                "DXGI: rotated outputs are not supported yet (rotation={}); see T-0605",
                desc.Rotation.0
            )));
        }

        let coords = desc.DesktopCoordinates;
        let width = u32::try_from(coords.right - coords.left)
            .map_err(|_| Error::Unavailable("DXGI: negative output width".into()))?;
        let height = u32::try_from(coords.bottom - coords.top)
            .map_err(|_| Error::Unavailable("DXGI: negative output height".into()))?;
        if width == 0 || height == 0 {
            return Err(Error::Unavailable(format!(
                "DXGI: monitor {monitor_index} reports an empty desktop area"
            )));
        }

        // SAFETY: interface call on a live output; `IDXGIOutput1` is its required
        // descendant interface on Windows 8+.
        let output1: IDXGIOutput1 = output
            .cast()
            .map_err(|e| Error::Unavailable(format!("DXGI: QI IDXGIOutput1: {e}")))?;
        // SAFETY: `device` has BGRA support and outlives the duplication object.
        let duplication =
            unsafe { output1.DuplicateOutput(device) }.map_err(|e| match e.code() {
                DXGI_ERROR_NOT_CURRENTLY_AVAILABLE => Error::Unavailable(
                    "DXGI: desktop duplication is not available (another exclusive capture app, \
                 a remote session, or the secure desktop is active)"
                        .into(),
                ),
                _ => Error::Unavailable(format!(
                    "DXGI: DuplicateOutput: {e} (monitor {monitor_index}, {width}x{height})"
                )),
            })?;

        let staging = StagingTexture::new(device, width, height)?;

        self.duplication = Some(duplication);
        self.staging = Some(staging);
        self.width = width;
        self.height = height;
        Ok(())
    }

    /// Rebuild the duplication interface after `DXGI_ERROR_ACCESS_LOST`.
    ///
    /// Windows hands out this error on desktop switch, resolution change, UAC/secure
    /// desktop and lock/unlock. The D3D11 device survives; the duplication and staging
    /// texture do not.
    fn recover(&mut self) -> Result<()> {
        tracing::warn!(
            monitor = self.monitor_index,
            "DXGI: access lost, rebuilding duplication"
        );
        self.duplication = None;
        self.staging = None;
        self.needs_keyframe = true;
        self.attach(self.monitor_index)
    }

    /// Copy the acquired desktop texture into a tightly packed BGRA buffer.
    ///
    /// Runs before `ReleaseFrame`, because the frame metadata (and the texture contents)
    /// are only valid while the frame is held.
    fn copy_acquired(
        &self,
        info: &DXGI_OUTDUPL_FRAME_INFO,
        resource: Option<&IDXGIResource>,
    ) -> Result<Option<crate::CapturedFrame>> {
        // Pointer-only update: the desktop image did not change, so there is nothing to
        // encode. Cursor delivery is T-0409.
        if info.LastPresentTime == 0 {
            return Ok(None);
        }
        let Some(resource) = resource else {
            return Ok(None);
        };
        let staging = self
            .staging
            .as_ref()
            .ok_or_else(|| Error::Other("DXGI capturer has no staging texture".into()))?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| Error::Other("DXGI capturer has no device context".into()))?;

        // SAFETY: `resource` is the acquired desktop surface, which is a 2D texture.
        let source_texture: ID3D11Texture2D = resource.cast().map_err(|e| {
            Error::Unavailable(format!("DXGI: acquired resource is not a texture: {e}"))
        })?;
        // SAFETY: live interface query on the acquired texture.
        let source: ID3D11Resource = source_texture
            .cast()
            .map_err(|e| Error::Unavailable(format!("DXGI: QI ID3D11Resource: {e}")))?;
        // SAFETY: both resources are live, same device, same dimensions and format.
        unsafe { context.CopyResource(&staging.resource, &source) };

        let mut mapped = windows::Win32::Graphics::Direct3D11::D3D11_MAPPED_SUBRESOURCE::default();
        // SAFETY: staging is a CPU-readable staging texture; mapping blocks until the copy
        // above has completed, which is exactly the synchronisation we need.
        unsafe {
            context.Map(
                &staging.resource,
                0,
                D3D11_MAP_READ,
                0,
                Some(&raw mut mapped),
            )
        }
        .map_err(|e| Error::Unavailable(format!("DXGI: Map staging texture: {e}")))?;

        let result = self.copy_mapped(&mapped, info);

        // SAFETY: balancing Unmap for the successful Map above; the slice created in
        // `copy_mapped` no longer exists.
        unsafe { context.Unmap(&staging.resource, 0) };
        result
    }

    fn copy_mapped(
        &self,
        mapped: &windows::Win32::Graphics::Direct3D11::D3D11_MAPPED_SUBRESOURCE,
        info: &DXGI_OUTDUPL_FRAME_INFO,
    ) -> Result<Option<crate::CapturedFrame>> {
        let row_pitch = usize::try_from(mapped.RowPitch)
            .map_err(|_| Error::Other("DXGI: impossible row pitch".into()))?;
        let height = usize::try_from(self.height)
            .map_err(|_| Error::Other("DXGI: impossible height".into()))?;
        let visible = row_pitch
            .checked_mul(height)
            .ok_or_else(|| Error::Other("DXGI: mapped size overflows usize".into()))?;
        if mapped.pData.is_null() {
            return Err(Error::Unavailable("DXGI: staging map returned null".into()));
        }
        // SAFETY: a mapped staging texture exposes at least RowPitch * Height readable
        // bytes until Unmap; the slice is confined to this call.
        let source = unsafe { std::slice::from_raw_parts(mapped.pData.cast::<u8>(), visible) };

        // One allocation per delivered frame. The owning `Vec<u8>` in `CapturedFrame`
        // gives us no way to recycle it; removing this allocation needs the P5 GPU /
        // buffer-pool path. See the T-0107 HANDOFF note.
        let len = (self.width as usize)
            .checked_mul(self.height as usize)
            .and_then(|px| px.checked_mul(BYTES_PER_PIXEL as usize))
            .ok_or_else(|| Error::Other("DXGI: frame size overflows usize".into()))?;
        let mut data = vec![0u8; len];
        copy_padded_bgra(&mut data, source, row_pitch, self.width, self.height)?;

        let dirty_rects = if info.TotalMetadataBufferSize > 0 {
            self.read_dirty_rects()
        } else {
            Vec::new()
        };

        Ok(Some(crate::CapturedFrame {
            width: self.width,
            height: self.height,
            format: crate::PixelFormat::Bgra8,
            // Timestamp of the CPU copy, not of the desktop present. A true capture
            // timestamp needs the DXGI present-time/QPC mapping (P4 telemetry).
            capture_ts_us: self.clock.now_us().0,
            dirty_rects,
            data,
        }))
    }

    /// Best-effort dirty-rect list for the frame that is currently acquired.
    ///
    /// An empty list means "whole frame" per [`crate::CapturedFrame::dirty_rects`], so a
    /// metadata failure degrades to a full-frame encode rather than a wrong sub-rect.
    fn read_dirty_rects(&self) -> Vec<(u32, u32, u32, u32)> {
        let Some(duplication) = self.duplication.as_ref() else {
            return Vec::new();
        };
        let mut buffer: Vec<RECT> = Vec::new();
        let mut required = 0u32;
        for _ in 0..3 {
            let capacity_bytes =
                u32::try_from(buffer.capacity() * std::mem::size_of::<RECT>()).unwrap_or(u32::MAX);
            // SAFETY: the buffer's allocation is at least `capacity_bytes`; the API writes
            // at most that many bytes and reports the real size in `required`.
            let call = unsafe {
                duplication.GetFrameDirtyRects(
                    capacity_bytes,
                    buffer.as_mut_ptr(),
                    &raw mut required,
                )
            };
            match call {
                Ok(()) => {
                    let count =
                        (required as usize / std::mem::size_of::<RECT>()).min(buffer.capacity());
                    // SAFETY: the API initialized `count` complete RECTs in place.
                    unsafe { buffer.set_len(count) };
                    return buffer
                        .iter()
                        .filter_map(|r| rect_to_dirty_rect(*r, self.width, self.height))
                        .collect();
                }
                Err(e) if e.code() == DXGI_ERROR_MORE_DATA => {
                    let needed = (required as usize / std::mem::size_of::<RECT>()) + 1;
                    buffer.reserve(needed.saturating_sub(buffer.capacity()));
                }
                Err(_) => return Vec::new(),
            }
        }
        Vec::new()
    }

    fn teardown(&mut self) {
        // Release in dependency order: duplication and staging reference the device, and
        // the COM apartment must go last.
        self.duplication = None;
        self.staging = None;
        self.context = None;
        self.device = None;
        self.com = None;
        self.width = 0;
        self.height = 0;
    }
}

impl Drop for DxgiCapturer {
    fn drop(&mut self) {
        self.teardown();
    }
}

impl crate::Capturer for DxgiCapturer {
    fn start(&mut self, monitor_index: u32) -> Result<()> {
        self.teardown();
        self.com = Some(ComApartment::init()?);
        let (device, context) = create_device()?;
        self.device = Some(device);
        self.context = Some(context);
        self.monitor_index = monitor_index;
        if let Err(e) = self.attach(monitor_index) {
            self.teardown();
            return Err(e);
        }
        self.needs_keyframe = true; // first frame must be an IDR in any case
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<Option<crate::CapturedFrame>> {
        let mut info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource: Option<IDXGIResource> = None;
        let acquire = {
            let duplication = self
                .duplication
                .as_ref()
                .ok_or_else(|| Error::Other("DXGI capturer used before start()".into()))?;
            // SAFETY: live duplication object; `info` and `resource` are valid out-params.
            // Timeout 0 keeps the call non-blocking, as the trait prefers.
            unsafe { duplication.AcquireNextFrame(0, &raw mut info, &raw mut resource) }
        };

        match acquire {
            Ok(()) => {}
            Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => return Ok(None),
            Err(e) if e.code() == DXGI_ERROR_ACCESS_LOST => {
                // The frame was not acquired, so there is nothing to release.
                self.recover()?;
                return Ok(None);
            }
            Err(e) => return Err(Error::Unavailable(format!("DXGI: AcquireNextFrame: {e}"))),
        }

        // A frame is held: every path from here must ReleaseFrame.
        let copied = self.copy_acquired(&info, resource.as_ref());
        let released = self
            .duplication
            .as_ref()
            .ok_or_else(|| Error::Other("DXGI: duplication vanished".into()))
            .and_then(|duplication| {
                // SAFETY: balances the successful AcquireNextFrame above.
                unsafe { duplication.ReleaseFrame() }
                    .map_err(|e| Error::Unavailable(format!("DXGI: ReleaseFrame: {e}")))
            });

        match copied {
            // A copy failure is the more useful error; still surface a release failure if
            // it is the only one.
            Err(e) => {
                let _ = released;
                Err(e)
            }
            Ok(frame) => {
                released?;
                Ok(frame)
            }
        }
    }

    fn stop(&mut self) -> Result<()> {
        self.teardown();
        Ok(())
    }

    fn take_needs_keyframe(&mut self) -> bool {
        std::mem::take(&mut self.needs_keyframe)
    }
}

fn create_device() -> Result<(ID3D11Device, ID3D11DeviceContext)> {
    let levels = [
        D3D_FEATURE_LEVEL_11_0,
        D3D_FEATURE_LEVEL_10_1,
        D3D_FEATURE_LEVEL_10_0,
    ];
    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;
    // SAFETY: no adapter (default hardware device), BGRA support is required by Desktop
    // Duplication, and both out-params are valid.
    unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&levels),
            D3D11_SDK_VERSION,
            Some(&raw mut device),
            None,
            Some(&raw mut context),
        )
    }
    .map_err(|e| Error::Unavailable(format!("DXGI: D3D11CreateDevice: {e}")))?;
    let device =
        device.ok_or_else(|| Error::Other("DXGI: D3D11CreateDevice returned no device".into()))?;
    let context = context
        .ok_or_else(|| Error::Other("DXGI: D3D11CreateDevice returned no context".into()))?;
    Ok((device, context))
}

/// Find the `monitor_index`-th desktop-attached output on `adapter`.
fn find_output(adapter: &IDXGIAdapter, monitor_index: u32) -> Result<IDXGIOutput> {
    let mut attached = 0u32;
    let mut next = 0u32;
    loop {
        // SAFETY: `EnumOutputs` is called with increasing indices until DXGI reports
        // out-of-range, which is the documented enumeration-stop condition.
        let Ok(output) = (unsafe { adapter.EnumOutputs(next) }) else {
            break;
        };
        next += 1;
        // SAFETY: live output interface.
        let Ok(desc) = (unsafe { output.GetDesc() }) else {
            continue;
        };
        if !desc.AttachedToDesktop.as_bool() {
            continue;
        }
        if attached == monitor_index {
            return Ok(output);
        }
        attached += 1;
    }
    Err(Error::InvalidArgument(format!(
        "DXGI: monitor index {monitor_index} not found on this adapter ({attached} attached)"
    )))
}

/// Copy a padded BGRA surface into a tightly packed buffer.
///
/// `source` may be the whole mapped texture (`row_pitch * height` bytes) or just enough
/// bytes for the visible rows.
fn copy_padded_bgra(
    destination: &mut [u8],
    source: &[u8],
    row_pitch: usize,
    width: u32,
    height: u32,
) -> Result<()> {
    let row_bytes = (width as usize)
        .checked_mul(BYTES_PER_PIXEL as usize)
        .ok_or_else(|| Error::Other("DXGI: row size overflows usize".into()))?;
    let rows = height as usize;
    let needed = row_bytes
        .checked_mul(rows)
        .ok_or_else(|| Error::Other("DXGI: frame size overflows usize".into()))?;
    if destination.len() != needed {
        return Err(Error::InvalidArgument(format!(
            "DXGI: destination is {} bytes, expected {needed}",
            destination.len()
        )));
    }
    if row_pitch < row_bytes {
        return Err(Error::InvalidArgument(format!(
            "DXGI: row pitch {row_pitch} is smaller than {row_bytes}"
        )));
    }
    let source_needed = if rows == 0 {
        0
    } else {
        row_pitch
            .checked_mul(rows - 1)
            .and_then(|n| n.checked_add(row_bytes))
            .ok_or_else(|| Error::Other("DXGI: source size overflows usize".into()))?
    };
    if source.len() < source_needed {
        return Err(Error::InvalidArgument(format!(
            "DXGI: source is {} bytes, need {source_needed} for {width}x{height} at pitch {row_pitch}",
            source.len()
        )));
    }
    for y in 0..rows {
        let start = y * row_pitch;
        let target = y * row_bytes;
        destination[target..target + row_bytes].copy_from_slice(&source[start..start + row_bytes]);
    }
    Ok(())
}

/// Convert a DXGI `RECT` (exclusive right/bottom) into a clipped `(x, y, w, h)` tuple.
fn rect_to_dirty_rect(rect: RECT, width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
    let left = rect.left.max(0);
    let top = rect.top.max(0);
    let right = rect.right.min(i32::try_from(width).ok()?);
    let bottom = rect.bottom.min(i32::try_from(height).ok()?);
    if right <= left || bottom <= top {
        return None;
    }
    Some((
        u32::try_from(left).ok()?,
        u32::try_from(top).ok()?,
        u32::try_from(right - left).ok()?,
        u32::try_from(bottom - top).ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn copies_visible_rows_and_skips_pitch_padding() {
        // 2x2 BGRA (8 bytes/row) at a 12-byte pitch: each row has 4 padding bytes.
        let source = [
            1, 2, 3, 4, 5, 6, 7, 8, 0xAA, 0xAA, 0xAA, 0xAA, // row 0
            9, 10, 11, 12, 13, 14, 15, 16, 0xBB, 0xBB, 0xBB, // last row may be short
        ];
        let mut destination = [0u8; 16];
        copy_padded_bgra(&mut destination, &source, 12, 2, 2).expect("copy");
        assert_eq!(
            destination,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        );
    }

    #[test]
    fn rejects_inconsistent_buffers() {
        let source = [0u8; 16];
        let mut destination = [0u8; 15];
        assert!(matches!(
            copy_padded_bgra(&mut destination, &source, 8, 2, 2),
            Err(Error::InvalidArgument(_))
        ));
        let mut destination = [0u8; 16];
        assert!(matches!(
            copy_padded_bgra(&mut destination, &source, 4, 2, 2),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            copy_padded_bgra(&mut destination, &[0u8; 10], 8, 2, 2),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn dirty_rects_are_clipped_to_the_surface() {
        assert_eq!(
            rect_to_dirty_rect(rect(0, 0, 10, 8), 10, 8),
            Some((0, 0, 10, 8))
        );
        assert_eq!(
            rect_to_dirty_rect(rect(-4, -2, 6, 4), 10, 8),
            Some((0, 0, 6, 4))
        );
        assert_eq!(
            rect_to_dirty_rect(rect(8, 6, 99, 99), 10, 8),
            Some((8, 6, 2, 2))
        );
        assert_eq!(rect_to_dirty_rect(rect(10, 0, 12, 4), 10, 8), None);
        assert_eq!(rect_to_dirty_rect(rect(-9, -9, 0, 0), 10, 8), None);
    }

    #[test]
    fn zero_sized_surfaces_are_rejected() {
        let mut destination = [0u8; 0];
        copy_padded_bgra(&mut destination, &[], 0, 0, 0).expect("empty frame");
        assert_eq!(rect_to_dirty_rect(rect(0, 0, 1, 1), 0, 0), None);
    }

    /// Manual check against the real desktop. Ignored in CI because it needs an
    /// interactive Windows session with a GPU-backed desktop.
    ///
    /// Run with desktop activity (a static desktop legitimately produces no frames):
    /// `cargo test -p vd-capture -- --ignored --nocapture`
    #[test]
    #[ignore = "requires an interactive Windows desktop and GPU"]
    fn dxgi_captures_primary_output_and_recovers() {
        use crate::Capturer;

        let mut capturer = DxgiCapturer::new();
        capturer.start(0).expect("DXGI start on monitor 0");
        let (width, height) = capturer.dimensions();
        assert!(width > 0 && height > 0, "output must have a size");
        assert!(
            capturer.take_needs_keyframe(),
            "the first frame after start must be an IDR"
        );

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut frames = 0u32;
        let mut samples: Vec<u64> = Vec::new();
        while std::time::Instant::now() < deadline && frames < 60 {
            let started = std::time::Instant::now();
            match capturer.capture_frame().expect("capture") {
                Some(frame) => {
                    samples.push(u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX));
                    assert_eq!((frame.width, frame.height), (width, height));
                    assert_eq!(frame.format, crate::PixelFormat::Bgra8);
                    assert_eq!(frame.data.len(), (width as usize) * (height as usize) * 4);
                    for (x, y, w, h) in &frame.dirty_rects {
                        assert!(x + w <= width && y + h <= height, "dirty rect in bounds");
                    }
                    frames += 1;
                }
                None => std::thread::sleep(std::time::Duration::from_millis(5)),
            }
        }
        assert!(
            frames > 0,
            "no desktop frame within 5 s — the desktop may be static; create activity and retry"
        );
        if !samples.is_empty() {
            samples.sort_unstable();
            let mean = samples.iter().sum::<u64>() / samples.len() as u64;
            let p50 = samples[samples.len() / 2];
            let max = samples[samples.len() - 1];
            println!(
                "dxgi capture at {width}x{height}: {} frames, mean {mean} us, p50 {p50} us, max {max} us",
                samples.len()
            );
        }

        // Exercise the same rebuild path that ACCESS_LOST triggers. A real desktop
        // switch/lock cannot be automated safely, so recovery is verified by calling it.
        capturer.recover().expect("rebuild duplication");
        assert!(
            capturer.take_needs_keyframe(),
            "recovery must force a keyframe"
        );
        assert!(
            !capturer.take_needs_keyframe(),
            "hint must be consumed once"
        );

        capturer.stop().expect("stop");
        assert_eq!(capturer.dimensions(), (0, 0), "stop releases the surface");
    }
}
