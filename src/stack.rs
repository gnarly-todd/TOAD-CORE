//! One entry point, separate upstream processes, per-title mount ownership.
//! Uses pinned unmodified upstream ELFs. Does not execute them in this process.
use crate::{stack_policy, web::State};
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool,Ordering};
static NATIVE_ALLOWED:AtomicBool=AtomicBool::new(false);
static NATIVE_TITLES:std::sync::OnceLock<Mutex<std::collections::HashSet<String>>>=std::sync::OnceLock::new();
fn allow_title(tid:&str){NATIVE_TITLES.get_or_init(Default::default).lock().unwrap_or_else(|e|e.into_inner()).insert(tid.into());}
pub fn native_title_permitted(tid:&str)->bool{ NATIVE_TITLES.get_or_init(Default::default).lock().unwrap_or_else(|e|e.into_inner()).contains(tid)}
pub fn native_permitted()->bool{NATIVE_ALLOWED.load(Ordering::Acquire)}
use std::time::Duration;

const KSTUFF: &[u8] = include_bytes!("../vendor/unified/kstuff1.13-fpkg-dr-test8-rest.elf");
const SHADOW: &[u8] = include_bytes!("../vendor/unified/shadowmountplus.elf");
const KSTUFF_PATH: &str = "/data/titan-core/payloads/kstuff1.13-fpkg-dr-test8-rest.elf";
const SHADOW_PATH: &str = "/data/titan-core/payloads/shadowmountplus.elf";
// system_tmp is boot-scoped. Never remove another launch's attempt marker.
#[cfg(target_os = "freebsd")]
fn boot_identity() -> Option<(u64,u64)> {
    let mut bytes = [0u8;16]; let mut length = bytes.len();
    if unsafe { libc::sysctlbyname(b"kern.boottime\0".as_ptr().cast(),bytes.as_mut_ptr().cast(),&mut length,std::ptr::null_mut(),0) } != 0 || length != bytes.len() { return None; }
    let seconds = u64::try_from(i64::from_ne_bytes(bytes[..8].try_into().ok()?)).ok()?;
    let micros = u64::try_from(i64::from_ne_bytes(bytes[8..].try_into().ok()?)).ok()?;
    Some((seconds,micros))
}
#[cfg(not(target_os = "freebsd"))]
fn boot_identity() -> Option<(u64,u64)> { None }

struct Snapshot {
    phase: String, detail: String, firmware: Option<String>, port: u16,
    kstuff: bool, shadow: bool, games: Vec<Value>, updated: bool, mode:String, boot_error:Option<String>,
}
pub struct Stack { snapshot: Mutex<Snapshot> }

#[cfg(not(test))]
unsafe extern "C" { fn kernel_get_fw_version() -> u32; }
#[cfg(test)]
unsafe fn kernel_get_fw_version() -> u32 { panic!("Hardware bootstrap is never run by host tests") }

/// Read-only KEKCALL_CHECK readiness probe; verify compatibility with the bundled payload on hardware.
#[cfg(all(target_arch = "x86_64", not(test)))]
fn kstuff_ready() -> bool {
    let mut result: u64 = 0xffff_ffff_0000_0027;
    let error: u8;
    unsafe { std::arch::asm!("syscall", "setc {error}",
        inlateout("rax") result, in("rdi") 0u64, in("rsi") 0u64, in("rdx") 0u64,
        inlateout("r10") 0u64 => _, inlateout("r8") 0u64 => _, inlateout("r9") 0u64 => _,
        lateout("rcx") _, lateout("r11") _, error = lateout(reg_byte) error); }
    error == 0 && result == 0
}

#[cfg(any(not(target_arch = "x86_64"), test))]
fn kstuff_ready() -> bool { panic!("Hardware readiness is never run by host tests") }

#[cfg(target_os = "freebsd")]
fn dependency_processes() -> Result<(bool,bool),String> {
    let mib = [libc::CTL_KERN,libc::KERN_PROC,libc::KERN_PROC_PROC,0]; let mut length = 0usize;
    if unsafe { libc::sysctl(mib.as_ptr(),4,std::ptr::null_mut(),&mut length,std::ptr::null_mut(),0) } != 0 || length == 0 || length > 8 << 20 {
        return Err("Process visibility unavailable; dependencies were not launched".into());
    }
    let mut bytes = vec![0u8;length];
    if unsafe { libc::sysctl(mib.as_ptr(),4,bytes.as_mut_ptr().cast(),&mut length,std::ptr::null_mut(),0) } != 0 || length > bytes.len() {
        return Err("Process snapshot unavailable; dependencies were not launched".into());
    }
    bytes.truncate(length); crate::stack_policy::dependency_presence(&bytes)
}
#[cfg(not(target_os = "freebsd"))]
fn dependency_processes() -> Result<(bool,bool),String> { Err("PS5 process query is not run by host tests".into()) }

pub struct HttpReply { pub code: u16, pub ctype: String, pub body: Vec<u8> }
fn http(port: u16, method: &str, path: &str, body: &[u8], seconds: u64) -> Result<HttpReply, String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2)).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(seconds))).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(3))).map_err(|e| e.to_string())?;
    let head = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n", body.len());
    stream.write_all(head.as_bytes()).and_then(|_| stream.write_all(body)).map_err(|e| e.to_string())?;
    let mut reply = Vec::new();
    stream.take((8 << 20) + 1).read_to_end(&mut reply).map_err(|e| e.to_string())?;
    if reply.len() > 8 << 20 { return Err("Engine response exceeds 8 MiB".into()); }
    let cut = reply.windows(4).position(|p| p == b"\r\n\r\n").ok_or("Invalid engine HTTP reply")?;
    if cut > 8192 { return Err("Engine header too large".into()); }
    let head = std::str::from_utf8(&reply[..cut]).map_err(|_| "Invalid HTTP header")?;
    let code = head.lines().next().and_then(|l| l.split_whitespace().nth(1)).and_then(|n| n.parse().ok()).ok_or("Missing HTTP status")?;
    let mut ctype = String::from("application/json");
    let mut content_length = None;
    for line in head.lines().skip(1) {
        if let Some((key, value)) = line.split_once(':') {
            if key.eq_ignore_ascii_case("content-length") { content_length = Some(value.trim().parse::<usize>().map_err(|_| "Invalid content length")?); }
            if key.eq_ignore_ascii_case("content-type") { ctype = value.trim().into(); }
            if key.eq_ignore_ascii_case("transfer-encoding") { return Err("Chunked engine reply is unsupported".into()); }
        }
    }
    let body = reply[cut + 4..].to_vec();
    if content_length.is_some_and(|length| length != body.len()) { return Err("Truncated engine reply".into()); }
    Ok(HttpReply { code, ctype, body })
}

fn engine_json(port: u16, path: &str, body: Value, seconds: u64) -> Result<Value, String> {
    let r = http(port, "POST", path, &body.to_string().into_bytes(), seconds)?;
    if r.code != 200 { return Err(format!("ShadowMount HTTP {}", r.code)); }
    let value: Value = serde_json::from_slice(&r.body).map_err(|e| e.to_string())?;
    if value.get("status").and_then(Value::as_i64) != Some(0) { return Err(format!("ShadowMount rejected request: {}", value.get("status").unwrap_or(&Value::Null))); }
    Ok(value)
}
fn shadow_ready(port: u16) -> bool {
    engine_json(port, "/api/v1/version", json!({}), 3).is_ok_and(|v| v["api_version"] == 1 && v["shadowmount_version"].as_str().is_some_and(|v| v == "1.7beta12"))
}
fn install(path: &str, bytes: &[u8]) -> Result<(), String> {
    if fs::read(path).is_ok_and(|b| b == bytes) { return Ok(()); }
    if Path::new(path).exists() { return Err(format!("Existing different payload left untouched: {path}")); }
    // PS5 returned EOPNOTSUPP for hard-link publication. Exclusive creation needs no link syscall.
    let mut file=OpenOptions::new().write(true).create_new(true).open(path).map_err(|e|format!("Create {path}: {e}"))?;
    file.write_all(bytes).and_then(|_|file.sync_all()).map_err(|e|format!("Write {path}: {e}"))?;
    if fs::read(path).map_err(|e|e.to_string())? != bytes{return Err(format!("Verification failed: {path}"));}
    Ok(())
}

fn dispatch(path: &str) -> Result<(), String> {
    // elfldr HTTP pipe=0 dispatches into a separate payload process.
    let r = http(9021, "GET", &format!("{path}?pipe=0"), &[], 8)?;
    if r.code != 200 { return Err(format!("elfldr HTTP {} for {path}", r.code)); }
    if String::from_utf8_lossy(&r.body).contains("Error reading") { return Err(format!("elfldr could not read {path}")); }
    Ok(())
}

impl Stack {
    pub fn new() -> Self {
        Self { snapshot: Mutex::new(Snapshot {phase:"checking".into(),detail:"Checking services".into(),firmware:None,port:10101,kstuff:false,shadow:false,games:Vec::new(),updated:false,mode:"combined".into(),boot_error:None}) }
    }
    pub fn image_mode(&self)->bool{self.snapshot.lock().unwrap_or_else(|e|e.into_inner()).mode=="images"}
    pub fn native_ready(&self)->bool{native_permitted()}
    fn phase(&self, phase: &str, detail: &str) {
        let mut s = self.snapshot.lock().unwrap_or_else(|e| e.into_inner());
        s.phase = phase.into(); s.detail = detail.into();
        log!("unified: {phase}: {detail}");
    }
    pub fn services(&self) -> Value {
        let s = self.snapshot.lock().unwrap_or_else(|e| e.into_inner());
        json!({"phase":s.phase,"detail":s.detail,"firmware":s.firmware,"api_port":s.port,
            "kstuff_ready":s.kstuff,"shadowmount_ready":s.shadow,"library_fresh":s.updated,
            "kstuff_bundle":"1.13-fpkg-dr-test8-rest","shadowmount_bundle":"1.7beta12","mount_owner":"per-title","mode":"combined","native_ready":native_permitted()})
    }
    pub fn games(&self) -> Vec<Value> { self.snapshot.lock().unwrap_or_else(|e| e.into_inner()).games.clone() }
    pub fn port(&self) -> u16 { self.snapshot.lock().unwrap_or_else(|e| e.into_inner()).port }
    pub fn native_owns(&self,tid:&str)->bool {
        let s=self.snapshot.lock().unwrap_or_else(|e|e.into_inner());
        s.updated && native_permitted() && !crate::library::shadow_owns(&s.games,tid)
    }
    /// Arm native launch handling only for titles not managed by ShadowMount.
    /// Both engines also retain their upstream guards against foreign unionfs mounts.
    pub fn coordinate_native(&self,tid:&str)->Result<(),String>{
        if !self.native_owns(tid){return Err("Title belongs to ShadowMount or service snapshot is unavailable".into());}
        allow_title(tid);Ok(())
    }
    /// Native AMPR must own common/lib for installed packages prepared in place. Preserve the
    /// previous per-title policy before changing only this title through upstream's API.
    pub fn prepare_import_overlay(&self,tid:&str)->Result<(),String>{
        if !self.native_owns(tid){return Err("Native title ownership unavailable".into());}
        let games=self.games();let g=games.iter().find(|g|g["title_id"]==tid && g["installed_pkg"]==true).ok_or("Installed title inventory unavailable")?;
        let dir=std::path::Path::new("/data/titan-core/installed-backups").join(tid);fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
        let path=dir.join("shadow-policy.json");
        match OpenOptions::new().write(true).create_new(true).open(&path){
            Ok(mut f)=>{let previous=json!({"title_id":tid,"original_mode":g["fakelib_mode_override"].as_str().unwrap_or("default"),"effective_mode":g["fakelib_mode"],"import_mode":"disabled"});f.write_all(previous.to_string().as_bytes()).and_then(|_|f.sync_all()).map_err(|e|e.to_string())?;},
            Err(e)if e.kind()==std::io::ErrorKind::AlreadyExists=>{let previous:Value=serde_json::from_slice(&fs::read(&path).map_err(|e|e.to_string())?).map_err(|e|format!("Incomplete original overlay policy backup: {e}"))?;if previous["title_id"]!=tid || !matches!(previous["original_mode"].as_str(),Some("default"|"disabled"|"full"|"emulators")){return Err("Invalid original overlay policy backup; package remains in its original location".into());}},Err(e)=>return Err(e.to_string()),
        }
        if g["fakelib_mode"]=="disabled"{return Ok(());}
        let v=engine_json(self.port(),"/api/v1/games/fakelib",json!({"title_id":tid,"mode":"disabled"}),5)?;
        if !import_policy_confirmed(&v){return Err("ShadowMount did not confirm native overlay ownership; package remains in its original location".into());}
        log!("{tid}: ShadowMount overlay disabled for native in-place preparation; original policy preserved");Ok(())
    }
    pub fn restore_import_overlay(&self,tid:&str)->Result<(),String>{
        let path=std::path::Path::new("/data/titan-core/installed-backups").join(tid).join("shadow-policy.json");
        let previous:Value=serde_json::from_slice(&fs::read(path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        let mode=previous["original_mode"].as_str().ok_or("Missing original overlay mode")?;
        if previous["title_id"]!=tid || !matches!(mode,"default"|"full"|"emulators"|"disabled"){return Err("Invalid original overlay policy".into());}
        let v=engine_json(self.port(),"/api/v1/games/fakelib",json!({"title_id":tid,"mode":mode}),5)?;
        if v["saved"]!=true || v["fakelib_mode_override"]!=mode{return Err("Original overlay restore was not confirmed".into());}Ok(())
    }
    pub fn kernel_log(&self)->Result<Value,String>{let v=engine_json(self.port(),"/api/v1/kernel-log",json!({"max_bytes":65536}),5)?;if !v["content"].is_string(){return Err("Kernel log response is unavailable".into());}Ok(v)}
    pub fn rescan(&self) -> Result<(), String> { engine_json(self.port(), "/api/v1/scan", json!({"reset_attempts":false}), 5).map(|_| ()) }
    pub fn icon(&self, tid: &str) -> Result<HttpReply, String> { http(self.port(), "GET", &format!("/api/v1/games/icon?title_id={tid}&size=cover"), &[], 5) }
    pub fn sizes(&self, tid: &str) -> Result<Value, String> {
        let g = engine_json(self.port(), "/api/v1/games/info", json!({"title_id":tid}), 15)?;
        let size = |status: &str, value: &str| { if g[status] == 0 { g[value].clone() } else { Value::Null } };
        Ok(json!({"engine":"shadowmountplus", "game":size("size_status","size_bytes"),
            "save":size("save_data_size_status","save_data_size_bytes"), "dlc":size("dlc_size_status","dlc_size_bytes"),
            "patch":size("patch_size_status","patch_size_bytes"), "download":size("download_size_status","download_size_bytes"), "cache":null}))
    }
    pub fn start(state: Arc<State>) { std::thread::spawn(move || {
        if let Err(error) = state.stack.boot() { state.stack.phase("error", &error); return; }
        state.wake();
        loop {
            let port = state.stack.port();
            let response = engine_json(port, "/api/v1/games", json!({"include_size":false}), 5);
            match response {
                Ok(value) => {
                    let Some(games) = value["games"].as_array() else { { let mut s = state.stack.snapshot.lock().unwrap_or_else(|e| e.into_inner()); s.updated = false; s.shadow = false; NATIVE_ALLOWED.store(false,Ordering::Release); } state.stack.phase("error", "Invalid ShadowMount games snapshot"); std::thread::sleep(Duration::from_secs(3)); continue; };
                    let games:Vec<Value> = games.iter().filter(|g| g["title_id"].as_str().is_some_and(|tid| crate::scan::valid_tid(tid) && tid != "KRKN00001")).map(normalize_game).collect();
                    let mut s = state.stack.snapshot.lock().unwrap_or_else(|e| e.into_inner());
                    NATIVE_TITLES.get_or_init(Default::default).lock().unwrap_or_else(|e|e.into_inner()).retain(|tid|!crate::library::shadow_owns(&games,tid));
                    s.games = games; s.updated = true; s.shadow = true; s.kstuff = kstuff_ready();
                    NATIVE_ALLOWED.store(s.kstuff,Ordering::Release);
                    s.phase = if s.kstuff { "ready" } else { "degraded" }.into();
                    s.detail = if s.kstuff { "TITAN + kstuff + ShadowMount ready; native FPKGs and disk images enabled" } else { "kstuff readiness is no longer confirmed" }.into();
                }
                Err(error) => {
                    let mut s = state.stack.snapshot.lock().unwrap_or_else(|e| e.into_inner());
                    s.updated = false; s.shadow = false; NATIVE_ALLOWED.store(false,Ordering::Release); s.phase = "degraded".into(); s.detail = format!("Engine unavailable; cached library may be stale: {error}");
                }
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    }); }
    fn boot(&self) -> Result<(), String> {
        if let Some(error)=self.snapshot.lock().unwrap_or_else(|e|e.into_inner()).boot_error.clone(){return Err(error);}
        let raw = unsafe { kernel_get_fw_version() };
        let firmware = stack_policy::firmware_label(raw).ok_or("Firmware is unknown or outside 1.00–13.60; no dependency started")?;
        let config = match fs::read_to_string("/data/shadowmount/config.ini") {
            Ok(config) => config,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(format!("Existing ShadowMount config could not be read: {e}")),
        };
        let port = stack_policy::api_port(&config)?;
        let kstuff = kstuff_ready(); let shadow = shadow_ready(port);
        { let mut s = self.snapshot.lock().unwrap_or_else(|e| e.into_inner()); s.firmware = Some(firmware); s.port = port; s.kstuff = kstuff; s.shadow = shadow; }
        let attempt = stack_policy::attempt_name(boot_identity());
        let previous = Path::new(&attempt).exists();
        match stack_policy::boot_decision(kstuff, shadow, previous) {
            stack_policy::BootDecision::Reuse => { self.phase("ready", "Existing services reused without reloading"); return Ok(()); }
            stack_policy::BootDecision::Inconsistent => return Err("ShadowMount responds but kstuff readiness is unconfirmed; no reload attempted".into()),
            stack_policy::BootDecision::ObservePrevious => self.phase("waiting", "A startup was already attempted this boot; observing without duplicate launches"),
            stack_policy::BootDecision::Start => {
                let (kstuff_process,shadow_process) = dependency_processes()?;
                fs::create_dir_all("/data/titan-core/payloads").map_err(|e| e.to_string())?;
                install(KSTUFF_PATH, KSTUFF)?; install(SHADOW_PATH, SHADOW)?;
                // One atomic attempt per boot, including across TITAN restarts.
                let mut marker = OpenOptions::new().write(true).create_new(true).open(&attempt).map_err(|e| format!("Startup attempt already owned or unavailable: {e}"))?;
                marker.write_all(b"ShadowMountPlus 1.7beta12 + kstuff-fpkg 1.13-dr-test6\n").map_err(|e| e.to_string())?;
                self.phase("starting", "Starting ShadowMount before kstuff");
                // ShadowMount creates its no-automount sentinel before waiting for kstuff.
                if shadow_process {
                    self.phase("waiting", "Existing ShadowMount process observed; waiting without restart");
                } else if let Err(e) = dispatch(SHADOW_PATH) { log!("unified: ShadowMount dispatch uncertain: {e}; not retrying"); }
                if !kstuff {
                    // Wait for the upstream sentinel rather than guessing readiness from a delay.
                    let mut sentinel = false;
                    for _ in 0..80 {
                        if Path::new("/data/.kstuff_noautomount").is_file() { sentinel = true; break; }
                        std::thread::sleep(Duration::from_millis(250));
                    }
                    if !sentinel { return Err("ShadowMount startup sentinel not observed; kstuff was not dispatched".into()); }
                    if !kstuff_ready() && !kstuff_process {
                        self.phase("starting", "Starting bundled kstuff-fpkg 1.13-dr-test6");
                        if let Err(e) = dispatch(KSTUFF_PATH) { log!("unified: kstuff dispatch uncertain: {e}; not retrying"); }
                    }
                }
            }
        }
        let deadline = std::time::Instant::now() + Duration::from_secs(90);
        while std::time::Instant::now() < deadline {
            let kstuff = kstuff_ready(); let shadow = kstuff && shadow_ready(port);
            { let mut s = self.snapshot.lock().unwrap_or_else(|e| e.into_inner()); s.kstuff = kstuff; s.shadow = shadow; }
            if kstuff && shadow { self.phase("ready", "Services verified ready"); return Ok(()); }
            std::thread::sleep(Duration::from_millis(500));
        }
        Err("Services did not become ready. Inspect Diagnostics; no repeated kernel-payload launch was attempted".into())
    }
}

fn import_policy_confirmed(v:&Value)->bool{v["saved"]==true && v["fakelib_mode"]=="disabled" && v["fakelib_effective_enabled"]==false}

fn normalize_game(g: &Value) -> Value {
    let kind = g["source_type"].as_str().unwrap_or("folder");
    let path = g["path"].as_str().unwrap_or("");
    let drive = if path.starts_with("/mnt/") { path.split('/').take(3).collect::<Vec<_>>().join("/") } else { "/data".into() };
    let available = g["source_available"].as_bool().unwrap_or(false);
    json!({"title_id":g["title_id"],"title_name":g["title_name"],"version":g["version"],"path":g["path"],
        "kind":kind,"source_available":available,"packing":null,"drive":drive,"size":g["size_bytes"].as_u64().or_else(||g["app_db_size_bytes"].as_u64()),
        "status":if !available {"error"} else if g["installed"] == true {"ready"} else {"pending"},
        "error":if available {Value::Null} else {json!("Source unavailable")},
        "cache_size":null,"cache_built":null,"backport":g["backport_path"],
        "engine":"shadowmountplus","image_type":g["image_type"],"platform":g["platform"],
        "installed_pkg":g["installed_pkg"]==true,"mounted":g["mounted"],"installed":g["installed"],"runtime_path":g["runtime_path"],"fakelib_mode":g["fakelib_mode"],"fakelib_mode_override":g["fakelib_mode_override"]})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    fn server(reply: Vec<u8>) -> (u16, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind(("127.0.0.1",0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            let (mut connection,_) = listener.accept().unwrap();
            connection.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            let mut request = Vec::new();
            loop {
                let mut bytes = [0u8;1024]; let n = connection.read(&mut bytes).unwrap(); request.extend_from_slice(&bytes[..n]);
                if let Some(cut) = request.windows(4).position(|b| b == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&request[..cut]);
                    let length = head.lines().find_map(|line|line.strip_prefix("Content-Length: ")).unwrap_or("0").parse::<usize>().unwrap();
                    if request.len() >= cut + 4 + length { break; }
                }
                assert!(n > 0);
            }
            connection.write_all(&reply).unwrap();
            String::from_utf8(request).unwrap()
        });
        (port,handle)
    }
    #[test] fn actual_http_client_frames_json_and_reads_response() {
        let body = br#"{"status":0,"api_version":1,"shadowmount_version":"1.7beta12"}"#;
        let mut response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",body.len()).into_bytes(); response.extend_from_slice(body);
        let (port,worker) = server(response);
        let result = engine_json(port,"/api/v1/version",json!({}),2).unwrap();
        assert_eq!(result["api_version"],1);
        let request = worker.join().unwrap(); assert!(request.starts_with("POST /api/v1/version HTTP/1.1\r\n")); assert!(request.ends_with("\r\n\r\n{}"));
    }
    #[test] fn truncated_or_streaming_reply_is_rejected() {
        for response in [b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\n{}".to_vec(),b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n".to_vec()] {
            let (port,worker)=server(response);assert!(http(port,"POST","/api/v1/version",b"{}",2).is_err());worker.join().unwrap();
        }
    }
    #[test] fn combined_startup_has_no_manual_profile_gate(){
        let stack=Stack::new();assert_eq!(stack.services()["mode"],"combined");assert_eq!(stack.services()["mount_owner"],"per-title");
    }
    #[test] fn native_claim_requires_fresh_snapshot_and_disjoint_shadow_title(){
        let stack=Stack::new();NATIVE_ALLOWED.store(true,Ordering::Release);
        assert!(!stack.native_owns("PPSA12345"));
        {let mut s=stack.snapshot.lock().unwrap();s.updated=true;s.games=vec![json!({"title_id":"PPSA12345","kind":"pkg"})];}
        assert!(stack.native_owns("PPSA12345"));
        {stack.snapshot.lock().unwrap().games[0]["kind"]=json!("image");}
        assert!(!stack.native_owns("PPSA12345"));
        assert!(stack.native_owns("PPSA54321"));
        NATIVE_ALLOWED.store(false,Ordering::Release);assert!(!stack.native_owns("PPSA54321"));
    }
    #[test] fn installed_pkg_metadata_is_preserved_without_mount_ownership(){
        let g=normalize_game(&json!({"title_id":"PPSA22715","title_name":"Installed fixture","path":"/user/app/PPSA22715/app.pkg","source_type":"pkg","installed_pkg":true,"installed":true,"source_available":true,"app_db_size_bytes":6000000000u64}));
        assert_eq!(g["installed_pkg"],true);assert_eq!(g["status"],"ready");assert_eq!(g["size"],6000000000u64);assert!(!crate::library::shadow_owns(&[g],"PPSA22715"));
    }
    #[test] fn import_overlay_requires_explicit_saved_disabled_confirmation(){assert!(import_policy_confirmed(&json!({"saved":true,"fakelib_mode":"disabled","fakelib_effective_enabled":false})));assert!(!import_policy_confirmed(&json!({"saved":true,"fakelib_mode":"full","fakelib_effective_enabled":true})));assert!(!import_policy_confirmed(&json!({"status":0})));}
    #[test] fn engine_error_is_not_reported_as_success() {
        let body=b"{\"status\":16}";let mut response=format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",body.len()).into_bytes();response.extend_from_slice(body);
        let (port,worker)=server(response);assert!(engine_json(port,"/api/v1/scan",json!({}),2).is_err());worker.join().unwrap();
    }
    #[test] fn installs_are_idempotent_and_do_not_overwrite_owner_files() {
        let dir=std::env::temp_dir().join(format!("titan-stack-{}",std::process::id()));fs::create_dir_all(&dir).unwrap();let path=dir.join("service.elf");let path=path.to_str().unwrap();
        install(path,b"pinned").unwrap(); install(path,b"pinned").unwrap(); assert!(install(path,b"changed").is_err());assert_eq!(fs::read(path).unwrap(),b"pinned");fs::remove_dir_all(dir).unwrap();
    }
    #[test] fn image_metadata_preserves_real_source_and_unknown_cache() {
        let g=normalize_game(&json!({"title_id":"PPSA12345","title_name":"Owner <game>","path":"/mnt/usb0/Game.ffpkg","source_type":"image","image_type":"ufs","source_available":true,"installed":true,"mounted":false,"size_bytes":123}));
        assert_eq!(g["kind"],"image");assert_eq!(g["drive"],"/mnt/usb0");assert_eq!(g["status"],"ready");assert_eq!(g["title_name"],"Owner <game>");assert!(g["cache_size"].is_null());
        assert_eq!(normalize_game(&json!({"installed":true,"source_available":false}))["status"],"error");
    }
}
