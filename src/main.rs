//! TITAN CORE: finds FPKG .pkg files and game folders on the internal SSD and USB/extended
//! drives, keeps a per-game cache in /data/titan-core/caches/<TID>, registers each game on the
//! home screen and overlays its fakelib at launch (DESIGN.md). Web UI on port web::PORT.

// ponytail: ampr_index keeps two unused format constants (FILE_STREAMING, CHUNK_STREAMING).
#![allow(dead_code)]

#[macro_use]
mod sys;
mod ampr_index;
mod cache;
mod config;
mod launch;
mod register;
mod scan;
mod storage;
mod telemetry;
mod telemetry_math;
mod web;
mod library;
mod migration;
mod installed;
mod installed_access;
mod relocation;
mod backup;
mod installer;
mod installer_io;
mod background;
#[cfg(feature="unified-stack")] mod stack;
#[cfg(feature="unified-stack")] mod stack_policy;

use cache::Fresh;
use config::Settings;
use scan::{Kind, Source};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use web::State;

pub const DATA_DIR: &str = "/data/titan-core";

/// A system toast when `notify=1`; always logged.
fn toast(s: &Settings, msg: &str) {
    if s.notify {
        sys::notify(msg);
    } else {
        log!("{msg}");
    }
}

/// What a failed game's retry waits for: another stamp, or any build input (pkg/folder files,
/// backport, ampr dir) appearing, vanishing or changing mtime.
fn inputs_key(src: &Source, s: &Settings) -> String {
    let mut k = cache::stamp(src, s).to_string();
    for p in cache::inputs(src, s) {
        let mtime = std::fs::metadata(&p).and_then(|m| m.modified()).ok();
        k += &format!("|{}={mtime:?}", p.display());
    }
    k
}

/// Low /data space: the UI banner, and a toast once per run. Checked before and after builds.
fn check_space(state: &State, s: &Settings) {
    static WARNED: AtomicBool = AtomicBool::new(false);
    let free = storage::drives().into_iter().find(|d| d.mount == "/data").map_or(u64::MAX, |d| d.free);
    let low = free < s.low_space_gb << 30;
    state.low_space.store(low, Ordering::SeqCst);
    if low && !WARNED.swap(true, Ordering::SeqCst) {
        toast(s, &format!("TITAN CORE: /data has only {} GB free", free >> 30));
    }
}

fn backport_of(src: &Source) -> Option<&std::path::Path> {
    match src.kind {
        Kind::Folder(_) => src.backport.as_deref(),
        Kind::Pkg => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Registration { Existing, New, Staged }

/// Builds when stale or forced, then registers (or stages a FW12+ batch title).
fn build_and_register(state: &State, s: &Settings, src: &Source, fresh: &Fresh, forced: bool) -> std::io::Result<Registration> {
    let tid = &src.title_id;
    if library::installed_package_path(&src.path){backup::installed(tid, storage::drives().into_iter().find(|d| d.mount == "/data").map_or(0, |d| d.free))?;}
    if let (false, Fresh::Fakelib(why)) = (forced, fresh) {
        // Only fakelib/ is behind: recompose it, then go on as a fresh cache (mount or register).
        log!("{tid}: {why}; refreshing fakelib only");
        cache::refresh_fakelib(src, s)?;
    }
    let stale = forced || !matches!(fresh, Fresh::Fresh | Fresh::Fakelib(_));
    let app0 = cache::app0(src);
    if !stale && register::is_registered(tid) {
        // ponytail: registered by an earlier run; only the nullfs is redone (gone after a
        // reboot; a no-op when mounted): no sce_sys copy, no install call.
        register::mount(tid, &app0, backport_of(src))?;
        return Ok(Registration::Existing);
    }
    if stale {
        match fresh {
            Fresh::Stale(why) => log!("{tid}: cache stale ({why}); rebuilding"),
            Fresh::Missing => log!("{tid}: no cache; building"),
            Fresh::Fresh | Fresh::Fakelib(_) => log!("{tid}: rebuild forced"),
        }
        state.update(&src.path, |g| g.status = "building");
        check_space(state, s);
        let started = Instant::now();
        let r = cache::build(src, s);
        if s.debug { log!("perf: cache build {tid} took {} ms", started.elapsed().as_millis()); }
        check_space(state, s);
        r?;
    }
    let immediate = register::register_staged(tid, &app0, &cache::sce_sys(src), backport_of(src))?;
    Ok(if immediate { Registration::New } else { Registration::Staged })
}

/// System-path PKGs use the same resident-service ownership gate as classic installs.
#[cfg(feature="unified-stack")]
fn coordinate_source(state:&State,src:&Source)->Result<(),String>{
    if library::installed_package_path(&src.path){
        installed_access::prepare(&src.path,&src.title_id,src.size).map_err(|e|e.to_string())?;
        if let Err(error)=state.stack.prepare_import_overlay(&src.title_id){let _=state.stack.rescan();return Err(error);}
    }
    state.stack.coordinate_native(&src.title_id)
}

/// One game of the scan: check, build, register, status. A failed game is retried only when
/// its inputs_key changes or a rebuild is forced.
fn process(state: &State, s: &Settings, src: &Source, registered: &mut HashSet<String>, failed: &mut HashMap<PathBuf, String>, staged: &mut Vec<Source>) {
    let tid = &src.title_id;
    #[cfg(feature="unified-stack")]
    if !state.stack.native_owns(tid){
        state.titles.lock().unwrap().remove(tid);
        state.update(&src.path,|g|{g.status="shadow_owned";g.error=Some("Managed by ShadowMount, or waiting for a fresh service snapshot".into());});return;
    }
    let forced = state.forced.lock().unwrap().remove(tid);
    let fresh = cache::check(src, s);
    // The failure fingerprint walks all cache inputs. Do not compute it on
    // healthy titles (the common path), only when a prior failure exists.
    let same_failed_inputs = !forced && failed.get(&src.path)
        .is_some_and(|previous| previous == &inputs_key(src, s));
    let r = if same_failed_inputs {
        state.update(&src.path, |g| g.status = "error");
        return;
    } else if !forced && matches!(fresh, Fresh::Fresh) && registered.contains(tid) {
        // Each scan: redo /system_ex/app/<TID> when it is gone or stale (USB replug, rest
        // mode); a no-op when app0 is mounted there. Status stays "ready" (no UI flicker).
        match register::mount(tid, &cache::app0(src), backport_of(src)) {
            Ok(()) => {
                #[cfg(feature="unified-stack")]
                if let Err(error)=coordinate_source(state,src){state.titles.lock().unwrap().remove(tid);state.update(&src.path,|g|{g.status="error";g.error=Some(error);});return;}
                state.titles.lock().unwrap().insert(tid.clone());
                state.update(&src.path, |g| { g.status = "ready"; g.error = None; });
                return;
            }
            Err(e) => Err(e),
        }
    } else {
        state.update(&src.path, |g| g.status = "checking");
        build_and_register(state, s, src, &fresh, forced)
    };
    let (size, built) = (cache::size(tid), cache::built_at(tid));
    match r {
        Ok(Registration::Staged) => {
            staged.push(src.clone());
            state.update(&src.path, |g| { g.status = "checking"; g.error = None; });
            log!("{tid}: staged; awaiting one batch registration after the scan");
        }
        Ok(registration) => {
            #[cfg(feature="unified-stack")]
            if let Err(error)=coordinate_source(state,src){
                state.titles.lock().unwrap().remove(tid);
                log!("{tid}: native overlay coordination: {error}");
                state.update(&src.path,|g|{g.status="error";g.error=Some(format!("Overlay coordination: {error}"));});return;
            }
            failed.remove(&src.path);
            registered.insert(tid.clone());
            state.titles.lock().unwrap().insert(tid.clone());
            state.update(&src.path, |g| {
                (g.status, g.error, g.cache_size, g.cache_built) = ("ready", None, size, built);
            });
            if registration == Registration::New {
                toast(s, &format!("{} registered", src.title_name));
            }
        }
        Err(e) => {
            let e = format!("{tid} ({}): {e}", src.path.display());
            log!("{e}");
            failed.insert(src.path.clone(), inputs_key(src, s));
            state.update(&src.path, |g| {
                (g.status, g.error, g.cache_size, g.cache_built) = ("error", Some(e.clone()), size, built);
            });
            toast(s, &format!("{}: failed, see the log", src.title_name));
        }
    }
}

/// Replaces the UI's game list with the scan: an entry keeps its status while its source
/// stays listed, new ones are "pending", duplicates "duplicate".
fn publish(state: &State, r: &scan::ScanResult) {
    let mut games = state.games.lock().unwrap();
    let old = std::mem::take(&mut *games);
    // Preserve the first non-duplicate status for each (source path, title)
    // without rescanning the previous dashboard on every library entry.
    let mut old_by_source: HashMap<(&std::path::Path, &str), &web::Game> = HashMap::new();
    for g in &old {
        if g.status != "duplicate" {
            old_by_source.entry((g.src.path.as_path(), g.src.title_id.as_str())).or_insert(g);
        }
    }
    let mut visible: HashSet<String> = HashSet::with_capacity(r.games.len());
    games.reserve(r.games.len() + r.duplicates.len() + r.unstable.len());
    for src in &r.games {
        visible.insert(src.title_id.clone());
        let prev = old_by_source.get(&(src.path.as_path(), src.title_id.as_str()));
        games.push(match prev {
            Some(g) => web::Game { src: src.clone(), ..(*g).clone() },
            None => web::Game { src: src.clone(), status: "pending", error: None, cache_size: 0, cache_built: None },
        });
    }
    for src in &r.duplicates {
        games.push(web::Game { src: src.clone(), status: "duplicate", error: None, cache_size: 0, cache_built: None });
    }
    // Keep an existing title visible while its source copy is in progress.
    for (path, tid) in &r.unstable {
        if let Some(previous) = old.iter().find(|g| &g.src.path == path
            && tid.as_ref().is_none_or(|t| g.src.title_id == *t)
            && g.status != "duplicate") {
            if visible.insert(previous.src.title_id.clone()) {
                games.push(previous.clone());
            }
        }
    }
}
/// Debug: copies the kernel log (/dev/klog: module load failures, process crashes) to
/// /data/titan-core/klog.txt, read through the web UI's /file route.
/// ponytail: restarts the file at 4 MiB; only one reader of /dev/klog can exist.
fn klog_capture() {
    use std::io::{Read, Write};
    const OUT: &str = "/data/titan-core/klog.txt";
    let mut klog = match std::fs::File::open("/dev/klog") {
        Ok(f) => f,
        Err(e) => {
            log!("klog: cannot open /dev/klog: {e}");
            return;
        }
    };
    log!("klog: capturing to {OUT}");
    let mut buf = vec![0u8; 64 * 1024];
    let mut out = std::fs::File::create(OUT).ok();
    let mut written = 0usize;
    loop {
        let n = match klog.read(&mut buf) {
            Ok(0) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            Ok(n) => n,
            Err(e) => {
                log!("klog: read failed: {e}");
                return;
            }
        };
        if written > 4 << 20 {
            out = std::fs::File::create(OUT).ok();
            written = 0;
        }
        if let Some(f) = out.as_mut() {
            let _ = f.write_all(&buf[..n]);
        }
        written += n;
    }
}

/// The ELF the tile's page asks elfldr to start (ui.html `ELF`), as Dump Forge saves itself.
pub const SELF_COPY: &str = "/data/titan-core/titan-core.elf";
/// Stage 1 of ps5/build.sh, embedded only in the pushed (stage 2) ELF; the saved copy has none.
#[cfg(feature = "self-copy")]
const SELF_ELF: &[u8] = include_bytes!(env!("TITAN_SELF_ELF"));
#[cfg(not(feature = "self-copy"))]
const SELF_ELF: &[u8] = &[];

/// Saves SELF_ELF to SELF_COPY unless it is there already: temp file, fsync, rename.
/// ponytail: the last pushed ELF wins, no version order (Dump Forge keeps the newer one).
fn save_self_copy() {
    if SELF_ELF.is_empty() || std::fs::read(SELF_COPY).is_ok_and(|b| b == SELF_ELF) {
        return;
    }
    let tmp = format!("{SELF_COPY}.tmp");
    let r = std::fs::File::create(&tmp)
        .and_then(|mut f| std::io::Write::write_all(&mut f, SELF_ELF).and_then(|()| f.sync_all()))
        .and_then(|()| std::fs::rename(&tmp, SELF_COPY));
    match r {
        Ok(()) => log!("saved {SELF_COPY} ({} bytes): the tile starts it", SELF_ELF.len()),
        Err(e) => log!("save {SELF_COPY}: {e}"),
    }
}

/// The ampr-emu-ng release bundled in the ELF (vendor/ampr-emu-ng; ps5/build.sh checks its SHA-256).
const BUNDLED_AMPR: &[u8] = include_bytes!("../vendor/ampr-emu-ng/libSceAmpr.sprx");
const BUNDLED_AMPR_VERSION: &str = include_str!("../vendor/ampr-emu-ng/VERSION");
/// "<len> <crc32>" of the bundled copy last written. Outside AMPR_DIR: every file there goes into fakelib/.
const BUNDLED_MARK: &str = "/data/titan-core/ampr.bundled";

/// Puts the bundled libSceAmpr.sprx into AMPR_DIR when it is missing or is a bundled copy we wrote
/// before (an older release). A file put there by hand (a debug build, say) is kept.
fn install_bundled_ampr() {
    let path = format!("{}/libSceAmpr.sprx", cache::AMPR_DIR);
    let mark = |b: &[u8]| format!("{} {:08x}", b.len(), ampr_index::crc32(&[b]));
    let write_mark = || {
        if let Err(e) = std::fs::write(BUNDLED_MARK, mark(BUNDLED_AMPR)) {
            log!("ampr: write {BUNDLED_MARK}: {e}");
        }
    };
    match std::fs::read(&path) {
        Ok(cur) if cur == BUNDLED_AMPR => {
            if std::fs::read_to_string(BUNDLED_MARK).ok() != Some(mark(BUNDLED_AMPR)) {
                write_mark();
            }
            return;
        }
        Ok(cur) if std::fs::read_to_string(BUNDLED_MARK).is_ok_and(|m| m == mark(&cur)) => {}
        Ok(_) => {
            log!("ampr: {path} is not a bundled ampr-emu-ng: kept (delete it to get {BUNDLED_AMPR_VERSION})");
            return;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            log!("ampr: read {path}: {e}");
            return;
        }
    }
    // Temp file beside AMPR_DIR, not in it, so a scan never copies a half-written one.
    let tmp = format!("{DATA_DIR}/libSceAmpr.sprx.tmp");
    let r = std::fs::File::create(&tmp)
        .and_then(|mut f| std::io::Write::write_all(&mut f, BUNDLED_AMPR).and_then(|()| f.sync_all()))
        .and_then(|()| std::fs::rename(&tmp, &path));
    match r {
        Ok(()) => {
            write_mark();
            log!("ampr: installed bundled ampr-emu-ng {BUNDLED_AMPR_VERSION} ({} bytes) to {path}", BUNDLED_AMPR.len());
        }
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            log!("ampr: install bundled ampr-emu-ng to {path}: {e}");
        }
    }
}

/// ampr files are loaded by every game's sandbox: 0777 like the caches. A libkernel.sprx there
/// replaces the system one for every game, which is rarely wanted.
fn prepare_ampr() {
    let Ok(rd) = std::fs::read_dir(cache::AMPR_DIR) else { return };
    for e in rd.flatten() {
        let path = e.path();
        if !path.is_file() {
            continue;
        }
        if let Err(e) = std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o777)) {
            log!("ampr: chmod 0777 {}: {e}", path.display());
        }
        if e.file_name().eq_ignore_ascii_case("libkernel.sprx") {
            log!("ampr: WARNING {} overrides the system libkernel in every game; remove it unless intended", path.display());
        }
    }
}

fn main() {
    // panic=abort: record the panic in titan-core.log before the payload dies.
    std::panic::set_hook(Box::new(|info| sys::log_line(&format!("PANIC: {info}"))));
    // Take ownership before renaming cache directories used by an older process.
    let Some(listener)=web::take_port() else{sys::notify("TITAN CORE: close the running game before upgrading");return;};
    let migration=match migration::migrate(std::path::Path::new("/data/krakenmount"),std::path::Path::new(DATA_DIR)){Ok(report)=>report,Err(e)=>{sys::notify(&format!("TITAN CORE: data migration stopped: {e}"));return;}};
    for dir in [DATA_DIR,cache::CACHES,cache::AMPR_DIR]{if let Err(e)=std::fs::create_dir_all(dir){sys::notify(&format!("TITAN CORE: cannot create {dir}: {e}"));return;}}
    let s=config::load();sys::LOG_FILE.store(s.log,Ordering::Relaxed);
    if let Some(report)=migration{log!("{report}");}
    // The bundled runtime must be available even while the optional image API is busy.
    install_bundled_ampr();prepare_ampr();
    toast(&s, "TITAN CORE started");
    let state=Arc::new(State::new(s.clone()));
    #[cfg(feature="unified-stack")]
    stack::Stack::start(state.clone());
    let web_state=state.clone();
    std::thread::spawn(move||web::serve(web_state,listener));
    std::thread::spawn(save_self_copy);
    let mut watcher_started=false;
    #[cfg(feature="unified-stack")]
    let mut restore_errors: HashMap<String,String> = HashMap::new();
    let (mut registered, mut failed) = (HashSet::new(), HashMap::new());
    loop {
        // A game runs: nothing that could touch its app0 or /system_ex mount. Rescan, rebuild
        // and purge requests stay queued (forced / purge) for after it exits.
        // ponytail: polls every 2 s instead of a wake from the watcher.
        if state.paused() {
            state.busy.store(false, Ordering::SeqCst);
            std::thread::sleep(Duration::from_secs(2));
            continue;
        }
        // Held to the end of the pass (dropped at `continue` and before the wait): /api/delete waits on it.
        let work = state.work.lock().unwrap();
        state.installer.run(&state);
        if state.paused() { continue; }
        let s = config::load();
        sys::LOG_FILE.store(s.log, Ordering::Relaxed);
        *state.settings.lock().unwrap() = s.clone();
        state.busy.store(true, Ordering::SeqCst);
        #[cfg(feature="unified-stack")]
        if s.auto_import_installed && state.stack.native_ready() && !state.paused(){
            let mut restored=false;
            for (tid,result) in relocation::restore_legacy(std::path::Path::new(relocation::ROOT),std::path::Path::new("/user/app"),||state.paused(),|path,expected|{
                let (actual,_,_)=cache::peek_pkg(path)?;
                if actual!=expected{return Err(std::io::Error::other("Legacy package title does not match its receipt; source kept"));}
                if state.paused(){return Err(std::io::Error::other("A game started; restoration deferred"));}
                let metadata=cache::dir(expected).join("sce_sys");
                let recreated=relocation::restore_title_directory(std::path::Path::new("/user/app"),expected,&metadata)?;
                if recreated||!register::is_registered(expected){
                    if state.paused(){return Err(std::io::Error::other("A game started; title registration deferred"));}
                    register::register(expected,&cache::dir(expected),&metadata,None)?;
                    log!("{expected}: original title metadata registered from validated cache");
                }
                Ok(())
            }){
                match result{
                    Ok(path)=>{restore_errors.remove(&tid);restored=true;log!("{tid}: restored original system package path {} without a package copy",path.display());},
                    Err(error)=>{let message=error.to_string();if restore_errors.get(&tid)!=Some(&message){log!("{tid}: original-path restoration deferred: {message}");restore_errors.insert(tid,message);}}
                }
            }
            if restored{if let Err(error)=state.stack.rescan(){log!("Refresh installed inventory after restoration: {error}");}}
        }
        let mut roots = if s.scan_paths.is_empty() { storage::default_scan_roots() } else { s.scan_paths.clone() };
        roots.extend(relocation::scan_roots(std::path::Path::new(relocation::ROOT)));
        let mut install_roots=installer::roots();install_roots.extend(roots.iter().cloned());
        let scan_started = Instant::now();
        let mut r = scan::scan(&install_roots, s.backports, s.stability_wait_secs);
        if s.debug {
            log!("perf: native discovery {} games across {} roots in {} ms",
                  r.games.len(), install_roots.len(), scan_started.elapsed().as_millis());
        }
        #[cfg(feature="unified-stack")]
        if s.auto_import_installed && state.stack.native_ready(){
            let native=r.games.iter().map(|g|g.title_id.clone()).collect();
            for g in installed::candidates(&state.stack.games(),&native){
                if state.paused() || !state.settings.lock().unwrap().auto_import_installed{break;}
                let path=PathBuf::from(g["path"].as_str().unwrap());let tid=g["title_id"].as_str().unwrap();
                let prepared=(||->Result<scan::Source,String>{
                    let mut src=scan::installed_source(&path,tid).map_err(|e|e.to_string())?;
                    backup::installed(tid, storage::drives().into_iter().find(|d| d.mount == "/data").map_or(0, |d| d.free)).map_err(|e|e.to_string())?;
                    if state.paused(){return Err("A game started; installed package preparation deferred".into());}
                    state.stack.prepare_import_overlay(tid)?;
                    if state.paused(){return Err("A game started; in-place access preparation deferred".into());}
                    src.path=match installed_access::prepare(&path,tid,src.size){
                        Ok(path)=>path,
                        Err(error)=>{if let Err(restore)=state.stack.restore_import_overlay(tid){log!("{tid}: original package kept; overlay policy restore needs attention: {restore}");}return Err(error.to_string());}
                    };
                    log!("{tid}: native package kept in place at {}; preparing native cache",src.path.display());Ok(src)
                })();
                match prepared{Ok(src)=>r.games.push(src),Err(error)=>{if state.installed_errors.lock().unwrap().get(path.to_string_lossy().as_ref())!=Some(&error){log!("Installed FPKG {tid}: {error}");}r.errors.push((path,error));}}
            }
        }
        *state.installed_errors.lock().unwrap()=r.errors.iter().filter(|(p,_)|library::installed_package_path(p)).map(|(p,e)|(p.to_string_lossy().into_owned(),e.clone())).collect();
        if state.paused() {
            continue;
        }
        // Discovery is independent of service health: an unavailable image provider cannot erase FPKGs.
        *state.image_files.lock().unwrap()=library::discover_exfat(&roots);
        publish(&state,&r);
        #[cfg(feature="unified-stack")]
        if !state.stack.native_ready(){
            *state.cache_total.lock().unwrap()=cache::total_size();
            state.busy.store(false,Ordering::SeqCst);drop(work);state.wait(Duration::from_secs(2));continue;
        }
        if !watcher_started{
            let (titles,running)=(state.titles.clone(),state.running.clone());
            std::thread::spawn(move||launch::watch(titles,running));watcher_started=true;
            if s.debug{std::thread::spawn(klog_capture);}
        }
        #[cfg(feature="unified-stack")]
        if !state.paused() && state.stack.services()["shadowmount_ready"]==true {background::hide_shadow_launcher();}
        // Purge after the scan, so `keep` is this scan's library and every kept title rebuilds below.
        if state.purge.swap(false, Ordering::SeqCst) {
            let keep: Vec<String> = r.games.iter().map(|g| g.title_id.clone()).collect();
            match cache::purge(&keep) {
                Ok(()) => log!("caches purged; {} kept for rebuild", keep.len()),
                Err(e) => log!("purge {}: {e}", cache::CACHES),
            }
            state.forced.lock().unwrap().extend(keep);
        }
        publish(&state, &r);
        // A source that left the scan (USB unplugged) goes through register::mount again when back.
        let available: HashSet<&str> = r.games.iter().map(|g| g.title_id.as_str()).collect();
        registered.retain(|t: &String| available.contains(t.as_str()));
        // Built caches are overlaid at launch from the start, not only once the scan reaches them.
        check_space(&state, &s);
        let mut staged = Vec::new();
        for src in &r.games {
            if state.paused() {
                log!("a game started; scan paused until it exits");
                break;
            }
            process(&state, &s, src, &mut registered, &mut failed, &mut staged);
        }
        // Batch only when no game is running. If interrupted, staged titles
        // remain on disk and are naturally retried on the next scan pass.
        if !staged.is_empty() && !state.paused() {
            let titles: Vec<String> = staged.iter().map(|src| src.title_id.clone()).collect();
            let result = register::install_all(&titles);
            let mut completed = 0usize;
            for src in &staged {
                if state.paused() { break; }
                let tid = &src.title_id;
                let finish = match &result {
                    Ok(()) => {
                        #[cfg(feature="unified-stack")]
                        {
                            coordinate_source(&state, src).map_err(std::io::Error::other)
                        }
                        #[cfg(not(feature="unified-stack"))]
                        { Ok(()) }
                    }
                    Err(e) => Err(std::io::Error::other(e.to_string())),
                };
                match finish {
                    Ok(()) => {
                        failed.remove(&src.path);
                        registered.insert(tid.clone());
                        state.titles.lock().unwrap().insert(tid.clone());
                        state.update(&src.path, |g| {
                            (g.status, g.error, g.cache_size, g.cache_built) =
                                ("ready", None, cache::size(tid), cache::built_at(tid));
                        });
                        completed += 1;
                    }
                    Err(e) => {
                        state.titles.lock().unwrap().remove(tid);
                        let message = format!("{tid}: batch registration: {e}");
                        log!("{message}");
                        failed.insert(src.path.clone(), inputs_key(src, &s));
                        state.update(&src.path, |g| {
                            (g.status, g.error) = ("error", Some(message.clone()));
                        });
                    }
                }
            }
            if result.is_ok() && completed > 0 {
                toast(&s, &format!("{completed} games registered"));
            } else if let Err(e) = result {
                toast(&s, &format!("Batch registration failed for {} games: {e}", titles.len()));
            }
        }
        *state.cache_total.lock().unwrap() = cache::total_size();
        check_space(&state, &s);
        state.busy.store(false, Ordering::SeqCst);
        drop(work);
        state.wait(Duration::from_secs(s.scan_interval_secs));
    }
}
