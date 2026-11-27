# VIctory: local patch series

A patch series, not a copy of any tree. It holds only the changes made or
ported locally on top of a public commit of the fork. No game data, no
shader containers, no captures, no third-party tree.

Licence: GPL-3.0, the same as VIctory (see `LICENSE`). Every patch keeps
the licence of the tree it applies to.

## Base

| What | Where |
|---|---|
| Upstream project | https://github.com/Marvisak/VIctory |
| Fork the series applies to | https://github.com/fxceless/victory-fork (standalone repo, GPL-3.0), commit `9376fd60c0ba0e76a74c39b49b1407dbac0a64c7` |

## Lines

| Folder | Result | What it is |
|---|---|---|
| `patches/b44/` 0001-0027 | b40 base (called P1 in the notes) | fork `9376fd60` + the ports of the earlier fast line: validator 1.8 for compute derivatives, NaN position at the loop limit, view memory in place (switch), descriptor size and view-address fixes, per-stage loop limit, sampler and signature fixes, texture fixes |
| `patches/b44/` 0028-0030 | capture | window size, typeless captures, capture every N frames (also alone in `patches/capture/`) |
| `patches/b44/` 0031-0032 | b43 | Series S keeps its own upscaler; the title patches apply on Series S only |
| `patches/b44/` 0033-0043 | b44 | walls fix (no blending on integer render targets), Series S interface plane, and switches that are off by default |
| `patches/b45-prep/` 0001-0002 | b45-prep | exact tiled cascade shadows on 32-lane hosts; texture upload thread (switch). NOT compiled or tested when this was written |

Added 2026-11-27. Each folder below applies on top of `b44/` (plus `b44a/` where its base says so), in the order given. All compiled and unit-tested; "game run" says whether it has been played.

| Folder | Base | What it is | Game run |
|---|---|---|---|
| `patches/b44a/` | b44 | a frame to be saved is read with both planes (capture fix) | yes, the main line |
| `patches/b45/` | b44a | exact tiled cascade shadows on 32-lane hosts (fork #1002); texture upload thread `VICTORY_HEAP_WORKER=1` (off) | yes (interiors dark that run, cause not settled) |
| `patches/b46/` | b44a | placed-texture head cap: the block a newer view replaces is never longer than the texture | yes (interiors darker; not adopted) |
| `patches/b46b/` | b44a | b46 + a diagnostics line listing placed textures under 4 KiB | no |
| `patches/b47/` | b44a | physical pages from sections of up to 2 MiB (`VICTORY_SECTION_MB`, default 2) + a DirectStorage 5 s summary | yes: remap wait ~0, furthest run |
| `patches/b48/` | b44a | DirectStorage: one file handle per worker (`VICTORY_DSTORAGE_HANDLES=0` = as before); kept buffers per worker | no |
| `patches/b49/` | b44a | range-scoped mapping lock (fork #95) + a compile fix for this line | no |
| `patches/b50/` | b44a | fork #190 + #461 + #563: the two 64-lane water volumetrics programs build their pipelines | no (offline check only) |
| `patches/b51/` | b44a | texture copy-back for textures the title reads on the CPU (fork #999, `VICTORY_TEXTURE_BACK=1`), save log line, diagnostics line naming each copied kind's writer | yes: interiors went dark with all four default kinds; under study |
| `patches/b52a/` | b44a | game-save fixes (upstream #1729, #1667, #3104, #2511, #1981) + a save log line | no |
| `patches/b52b/` | b44a | clean close: constrain before suspend (#1080) + stand-in title window (#1677) | no |
| `patches/b53/` | b47 | hand ports of fork switches `VICTORY_HEAP_LOCKS` and `VICTORY_CARD_SYNC_BATCH` (both off by default) | no |
| `patches/b54/` | b47 | fork #827 commit 1 (coalesced refreshes, `COPY_FAST` off), a guard for its write-watch reset, and a counter of partly copied pages | no |
| `patches/b55/` | b47 | NaN/infinity scrub of shader outputs (`VICTORY_NAN_SCRUB=1`, off) | no |
| `patches/b56/` | b47 | relabelled compute threads take their ids from a counter (`VICTORY_THREAD_TICKET=1`, off) + a guard for groups whose shared memory is full | yes: vertex spikes not fixed; suspected of a flat red colour fault; not adopted |
| `patches/b57/` | b47 | flights get more room before they are settled (`VICTORY_SETTLE_ROOM=1`, off) | yes: ~14% cheaper per draw; kept |
| `patches/b58/` | b47 | head-block cap of fork #1391 behind a switch (`VICTORY_HEAD_CAP=1`, off) | yes: no visible gain |
| `patches/b59a/` | b47 | diagnostics: dump what device methods 0x1c8 and 0x218 are handed (`VICTORY_PIPELINE_X_DUMP=<folder>`, off) | yes (diagnostic) |
| `patches/b59c/` | b47 | b59a + diagnostics: log which shaders reach us and by which way (`VICTORY_SHADER_HASHES=<file>`, off) | no |
| `patches/b60/` | b47 | the two 64-lane water volumetrics programs translate on 32-lane hosts (scc_some lowering, f64 cross-half guard, DOUBLES flag) | yes (in b65): water renders |
| `patches/b62/` | b47 | tiled cascade shadow programs translate exactly on 32-lane hosts (fork #1002) | yes (in b65; that program skipped in the run) |
| `patches/b63/` | b47 | image and buffer lowering moved out of lower.rs (no behaviour change; DXIL byte-identical on the corpus) | no |
| `patches/b65/` | b47 | combined boat build: b63 split + DXIL sums tool + b60 + b62 + b56 + b55 + b57 + texture copy-back (fork #999, `VICTORY_TEXTURE_BACK`); all switches off by default | yes: with TEXTURE_BACK=1 + SETTLE_ROOM=1 the boat floats, water renders; 3.35 fps; one crash while driving |
| `patches/b67/` | b65 | capture trigger file (`VICTORY_CAPTURE_TRIGGER=<file>`: the next 3 frames get their operations written out) + saved frames keep the free present (`VICTORY_CAPTURE_LATE=1`, untested); both off | yes: boat crash repeated (fixed in b68) |
| `patches/b67b/` | b67 | capture ops files: a late line from another recording thread no longer empties a frame's file | yes (b68+) |
| `patches/b68/` | b67b | fork #1376 re-register invalidated texture readback tickets: copy-backs of reused/remapped water textures are no longer dropped (fixed the boat-mission crash) | yes: boat mission passed, 48.6 min no crash |
| `patches/b69/` | b68 | raw buffer views off 16 bytes get their own copy (`VICTORY_RAW_OWN_COPY=1`, off) + refused views named in the log | yes: no views left null |
| `patches/b70/` | b69 | textures copied back by the program that writes them (`VICTORY_TEXTURE_BACK_PROGRAMS`, off) | yes: did not stop the water-surface holes (dropped) |
| `patches/b72/` | b70 | VIctory-NVIDIA PR #58 (regiumlepidi, GPL-3.0) picked: placed views ignore the unwritten padding words 1/13 and unused view words in the view-table key (always on), and at most `VICTORY_OWNERLESS_VIEWS` (default 1,048,576; 0 = all) entries of views no resource releases are kept | compiled + tested (victory-gpu 168/0 plain, 178/0 diagnostics); no game run yet |
| `patches/b72-guard/` | b72 | victory-shader build.rs reads its folder at run time and stops the build when it finds no source file (after a folder move it had hashed nothing: empty translator hash, stale shader cache key) | built (b72); no game run yet |
| `patches/b74/` | b72-guard | idle texture copies stay while the card has room (`VICTORY_TEXTURE_SWEEP_PRESSURE=1`, off) + the "could not be tiled" warning names and counts each kind | built; no game run yet |
| `patches/b75/` | b74 | buffer views off 16 bytes / structured views of stride != 4 made as structured views of 4-byte elements (`VICTORY_DWORD_VIEWS=1`, off; card probe on an RTX 3060 passed) | built; no game run yet |

Notes: these are patches only, made from public sources (the fork, upstream PRs) and our own changes. No game data, captures, shader containers or binaries.

`patches/capture/` repeats b44's 0028-0030 so they can be put on any other base.

## How to apply

```
git clone <the public fork> victory && cd victory
git checkout -b local 9376fd60c0ba0e76a74c39b49b1407dbac0a64c7
git am /path/to/patches/b44/*.patch
git am /path/to/patches/b45-prep/*.patch     # optional, untested
cargo build --profile diag -p victory --features diagnostics
```

Checked when the series was made: `git am` of `b44` on `9376fd60` gives the
same tree as the b44 source, and `b45-prep` on top gives the same tree as
its source. Tests on the b44 tree: victory-gpu 154 (one at a time),
victory-shader 227, victory-hle 98 + 1, victory-loader 25 + 2, 0 failed.

## Switches added by the series

All are environment variables. "off" means nothing changes unless set.

| Switch | Default | What it does |
|---|---|---|
| `VICTORY_INPLACE_VIEWS=1` | off | view memory in place |
| `VICTORY_SHADER_LOOP_LIMIT_CS=<n>` | compute 4,096 | loop limit for compute programs alone |
| `VICTORY_WINDOW=<w>x<h>` | off | size of the title's window |
| `VICTORY_CAPTURE_EVERY=<n>`, `VICTORY_CAPTURE_MIN_DRAWS=<n>` | off | capture every n-th frame (diagnostics build) |
| `VICTORY_TITLE_PATCHES=0` / `=1` | Series S only | known-build title patches: none / also on Series X |
| `VICTORY_TITLE_PATCH=<rva>:<old>:<new>` | none | extra title patch |
| `VICTORY_BACK_QUIET=1` | off | a copy-back's own write is not read as a title write |
| `VICTORY_BACK_OWN_VIEW=1` or `alt` | off | copy-backs into view memory through a view of our own |
| `VICTORY_AMBIENT=<level>` | off | a constant stands in for indirect light |
| `VICTORY_FAIR_DUE=1` | off | texture queue serves old and new slots in turns |
| `VICTORY_RELOOK_MS=<ms>` | 4,000 | how long before an unmapped slot is looked at again |
| `VICTORY_PLANES=0` | on with two planes | show the first present plane only |
| `VICTORY_PLANE_ALPHA=straight` | premultiplied | alpha of the second plane |
| `VICTORY_SKIP_WC_BACK=1` | off | experiment: write-combining memory is not fetched back |
| `VICTORY_SKIP_WC_COMPARE=1` | off | UNSAFE experiment, a device loss was reported |
| `VICTORY_CENSUS=1`, `VICTORY_CENSUS_SECONDS=<s>` | off | census of card copies in the log |
| `VICTORY_KEEP_GONE=1` or `<frames>` | off | a streamed-out texture keeps its last view |
| `VICTORY_HEAP_WORKER=1` | off | (b45-prep) textures picked by index uploaded on their own thread; needs `VICTORY_LATE_WAIT` |

## Credit

Patch headers carry a neutral author on purpose. The work is credited
here. "local" means written for this series; "hand port" means the idea
and most of the code are the named author's, fitted to this base by hand.

| Patch | Subject | Author |
|---|---|---|
| b44/0001 | Local: compute programs with implicit derivatives | local |
| b44/0002 | Local (fork-ports): a vertex a loop limit stops gets a | local |
| b44/0003 | Local (fork-ports): view memory in place, behind | local |
| b44/0004 | Typed buffer views of 32-bit integer formats move their | GamingRipperDoc |
| b44/0005 | Shaders: a sampler chosen by control flow between two | JRamos1993 |
| b44/0006 | Textures picked by index: the recheck's fingerprint | NickJBc |
| b44/0007 | Shaders: signature rows keep the console's order and | Mohammed Wasif Ahmed |
| b44/0008 | shader,gpu: resolve heap-loaded view addresses and | powplusplus |
| b44/0009 | GPU: a refused graphics or mesh pipeline logs its state | Joshua Odom |
| b44/0010 | Shaders: fix v_div_fixup_f32 turning 0.0/x and x/Inf | AscilCH |
| b44/0011 | GPU: the rectangle fallback's f16 rounds to even and | Jordan N |
| b44/0012 | Shaders: a sampler used with and without comparison | Jordan N |
| b44/0013 | Shaders: fix MIMG 3D Z texel offsets, a16 offset | AscilCH |
| b44/0014 | GPU: hash textures that changed lately at every check | armin |
| b44/0015 | Local (fork-ports): the shader tests take the sampler | local |
| b44/0016 | Shaders: recompute ISG1 input usage and barycentrics | Davi713 |
| b44/0017 | GPU: recheck the texture budget before each indexed | DoubleD17 |
| b44/0018 | gpu/texture: key THROUGH alias cache by (placed, | AscilCH |
| b44/0019 | Local (fork-ports2): local additions to the emitter | local |
| b44/0020 | A loop limit for compute programs alone (hair strands | Danigamer05 |
| b44/0021 | Local (fork-ports2): a texture's size read from its | local |
| b44/0022 | Formats: a texture of which only the smaller mips are | myokex |
| b44/0023 | Revert "shader,gpu: resolve heap-loaded view addresses | local |
| b44/0024 | Shaders: view-address table for descriptors picked by a | uiliannn (original), Revens2 (fork PR #749) |
| b44/0025 | Shaders: view records step 16 bytes, no table offset on | uiliannn (original), Revens2 (fork PR #749) |
| b44/0026 | Local (fork-ports2): fork PR #749's view table instead | local |
| b44/0027 | Local (fork-ports2): a ray-tracing structure's view | local |
| b44/0028 | Local: VICTORY_WINDOW=<w>x<h> sizes the title's | local |
| b44/0029 | Local: captures save typeless 8-bit frames (the shown | local |
| b44/0030 | Local: VICTORY_CAPTURE_EVERY/_MIN_DRAWS on the fork | local |
| b44/0031 | Series S: the title keeps its own upscaler, not FSR 4 | Kyle Soderlund |
| b44/0032 | Local: the known build's title patches apply on Series | local |
| b44/0033 | GPU: no blending on integer render targets (hand port | local |
| b44/0034 | GPU: VICTORY_BACK_QUIET=1 (hand port of fork #1188; NOT | local |
| b44/0035 | GPU: a constant can stand in for indirect light while | UwUCammie |
| b44/0036 | GPU: textures picked by index served in turns (fork | Exopus |
| b44/0037 | Present: the second plane (the Series S interface) is | local |
| b44/0038 | Card copies: a census, and an experiment that leaves | Chipsrain |
| b44/0039 | Count usable in-flight readback spans in the per-copy | iGoldenNinja |
| b44/0040 | Check declared guest ranges before skipping | iGoldenNinja |
| b44/0041 | Census: mark VICTORY_SKIP_WC_COMPARE unsafe | Chipsrain |
| b44/0042 | GPU: VICTORY_BACK_OWN_VIEW=1: copy-backs into title | local |
| b44/0043 | Local: VICTORY_KEEP_GONE: a texture picked by index | local |
| b45-prep/0001 | Shaders: the tiled cascade shadow programs translate | Bruno Magalhães (fork PR #1002) |
| b45-prep/0002 | Textures picked by index uploaded on a thread of their | local |
| capture/0001 | Local: VICTORY_WINDOW=<w>x<h> sizes the title's top-level | local |
| capture/0002 | Local: captures save typeless 8-bit frames (the shown | local |
| capture/0003 | Local: VICTORY_CAPTURE_EVERY/_MIN_DRAWS on the fork | local |

Hand ports and where they come from:

| Patch | Source | Author of the source |
|---|---|---|
| b44/0024-0025 view-address table | fork PR #749 by Revens2 (Revens2/victory-gta6), adapted from uiliannn/VItoriaBr (GPL-3.0, commits b62bbc0 and 77ad907); "integ" in the commits is only a signature | uiliannn, Revens2 |
| b44/0033 walls fix | upstream VIctory PR #2346 | Calvix2k |
| b44/0034 BACK_QUIET | fork PR #1188 | Yorivel |
| b44/0035 AMBIENT | fork PR #917; the pass is from VIctory-Belobug-AMD | UwUCammie, Belobug |
| b44/0036 texture queue in turns | fork PR #1246 | Exopus |
| b44/0037 Series S interface plane | fork PR #1081 | Kyle Soderlund |
| b44/0038-0041 census, SKIP_WC | fork PR #868 | Chipsrain, iGoldenNinja |
| b44/0042 BACK_OWN_VIEW | the fork's own-view copy-back | fxceless |
| b44/0031 Series S upscaler | fork PR #798 | Kyle Soderlund |
| b45-prep/0001 cascade shadows | fork PR #1002 | Bruno Magalhães |
| b45-prep/0002 texture upload thread | fork PR #1007 | TG0000 |

Note on b44/0034 (BACK_QUIET): its subject still says "NOT YET COMPILED", which was true when it was written. It compiles and passes the tests since b44.

Several commits were written with Claude (Anthropic); their messages say so.

## Left out on purpose

- Nothing derived from xcl (GPL-2.0-only, not compatible): no such file or hunk is in the series.
- No game data: no shader containers, captures or dumps. Every file the patches touch is a `.rs` source, except one `.md`.
- No local paths, user names or e-mail addresses.
