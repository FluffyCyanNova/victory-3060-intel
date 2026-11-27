//! Textures shaders write that the title's own code then reads on the CPU.
//! On the console the CPU and the GPU share memory, so what a compute pass
//! stores into a texture is in title memory for the CPU to read. Here it stays
//! in the texture's host copy. The title's ocean is one: its wave passes write
//! a 256x256 height map (R32_FLOAT) and a displacement map (R32G32_FLOAT), and
//! its CPU code samples both, which is what boats float on; it read memory
//! nothing had written (run rw1, `VICTORY_READ_WATCH=1`: about 30 places in
//! the title read those two kinds), and boats sank.
//!
//! For textures of the kinds in `VICTORY_TEXTURE_BACK` (comma-separated
//! `<width>x<height>f<format>`; `1` for the four water formats found by
//! the upstream trace), a list in which shaders write one copies it
//! into a read-back buffer of ours when it closes, and once the list has run
//! the texels are tiled into the texture's title memory in the console's
//! layout, as the console's GPU would have left them.
//!
//! shortcut: by kind, not by texture: textures of those kinds the title never
//! reads are copied back too (about a megabyte per such list). The copy shows
//! what the list's last write left; a title read in between sees the frame
//! before, as it would before the console's GPU has run.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use windows::Win32::Graphics::Direct3D12::*;

use crate::object::This;

#[cfg(test)]
thread_local! {
    pub(crate) static TEST_KINDS: std::cell::Cell<Option<&'static [(u64, u32, u32)]>> =
        const { std::cell::Cell::new(None) };
}

/// The kinds copied back: (width, height, format).
fn kinds() -> &'static [(u64, u32, u32)] {
    #[cfg(test)]
    if let Some(kinds) = TEST_KINDS.get() {
        return kinds;
    }
    static KINDS: OnceLock<Vec<(u64, u32, u32)>> = OnceLock::new();
    KINDS.get_or_init(|| {
        const DEFAULT: &str = "256x256f41,256x256f16,64x64f16,128x128f36";
        let list = match std::env::var("VICTORY_TEXTURE_BACK").as_deref() {
            Ok("1") => DEFAULT.to_owned(),
            Ok(spec) => spec.to_owned(),
            Err(_) => String::new(),
        };
        let kinds: Vec<(u64, u32, u32)> = list
            .split(',')
            .filter_map(|k| {
                let (size, format) = k.trim().split_once('f')?;
                let (w, h) = size.split_once('x')?;
                Some((w.parse().ok()?, h.parse().ok()?, format.parse().ok()?))
            })
            .collect();
        if !kinds.is_empty() {
            tracing::info!(?kinds, "textures of these kinds that shaders write are copied back into title memory (VICTORY_TEXTURE_BACK)");
        }
        kinds
    })
}

/// The water's three min-max programs (`water:CS_MinMax_Int_SinglePlane_Parallel`,
/// `CS_MinMax_Float_DualPlane_Parallel`, `CS_MinMax_Float_SinglePlane_Parallel`):
/// what `VICTORY_TEXTURE_BACK_PROGRAMS=1` names.
const WATER_MIN_MAX: [&str; 3] = ["bdf438d5731d36cc", "9809b8d965b04935", "a2f1004f8d6d7aad"];

/// The programs whose every texture write is copied back, as their
/// shaders' hashes or the first digits of them (at least eight), in lower
/// case: `spec` is `VICTORY_TEXTURE_BACK_PROGRAMS`'s value, a
/// comma-separated list, or `1` for [`WATER_MIN_MAX`]. What is not eight
/// to sixteen hex digits is left out.
fn parse_programs(spec: &str) -> Vec<String> {
    if spec.trim() == "1" {
        return WATER_MIN_MAX.iter().map(|h| (*h).to_owned()).collect();
    }
    spec.split(',')
        .map(|h| h.trim().to_ascii_lowercase())
        .filter(|h| (8..=16).contains(&h.len()) && h.bytes().all(|b| b.is_ascii_hexdigit()))
        .collect()
}

/// By writing program (`VICTORY_TEXTURE_BACK_PROGRAMS`; unset: none): every
/// texture a draw or dispatch of one of these programs writes through
/// unordered access is copied back, whatever its size and format.
///
/// The kinds of `VICTORY_TEXTURE_BACK` choose by size and format alone, and
/// the list found by the upstream trace holds the water's height and
/// displacement maps and its min-max maps (128x128, 64x64) but not the
/// planes of indices the same min-max programs write beside them
/// (2048x128, 4096x64): the title's CPU got the one without the other. (A
/// whole patch of the water's surface went undrawn now and then, run 69;
/// whether this is why is not shown yet.) Chosen by program, nothing a
/// program writes is left out, and nothing another program writes in the
/// same shape is taken along.
fn programs() -> &'static [String] {
    static PROGRAMS: OnceLock<Vec<String>> = OnceLock::new();
    PROGRAMS.get_or_init(|| {
        let programs = std::env::var("VICTORY_TEXTURE_BACK_PROGRAMS").map_or_else(|_| Vec::new(), |v| parse_programs(&v));
        if !programs.is_empty() {
            tracing::info!(?programs, "every texture these programs write is copied back into title memory (VICTORY_TEXTURE_BACK_PROGRAMS)");
        }
        programs
    })
}

/// Whether a pipeline named `name` (its shaders, as `[compute <hash>]`)
/// has one of `programs`.
fn named(name: &str, programs: &[String]) -> bool {
    programs.iter().any(|p| name.contains(p.as_str()))
}

thread_local! {
    /// Whether the pipeline whose tables this thread walks is one of
    /// [`programs`] ([`writer`]).
    static BY_PROGRAM: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// A draw or dispatch of the pipeline named `name` is being prepared on
/// this thread: what it writes is judged by it until the next one
/// ([`wanted`]). Nothing but a look at a flag while no program is named.
pub(crate) fn writer(name: &str) {
    let programs = programs();
    if !programs.is_empty() {
        BY_PROGRAM.set(named(name, programs));
    }
}

/// Whether title texture `resource`, written by the draw or dispatch this
/// thread prepares, is copied back: it is of a kind copied back, or the
/// program is one whose writes are ([`writer`]).
pub(crate) fn wanted(resource: This) -> bool {
    let kinds = kinds();
    let by_program = BY_PROGRAM.get();
    if kinds.is_empty() && !by_program {
        return false;
    }
    crate::resource::texture_kind(resource).is_some_and(|k| by_program || kinds.contains(&k))
}

/// Whether anything is copied back (`VICTORY_TEXTURE_BACK` or
/// `VICTORY_TEXTURE_BACK_PROGRAMS` set).
#[cfg(feature = "diagnostics")]
pub(crate) fn on() -> bool {
    !kinds().is_empty() || !programs().is_empty()
}

/// Diagnostics builds, with `VICTORY_TEXTURE_BACK` set: says once per kind
/// and pipeline which pipeline writes a texture of a kind copied back
/// (`pipeline`: the one whose tables the recording thread walked last).
/// The kinds are chosen by size and format alone, so this is how a kind
/// that is not the ocean's is told from one that is.
#[cfg(feature = "diagnostics")]
pub(crate) fn note_writer(resource: This, pipeline: &str) {
    /// Pairs said already; no more than this many are kept (or said).
    const MOST: usize = 256;
    static SAID: Mutex<Vec<((u64, u32, u32), String)>> = Mutex::new(Vec::new());
    let kinds = kinds();
    let by_program = BY_PROGRAM.get();
    if kinds.is_empty() && !by_program {
        return;
    }
    let Some(kind) = crate::resource::texture_kind(resource).filter(|k| by_program || kinds.contains(k)) else {
        return;
    };
    let mut said = SAID.lock().unwrap_or_else(|e| e.into_inner());
    if said.len() >= MOST || said.iter().any(|(k, p)| *k == kind && p == pipeline) {
        return;
    }
    said.push((kind, pipeline.to_owned()));
    if !kinds.contains(&kind) {
        // (its own text: the line below is searched for by its words)
        tracing::info!(
            width = kind.0,
            height = kind.1,
            format = kind.2,
            resource = format_args!("{:#x}", resource as usize),
            frame = crate::frame::current(),
            pipeline,
            "texture back: a texture copied back for the program that writes it (VICTORY_TEXTURE_BACK_PROGRAMS)"
        );
        return;
    }
    tracing::info!(
        width = kind.0,
        height = kind.1,
        format = kind.2,
        resource = format_args!("{:#x}", resource as usize),
        frame = crate::frame::current(),
        pipeline,
        "texture back: a texture of a kind copied back is written by this pipeline"
    );
}

/// An outstanding write to a specific allocation. Mapping and resource
/// changes invalidate only the writes that could touch the replaced memory.
struct Ticket {
    owner: usize,
    at: usize,
    end: usize,
    valid: AtomicBool,
}

static TICKETS: Mutex<Vec<Weak<Ticket>>> = Mutex::new(Vec::new());
static TRACKING: AtomicBool = AtomicBool::new(false);

fn track(owner: usize, at: usize, end: usize) -> Arc<Ticket> {
    let ticket = Arc::new(Ticket { owner, at, end, valid: AtomicBool::new(true) });
    let mut tickets = TICKETS.lock().unwrap_or_else(|e| e.into_inner());
    tickets.retain(|weak| weak.strong_count() != 0);
    tickets.push(Arc::downgrade(&ticket));
    TRACKING.store(true, Ordering::Relaxed);
    ticket
}

fn invalidate(matches: impl Fn(&Ticket) -> bool) {
    if !TRACKING.load(Ordering::Relaxed) {
        return;
    }
    TICKETS.lock().unwrap_or_else(|e| e.into_inner()).retain(|weak| {
        let Some(ticket) = weak.upgrade() else { return false };
        if matches(&ticket) {
            ticket.valid.store(false, Ordering::Release);
        }
        true
    });
}

pub(crate) fn released(owner: usize) {
    invalidate(|ticket| ticket.owner == owner);
}

pub(crate) fn replaced(at: usize) {
    invalidate(|ticket| ticket.at == at);
}

pub(crate) fn remapped(at: usize, len: usize) {
    let end = at.saturating_add(len);
    invalidate(|ticket| at < ticket.end && ticket.at < end);
}

pub(crate) struct Target {
    pub(crate) copy: Arc<crate::resource::HostTexture>,
    layout: victory_formats::texture::Layout,
    ticket: Arc<Ticket>,
}

impl Target {
    pub(crate) fn new(resource: This) -> Option<Self> {
        // Keep the host copy independently of the title's resource lifetime.
        let copy = crate::resource::host_copy(resource)?;
        let _mapping = crate::watch::title_memory_reading();
        let (layout, at, size) = crate::resource::title_texture(resource)?;
        let end = at.checked_add(size).filter(|&end| at != 0 && end > at)?;
        let ticket = track(resource as usize, at, end);
        Some(Self { copy, layout, ticket })
    }

    pub(crate) fn owner(&self) -> usize { self.ticket.owner }

    /// Whether the ticket is still valid: the texture's memory has not been
    /// reused or remapped since it was registered. A stale target is
    /// re-registered so the read-back tracks the memory the texture is in
    /// now, not where it was when it was last written.
    pub(crate) fn fresh(&self) -> bool { self.ticket.valid.load(Ordering::Acquire) }
}

/// The readback resources and the allocation the pixels belong to.
pub(crate) struct Back {
    buffer: ID3D12Resource,
    footprints: Vec<D3D12_PLACED_SUBRESOURCE_FOOTPRINT>,
    rows: Vec<u32>,
    mips: u32,
    layout: victory_formats::texture::Layout,
    ticket: Arc<Ticket>,
}

impl Back {
    pub(crate) fn range(&self) -> (usize, usize) {
        (self.ticket.at, self.ticket.end)
    }
}

/// Records a host copy from a texture already in COPY_SOURCE.
pub(crate) fn record(list: &ID3D12GraphicsCommandList, target: Target) -> Option<Back> {
    if !target.ticket.valid.load(Ordering::Acquire) {
        return None;
    }
    let host = crate::host::host()?;
    let texture = &target.copy.texture;
    // SAFETY: a live texture kept through the command list's state tracking.
    let desc = unsafe { texture.GetDesc() };
    let mips = u32::from(desc.MipLevels.max(1));
    let subresources = mips * u32::from(desc.DepthOrArraySize.max(1));
    let n = subresources as usize;
    let mut footprints = vec![D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default(); n];
    let mut rows = vec![0u32; n];
    let mut total = 0u64;
    // SAFETY: out arrays of `n` entries.
    unsafe {
        host.device.GetCopyableFootprints(
            &desc,
            0,
            subresources,
            0,
            Some(footprints.as_mut_ptr()),
            Some(rows.as_mut_ptr()),
            None,
            Some(&mut total),
        )
    };
    // The last footprint's advertised extent can omit row-end padding.
    // The tiler reads a pitched slice, so the mapped buffer must include it.
    for (footprint, &rows) in footprints.iter().zip(&rows) {
        let format = footprint.Footprint;
        let pitched = u64::from(format.RowPitch)
            .checked_mul(u64::from(rows))?
            .checked_mul(u64::from(format.Depth))?;
        total = total.max(footprint.Offset.checked_add(pitched)?);
    }
    let buffer = host.read_back_buffer(total)?;
    for (i, footprint) in footprints.iter().enumerate() {
        let to = D3D12_TEXTURE_COPY_LOCATION {
            // SAFETY: a borrowed pointer the location never releases.
            pResource: unsafe { std::mem::transmute_copy(&buffer) },
            Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { PlacedFootprint: *footprint },
        };
        let from = D3D12_TEXTURE_COPY_LOCATION {
            // SAFETY: as above.
            pResource: unsafe { std::mem::transmute_copy(texture) },
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: i as u32 },
        };
        // SAFETY: an open list; the texture is in the copy-source state.
        unsafe { list.CopyTextureRegion(&to, 0, 0, 0, &from, None) };
    }
    Some(Back { buffer, footprints, rows, mips, layout: target.layout, ticket: target.ticket })
}

/// What [`note_untiled`] counts by: a texture's width, height, bytes per
/// element and tiling mode, and why it could not be tiled.
type Untiled = (u32, u32, u32, u32, String);

/// The count of `kind` after one more, among `seen` (at most `most` kinds
/// are kept: one more than that is not counted, `None`).
fn count_untiled(seen: &mut Vec<(Untiled, u64)>, kind: Untiled, most: usize) -> Option<u64> {
    if let Some((_, n)) = seen.iter_mut().find(|(k, _)| *k == kind) {
        *n += 1;
        return Some(*n);
    }
    if seen.len() >= most {
        return None;
    }
    seen.push((kind, 1));
    Some(1)
}

/// A texture copied back whose texels could not be put into title memory
/// (`Layout::tile` failed; nothing is written for it): says which kind of
/// texture, the first time and at every doubling of its count. (It was
/// said once for all kinds together, without the texture: run 70 could
/// not tell which of three planes it was, or whether all three.)
fn note_untiled(desc: &victory_formats::texture::Desc, why: &str, mip: u32, slice: u32) {
    /// Kinds kept.
    const MOST: usize = 64;
    static SEEN: Mutex<Vec<(Untiled, u64)>> = Mutex::new(Vec::new());
    let kind = (desc.width, desc.height, desc.element_bytes, desc.swizzle.0, why.to_owned());
    let count = count_untiled(&mut SEEN.lock().unwrap_or_else(|e| e.into_inner()), kind, MOST);
    if let Some(count) = count.filter(|n| n.is_power_of_two()) {
        tracing::warn!(
            e = why,
            mip,
            slice,
            width = desc.width,
            height = desc.height,
            element_bytes = desc.element_bytes,
            tiling = desc.swizzle.0,
            count,
            "a texture copied back could not be tiled"
        );
    }
}

/// Once the list has run, tile its texels back into the same title allocation.
pub(crate) fn land(back: &Back) {
    static COPIED: AtomicU64 = AtomicU64::new(0);
    static STALE: AtomicU64 = AtomicU64::new(0);
    let _mapping = crate::watch::title_memory_reading();
    let _ticket_guard = TICKETS.lock().unwrap_or_else(|e| e.into_inner());
    if !back.ticket.valid.load(Ordering::Acquire) {
        let skipped = STALE.fetch_add(1, Ordering::Relaxed) + 1;
        if skipped.is_power_of_two() {
            tracing::info!(skipped, "skipping texture readbacks whose title allocation changed");
        }
        return;
    }
    let (at, end) = back.range();
    let size = end - at;
    let mut tiled = vec![0u8; size];
    // Keep title bytes the subresources do not cover.
    if !crate::host::read_into(at as *const u8, &mut tiled) {
        return;
    }
    let mut data = std::ptr::null_mut();
    // SAFETY: our read-back buffer, written by the list that has run.
    unsafe {
        if back.buffer.Map(0, None, Some(&mut data)).is_err() || data.is_null() {
            return;
        }
    }
    for (i, (footprint, &rows)) in back.footprints.iter().zip(&back.rows).enumerate() {
        let (mip, slice) = (i as u32 % back.mips, i as u32 / back.mips);
        let f = footprint.Footprint;
        let pitch = f.RowPitch as usize;
        let len = pitch * rows as usize * f.Depth as usize;
        // SAFETY: the subresource's part of the mapped buffer.
        let linear = unsafe { std::slice::from_raw_parts(data.cast::<u8>().add(footprint.Offset as usize), len) };
        if let Err(e) = back.layout.tile(&mut tiled, mip, slice, linear, pitch, pitch * rows as usize) {
            note_untiled(&back.layout.desc, &format!("{e:?}"), mip, slice);
            // SAFETY: mapped above.
            unsafe { back.buffer.Unmap(0, None) };
            return;
        }
    }
    // SAFETY: mapped above.
    unsafe { back.buffer.Unmap(0, None) };
    crate::host::copy_mapped(at as *mut u8, tiled.as_ptr(), size, true);
    let copied = COPIED.fetch_add(1, Ordering::Relaxed) + 1;
    if copied == 1 || copied.is_multiple_of(128) {
        tracing::info!(copied, bytes = size, "texture readback landed");
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_readbacks_for_changed_allocations_are_invalidated() {
        let first = track(91, 0x1000, 0x2000);
        let next = track(92, 0x2000, 0x3000);
        let far = track(93, 0x4000, 0x5000);

        remapped(0x0f00, 0x200);
        assert!(!first.valid.load(Ordering::Acquire));
        assert!(next.valid.load(Ordering::Acquire));
        replaced(0x2000);
        assert!(!next.valid.load(Ordering::Acquire));
        assert!(far.valid.load(Ordering::Acquire));

        released(93);
        assert!(!far.valid.load(Ordering::Acquire));
        assert!(track(94, 0x2000, 0x3000).valid.load(Ordering::Acquire));
    }

    /// Textures that could not be tiled are counted by kind, and no more
    /// kinds than the limit are kept.
    #[test]
    fn untiled_textures_are_counted_by_kind() {
        let kind = |w: u32, tiling: u32| (w, 64, 1, tiling, "Unsupported".to_owned());
        let mut seen = Vec::new();
        assert_eq!(count_untiled(&mut seen, kind(4096, 9), 2), Some(1));
        assert_eq!(count_untiled(&mut seen, kind(4096, 9), 2), Some(2));
        // another tiling mode of the same size is another kind
        assert_eq!(count_untiled(&mut seen, kind(4096, 3), 2), Some(1));
        assert_eq!(count_untiled(&mut seen, kind(4096, 9), 2), Some(3));
        // the limit is reached: a third kind is not counted, the known ones are
        assert_eq!(count_untiled(&mut seen, kind(2048, 9), 2), None);
        assert_eq!(count_untiled(&mut seen, kind(4096, 3), 2), Some(2));
        assert_eq!(seen.len(), 2);
    }

    /// `VICTORY_TEXTURE_BACK_PROGRAMS`: `1` names the water's three min-max
    /// programs; else hashes or their first digits, and nothing else.
    #[test]
    fn the_programs_copied_back_are_named_by_their_hashes() {
        assert_eq!(parse_programs("1"), WATER_MIN_MAX);
        assert_eq!(parse_programs(" 1 "), WATER_MIN_MAX);
        assert_eq!(
            parse_programs("BDF438D5731D36CC, 9809b8d9 ,a2f1004f8d6d7aad"),
            ["bdf438d5731d36cc", "9809b8d9", "a2f1004f8d6d7aad"]
        );
        // not hashes: too short, too long, not hex, a kind of the other switch
        assert!(parse_programs("bdf438d,bdf438d5731d36cc0,water,256x256f41,").is_empty());
        assert!(parse_programs("").is_empty());
        assert!(parse_programs("0").is_empty());
    }

    /// A pipeline is one of the programs when its name holds a hash of
    /// theirs: a compute pipeline's, or any stage's of a drawing one.
    #[test]
    fn a_pipeline_is_told_by_its_shaders_hashes() {
        let programs = parse_programs("1");
        assert!(named("[compute bdf438d5731d36cc]", &programs));
        assert!(named("[compute a2f1004f8d6d7aad]", &programs));
        assert!(!named("[compute ee153b918d4ef2c9]", &programs));
        assert!(!named("[mesh b7a81f5b96f4ca93, pixel ee78da6862364ddd]", &programs));
        assert!(named("[vertex b3ba0ef9ee352720, pixel 7115e46a79c78f98]", &parse_programs("7115e46a")));
        assert!(!named("[compute bdf438d5731d36cc]", &[]));
    }

    /// Without a program named, a draw's name changes nothing: the kinds
    /// alone decide, as before.
    #[test]
    fn no_program_named_leaves_the_kinds_to_decide() {
        // (the test process has no `VICTORY_TEXTURE_BACK_PROGRAMS`)
        if !programs().is_empty() {
            return;
        }
        writer("[compute bdf438d5731d36cc]");
        assert!(!BY_PROGRAM.get());
        assert!(!wanted(std::ptr::null_mut()));
    }
}
