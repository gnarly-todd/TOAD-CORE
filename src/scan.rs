//! Game discovery over the scan roots (DESIGN.md).
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Packing {
    Plain,
    Zstd,
    Lz4,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Pkg,
    Folder(Packing),
}

#[derive(Clone, Debug)]
pub struct Source {
    pub path: PathBuf,
    pub kind: Kind,
    pub title_id: String,
    pub title_name: String,
    pub version: String,
    /// Mount of the drive holding `path` ("/data", "/mnt/usb0").
    pub drive: String,
    /// .pkg file size; 0 for folders.
    pub size: u64,
    pub backport: Option<PathBuf>,
}

pub struct ScanResult {
    pub games: Vec<Source>,
    pub duplicates: Vec<Source>,
    pub errors: Vec<(PathBuf, String)>,
    /// Sources currently being written. Keep any previously observed library entries.
    pub unstable: Vec<(PathBuf, Option<String>)>,
}

/// Seconds since the youngest max(mtime, ctime) of `paths` when that is under `wait` (SMP
/// sm_stability.c); None = stable. A clock in the future (negative age) counts as stable, as in SMP.
/// A path that cannot be stat'ed is ignored here (the scan reports it).
pub fn still_written(paths: &[PathBuf], wait: u64) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    if wait == 0 {
        return None;
    }
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs() as i64;
    paths.iter().filter_map(|p| fs::metadata(p).ok())
        .map(|m| now - m.mtime().max(m.ctime()))
        .filter(|&age| (0..wait as i64).contains(&age))
        .min()
        .map(|age| age as u64)
}

/// Records an unstable source; logs it once until it is stable again.
fn skip_unstable(result: &mut ScanResult, path: PathBuf, tid: Option<String>, age: u64) {
    let logged = UNSTABLE.get_or_init(|| Mutex::new(HashSet::new()));
    if logged.lock().unwrap_or_else(|e| e.into_inner()).insert(path.clone()) {
        let label = tid.clone().unwrap_or_else(|| path.display().to_string());
        log!("{label}: source still being written (changed {age} s ago), waiting");
    }
    result.unstable.push((path, tid));
}

/// Unstable sources already logged.
static UNSTABLE: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();

type PkgParam = Result<(String, String, String), String>;
type PkgMemo = HashMap<PathBuf, (u64, SystemTime, PkgParam)>;
static PKG_MEMO: OnceLock<Mutex<PkgMemo>> = OnceLock::new();

fn pkg_param(path: &Path, size: u64, mtime: SystemTime) -> PkgParam {
    let memo = PKG_MEMO.get_or_init(|| Mutex::new(HashMap::new()));
    let mut memo = memo.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((old_size, old_mtime, result)) = memo.get(path) {
        if *old_size == size && *old_mtime == mtime {
            return result.clone();
        }
    }
    let result = crate::cache::peek_pkg(path);
    // Only deterministic results are memoized; a transient I/O error (USB waking up) retries next scan.
    let deterministic = match &result {
        Ok(_) => true,
        Err(e) => matches!(e.kind(), ErrorKind::InvalidData | ErrorKind::UnexpectedEof),
    };
    let result = result.map_err(|e| e.to_string());
    if deterministic {
        memo.insert(path.to_path_buf(), (size, mtime, result.clone()));
    } else {
        memo.remove(path);
    }
    result
}

/// Build a sorted snapshot of backport directories once per scan, rather than
/// reopening and sorting every root's backports for each discovered title.
/// Directory symlinks retain the original is_dir behavior; no writes are made.
type BackportIndex = HashMap<PathBuf, Vec<PathBuf>>;

fn index_backports(roots: &[String]) -> BackportIndex {
    let mut index = HashMap::new();
    for root in roots.iter().map(Path::new).chain(std::iter::once(Path::new("/data/homebrew"))) {
        let dir = root.join("backports");
        if index.contains_key(&dir) {
            continue;
        }
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path())
            .filter(|p| p.is_dir()).collect();
        paths.sort();
        index.insert(dir, paths);
    }
    index
}

/// SMP priority: owning root first, then configured roots, then /data/homebrew.
/// Exact TID folder wins over alphabetic TID-prefix matches within each root.
/// The directory snapshot is per scan, so newly connected backports are picked
/// up on the next scan without repeated USB directory walks per title.
fn backport_for(owner: &Path, roots: &[String], tid: &str, index: &BackportIndex) -> Option<PathBuf> {
    let mut order: Vec<&Path> = vec![owner];
    for root in roots.iter().map(Path::new).chain([Path::new("/data/homebrew")]) {
        if !order.contains(&root) {
            order.push(root);
        }
    }
    for root in order {
        let dir = root.join("backports");
        let exact = dir.join(tid);
        if exact.is_dir() {
            return Some(exact);
        }
        if let Some(path) = index.get(&dir).and_then(|paths| paths.iter().find(|p| {
            p.file_name().is_some_and(|name| name.to_string_lossy().starts_with(tid))
        })) {
            return Some(path.clone());
        }
    }
    None
}

/// .pkg path -> TID of each pkg cache dir, from the "source" of its `.done` stamp.
fn cached_pkgs() -> HashMap<PathBuf, String> {
    let Ok(rd) = fs::read_dir(crate::cache::CACHES) else { return HashMap::new() };
    rd.flatten()
        .filter_map(|e| {
            let tid = e.file_name().to_string_lossy().into_owned();
            let stamp: serde_json::Value = serde_json::from_slice(&fs::read(e.path().join(crate::cache::DONE)).ok()?).ok()?;
            if stamp["kind"] != "pkg" {
                return None;
            }
            Some((PathBuf::from(stamp["source"].as_str()?), tid))
        })
        .collect()
}

/// (title id, name, version) from the cache's sce_sys/param.json when that cache was built from `pkg`
/// after the pkg's last change and its stamp has the pkg's current size (a same-name replacement
/// that kept an older mtime differs in size). ponytail: trusts the cache; the .pkg is not opened then.
fn cached_param(cached: &HashMap<PathBuf, String>, pkg: &Path, size: u64, mtime: SystemTime) -> Option<(String, String, String)> {
    let dir = crate::cache::dir(cached.get(pkg)?);
    let done = dir.join(crate::cache::DONE);
    if fs::metadata(&done).and_then(|m| m.modified()).ok()? <= mtime {
        return None;
    }
    let stamp: serde_json::Value = serde_json::from_slice(&fs::read(&done).ok()?).ok()?;
    if stamp["size"].as_u64() != Some(size) {
        return None;
    }
    parse_param(&fs::read(dir.join("sce_sys/param.json")).ok()?).ok()
}

fn scan_error(result: &mut ScanResult, path: PathBuf, error: impl std::fmt::Display) {
    let message = error.to_string();
    // Log only new or changed errors per path: a bad entry would otherwise log every scan.
    let logged = LOGGED.get_or_init(|| Mutex::new(HashMap::new()));
    let mut logged = logged.lock().unwrap_or_else(|e| e.into_inner());
    if logged.get(&path) != Some(&message) {
        log!("{}: {message}", path.display());
        logged.insert(path.clone(), message.clone());
    }
    result.errors.push((path, message));
}

/// Last logged scan error per path.
static LOGGED: OnceLock<Mutex<HashMap<PathBuf, String>>> = OnceLock::new();

/// Sorted direct children of a scan root, filtering AppleDouble/dot entries.
fn list_scan_entries(dir: &Path, result: &mut ScanResult) -> Vec<PathBuf> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Vec::new(),
        Err(e) => {
            scan_error(result, dir.to_path_buf(), e);
            return Vec::new();
        }
    };
    let mut paths = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) if entry.file_name().to_string_lossy().starts_with('.') => {}
            Ok(entry) => paths.push(entry.path()),
            Err(e) => scan_error(result, dir.to_path_buf(), e),
        }
    }
    paths.sort();
    paths
}

/// Exclude backport folders regardless of case (exFAT preserves original casing).
fn is_backports(path: &Path) -> bool {
    path.file_name().is_some_and(|n| n.eq_ignore_ascii_case("backports"))
}

pub fn scan(roots: &[String], backports: bool, stability_wait: u64) -> ScanResult {
    let mut result = ScanResult { games: Vec::new(), duplicates: Vec::new(), errors: Vec::new(), unstable: Vec::new() };
    let mut seen = HashSet::new();
    let cached = cached_pkgs();
    let indexed_backports = if backports { index_backports(roots) } else { BackportIndex::new() };
    let installed_roots = crate::installer::roots();
    // Kraken Mount 0.3.1: direct root entries always precede one-level nested
    // entries, so a game in a subfolder cannot shadow a direct-root copy.
    // Do not descend into drive roots, backports, a game, or another scan root.
    // No general recursion: searching a USB drive's Movies, etc. is unsafe.
    let mut direct = Vec::new();
    let mut nested = Vec::new();
    let mut listed: HashSet<PathBuf> = HashSet::new();
    for root in roots {
        let root = Path::new(root);
        let drive_root = root.parent() == Some(Path::new("/mnt"));
        for path in list_scan_entries(root, &mut result) {
            if is_backports(&path) || !listed.insert(path.clone()) {
                continue;
            }
            let descend = !drive_root && path.is_dir()
                && !(path.join("eboot.bin").is_file() && path.join("sce_sys/param.json").is_file())
                && !roots.iter().any(|r| Path::new(r) == path);
            if descend {
                for child in list_scan_entries(&path, &mut result) {
                    if !is_backports(&child) && listed.insert(child.clone()) {
                        nested.push((root.to_path_buf(), child));
                    }
                }
            }
            direct.push((root.to_path_buf(), path));
        }
    }
    direct.extend(nested);
    for (root, path) in direct {
            let metadata = match fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(e) => {
                    scan_error(&mut result, path, e);
                    continue;
                }
            };
            let (kind, size, param) = if metadata.is_file()
                && path.extension().is_some_and(|e| e.eq_ignore_ascii_case("pkg")) {
                // External download archives belong to Installer, never automatic mount discovery.
                // Only a receipt-proven TITAN installation or a genuine system PKG path is playable.
                if crate::storage::drive_of(&path) != "/data"
                    && !crate::library::installed_package_path(&path)
                    && !installed_roots.iter().any(|dir| {
                        let dir = Path::new(dir);
                        dir.file_name().is_some_and(|tid| path == dir.join(format!("{}.pkg", tid.to_string_lossy())))
                            && path.canonicalize().ok().as_ref() == Some(&path)
                    }) {
                    continue;
                }
                // Skip partially copied packages before reading the header, preserving last good state.
                if let Some(age) = still_written(std::slice::from_ref(&path), stability_wait) {
                    let tid = cached.get(&path).cloned();
                    skip_unstable(&mut result, path, tid, age);
                    continue;
                }
                let mtime = match metadata.modified() {
                    Ok(mtime) => mtime,
                    Err(e) => {
                        scan_error(&mut result, path, e);
                        continue;
                    }
                };
                let param = match cached_param(&cached, &path, metadata.len(), mtime) {
                    Some(p) => Ok(p),
                    None => pkg_param(&path, metadata.len(), mtime),
                };
                (Kind::Pkg, metadata.len(), param)
            } else if metadata.is_dir() && !is_backports(&path)
                && path.join("eboot.bin").is_file() && path.join("sce_sys/param.json").is_file() {
                let probe = [path.clone(), path.join("sce_sys"), path.join("sce_sys/param.json"), path.join("eboot.bin")];
                if let Some(age) = still_written(&probe, stability_wait) {
                    let tid = cached.get(&path).cloned();
                    skip_unstable(&mut result, path, tid, age);
                    continue;
                }
                let packing = if path.join("ampr_zstd.index").is_file() {
                    Packing::Zstd
                } else if path.join("ampr_assets.index").is_file() {
                    Packing::Lz4
                } else {
                    Packing::Plain
                };
                (Kind::Folder(packing), 0, fs::read(path.join("sce_sys/param.json"))
                    .and_then(|json| parse_param(&json)).map_err(|e| e.to_string()))
            } else {
                continue;
            };
            let (title_id, title_name, version) = match param {
                Ok(param) => param,
                Err(e) => {
                    scan_error(&mut result, path, e);
                    continue;
                }
            };
            if !valid_tid(&title_id) {
                scan_error(&mut result, path, format!("invalid title id {title_id:?}"));
                continue;
            }
            let source = Source {
                drive: crate::storage::drive_of(&path),
                backport: if backports { backport_for(&root, roots, &title_id, &indexed_backports) } else { None },
                path, kind, title_id, title_name, version, size,
            };
            if seen.insert(source.title_id.clone()) {
                result.games.push(source);
            } else {
                result.duplicates.push(source);
            }
        }
    result.games.sort_by(|a, b| a.title_name.cmp(&b.title_name));
    // A fixed error that comes back is logged again.
    if let Some(logged) = LOGGED.get() {
        let errors: HashSet<&PathBuf> = result.errors.iter().map(|(p, _)| p).collect();
        logged.lock().unwrap_or_else(|e| e.into_inner()).retain(|p, _| errors.contains(p));
    }
    // A stable source can be reported as newly unstable if copying restarts.
    if let Some(logged) = UNSTABLE.get() {
        let unstable: HashSet<&PathBuf> = result.unstable.iter().map(|(p, _)| p).collect();
        logged.lock().unwrap_or_else(|e| e.into_inner()).retain(|p| unstable.contains(p));
    }
    result
}
pub fn parse_param(json: &[u8]) -> io::Result<(String, String, String)> {
    let v: serde_json::Value = serde_json::from_slice(json)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("param.json: {e}")))?;
    let tid = v["titleId"].as_str().or_else(|| v["title_id"].as_str()).unwrap_or_default();
    if !valid_tid(tid) {
        return Err(Error::new(ErrorKind::InvalidData, format!("param.json: bad titleId {tid:?}")));
    }
    let lp = &v["localizedParameters"];
    let name = lp["defaultLanguage"].as_str()
        .and_then(|lang| lp[lang]["titleName"].as_str())
        .filter(|name| !name.is_empty())
        .or_else(|| lp["en-US"]["titleName"].as_str().filter(|name| !name.is_empty()))
        .or_else(|| lp.as_object().and_then(|langs| langs.values()
            .filter_map(|lang| lang["titleName"].as_str()).find(|name| !name.is_empty())))
        .unwrap_or(tid);
    Ok((tid.to_owned(), name.to_owned(), v["contentVersion"].as_str().unwrap_or_default().to_owned()))
}
/// Game title ids only: NPXS ids are system apps (SMP is_supported_game_title_id).
pub fn valid_tid(t: &str) -> bool {
    let b = t.as_bytes();
    b.len() == 9 && b[..4].iter().all(u8::is_ascii_uppercase) && b[4..].iter().all(u8::is_ascii_digit)
        && !t.starts_with("NPXS")
}

/// Same memoized parser as homebrew FPKGs, used only for validated installed source paths.
pub fn installed_source(path:&Path,expected:&str)->io::Result<Source>{
    if !crate::library::installed_package_path(path) || path.parent().and_then(Path::file_name).is_none_or(|n|n!=expected){return Err(Error::new(ErrorKind::InvalidInput,"Installed package path/title mismatch"));}
    let meta=fs::metadata(path)?;let (tid,name,version)=pkg_param(path,meta.len(),meta.modified()?).map_err(|e|Error::new(ErrorKind::InvalidData,e))?;
    if tid!=expected{return Err(Error::new(ErrorKind::InvalidData,"Installed package metadata title mismatch"));}
    Ok(Source{path:path.to_owned(),kind:Kind::Pkg,title_id:tid,title_name:name,version,size:meta.len(),drive:crate::storage::drive_of(path),backport:None})
}


#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn indexed_backports_preserve_owner_and_exact_match_priority() {
        let base = std::env::temp_dir().join(format!(
            "titan-backport-index-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let a = base.join("owner");
        let b = base.join("fallback");
        let tid = "PPSA12345";
        for entry in [
            a.join("backports/PPSA12345-patch"),
            a.join("backports/PPSA12345"),
            b.join("backports/PPSA12345"),
        ] {
            fs::create_dir_all(entry).unwrap();
        }
        let roots = vec![a.to_string_lossy().into_owned(), b.to_string_lossy().into_owned()];
        let indexed = index_backports(&roots);
        assert_eq!(backport_for(&a, &roots, tid, &indexed),
                   Some(a.join("backports/PPSA12345")));
        assert_eq!(backport_for(&b, &roots, tid, &indexed),
                   Some(b.join("backports/PPSA12345")));
        fs::remove_dir_all(a.join("backports/PPSA12345")).unwrap();
        let indexed = index_backports(&roots);
        assert_eq!(backport_for(&a, &roots, tid, &indexed),
                   Some(a.join("backports/PPSA12345-patch")));
        fs::remove_dir_all(base).unwrap();
    }
}
