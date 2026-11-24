//! Managed PKG installation on internal or explicitly selected connected external storage.
use std::{fs,io::{self,Write},path::{Path,PathBuf},sync::{Mutex,atomic::{AtomicBool,Ordering}}};
use crate::installer_io::{transfer,verify};
use serde_json::{json,Value};
use std::os::unix::fs::{OpenOptionsExt,PermissionsExt,MetadataExt};
pub const ROOT:&str="/user/app";
pub const LEGACY_ROOT:&str="/data/homebrew/titan-core/installed";
const RESERVE:u64=256<<20;
struct Request{path:PathBuf,destination:String}
struct Destination{mount:PathBuf,root:PathBuf,external:bool,free:u64,dev:u64,ino:u64,base:PathBuf,base_dev:u64,base_ino:u64}
impl Destination{
 fn resolve(mount:&str)->io::Result<Self>{
  let d=crate::storage::drives().into_iter().find(|d|d.mount==mount).ok_or_else(||io::Error::other("Destination drive is disconnected"))?;
  let p=PathBuf::from(&d.mount);if p.canonicalize()?!=p{return Err(io::Error::other("Destination mount must not be a symlink"));}
  let m=fs::metadata(&p)?;let external=d.kind!="internal";let base=if external{p.clone()}else{PathBuf::from("/user")};if base.canonicalize()?!=base{return Err(io::Error::other("Destination base must not be a symlink"));}let bm=fs::metadata(&base)?;
  Ok(Self{root:if external{p.join("TITAN-CORE/Games")}else{PathBuf::from(ROOT)},mount:p,external,free:d.free,dev:m.dev(),ino:m.ino(),base,base_dev:bm.dev(),base_ino:bm.ino()})
 }
 fn check(&self)->io::Result<()>{let m=fs::metadata(&self.mount)?;if m.dev()!=self.dev||m.ino()!=self.ino||fs::metadata(&self.base).map_or(true,|b|b.dev()!=self.base_dev||b.ino()!=self.base_ino){return Err(io::Error::other("Destination drive changed or disconnected"));}Ok(())}
 fn create_root(&self)->io::Result<()>{
  self.check()?;let tail=self.root.strip_prefix(&self.base).map_err(io::Error::other)?;let mut p=self.base.clone();
  for part in tail.components(){p.push(part);match fs::symlink_metadata(&p){Ok(m)if !m.is_dir()||m.file_type().is_symlink()=>return Err(io::Error::other("Installation folder must not contain symlinks")),Ok(_)=>{},Err(e)if e.kind()==io::ErrorKind::NotFound=>fs::create_dir(&p)?,Err(e)=>return Err(e)}if p.canonicalize()?!=p{return Err(io::Error::other("Installation folder changed"));}}
  self.check()
 }
}
pub fn destinations()->Value{json!(crate::storage::drives().iter().map(|d|json!({"path":d.mount,"name":d.label,"free":d.free,"external":d.kind!="internal","folder":if d.kind=="internal"{format!("{ROOT}/<TITLE_ID>/app.pkg")}else{format!("{}/TITAN-CORE/Games",d.mount)}})).collect::<Vec<_>>())}
#[derive(Default)]pub struct Installer{pub job:Mutex<Value>,pending:Mutex<Option<Request>>,cancel:AtomicBool}
impl Installer{
 pub fn status(&self)->Value{self.job.lock().unwrap().clone()}
 pub fn active(&self)->bool{matches!(self.status()["phase"].as_str(),Some("queued"|"copying"|"verifying"))}
 pub fn cancel(&self){self.cancel.store(true,Ordering::SeqCst);}
 pub fn queue(&self,path:PathBuf)->io::Result<()>{self.queue_to(path,"/data")}
 pub fn queue_to(&self,path:PathBuf,destination:&str)->io::Result<()>{
  Destination::resolve(destination)?;let mut job=self.job.lock().unwrap();if matches!(job["phase"].as_str(),Some("queued"|"copying"|"verifying")){return Err(io::Error::other("An installation is already active"));}
  self.cancel.store(false,Ordering::SeqCst);*self.pending.lock().unwrap()=Some(Request{path:path.clone(),destination:destination.into()});*job=json!({"phase":"queued","path":path,"destination":destination,"done":0,"total":0});Ok(())
 }
 fn progress(&self,phase:&str,done:u64,total:u64){let mut j=self.job.lock().unwrap();j["phase"]=json!(phase);j["done"]=json!(done);j["total"]=json!(total);}
 fn check(&self,state:&crate::web::State,dest:&Destination)->io::Result<()>{if self.cancel.load(Ordering::SeqCst)||state.paused(){return Err(io::Error::other("Installation stopped; uncommitted source kept"));}dest.check()}
 pub fn run(&self,state:&crate::web::State){let Some(request)=self.pending.lock().unwrap().take()else{return};let path=request.path;let result=(||->io::Result<()>{
  let dest=Destination::resolve(&request.destination)?;self.check(state,&dest)?;
  let source=external(&path,false)?;let meta=fs::metadata(&source)?;let size=meta.len();let(tid,name,_)=crate::cache::peek_pkg(&source)?;
  if !tid.starts_with("PPSA")||!crate::scan::valid_tid(&tid){return Err(io::Error::other("Only supported PS5 debug PKGs can be installed"));}
  if dest.external&&crate::storage::data_free_bytes().unwrap_or(0)<RESERVE{return Err(io::Error::other("Internal storage needs free space for the game cache"));}
  let needed=size.saturating_add(RESERVE);
  if dest.free<needed{return Err(io::Error::other("Not enough destination storage for PKG and working reserve"));}
  if state.games.lock().unwrap().iter().any(|g|g.src.title_id==tid&&(g.src.drive=="/data"||(dest.external&&g.src.path!=source))){return Err(io::Error::other("This title already has another library source; nothing was overwritten"));}
  dest.create_root()?;let dir=dest.root.join(&tid);
  let created_dir=match fs::create_dir(&dir){Ok(())=>true,Err(e)if e.kind()==io::ErrorKind::AlreadyExists&&!dest.external=>{
   let m=fs::symlink_metadata(&dir)?;let param=dir.join("sce_sys/param.json");
   if !m.is_dir()||m.file_type().is_symlink()||dir.canonicalize()?!=dir||param.canonicalize()?!=param||!metadata_title_matches(&param,&tid)||crate::cache::pkg_source(&tid).as_deref()!=source.to_str()||dir.join("eboot.bin").exists(){return Err(io::Error::other("Existing system title is not this TITAN source; nothing was overwritten"));}false
  },Err(e)=>return Err(e)};
  for n in ["package.part","install.json","app.pkg"]{if fs::symlink_metadata(dir.join(n)).is_ok(){if created_dir{let _=fs::remove_dir(&dir);}return Err(io::Error::other("System destination already contains package data; nothing was overwritten"));}}
  let owned=fs::metadata(&dir)?;let mut created_files:Vec<(PathBuf,u64,u64)>=Vec::new();let mut created_sys:Option<(u64,u64)>=None;
  let owned_dir=||fs::symlink_metadata(&dir).is_ok_and(|m|m.is_dir()&&!m.file_type().is_symlink()&&m.dev()==owned.dev()&&m.ino()==owned.ino())&&dir.canonicalize().ok().as_ref()==Some(&dir);
  let result=(||->io::Result<()>{
   let mode=owned.permissions().mode()&0o7777;if !dest.external&&created_dir{fs::set_permissions(&dir,fs::Permissions::from_mode(0o755))?;}else if mode&0o555!=0o555{fs::set_permissions(&dir,fs::Permissions::from_mode(mode|0o555))?;}
   let tmp=dir.join("package.part");let dst=dir.join(if dest.external{format!("{tid}.pkg")}else{"app.pkg".into()});
    let mut input=fs::File::open(&source)?;let mut out=fs::OpenOptions::new().write(true).create_new(true).mode(0o644).open(&tmp)?;let om=out.metadata()?;created_files.push((tmp.clone(),om.dev(),om.ino()));
    self.progress("copying",0,size);transfer(&mut input,&mut out,size,|done|{self.check(state,&dest)?;if !owned_dir(){return Err(io::Error::other("Destination folder changed"));}self.progress("copying",done,size);Ok(())})?;out.sync_all()?;drop(out);
    self.progress("verifying",0,size);verify(&source,&tmp,size,|done|{self.check(state,&dest)?;if !owned_dir(){return Err(io::Error::other("Destination folder changed"));}self.progress("verifying",done,size);Ok(())})?;
    let(read_tid,_,_)=crate::cache::peek_pkg(&tmp)?;if read_tid!=tid{return Err(io::Error::other("Copied package title mismatch"));}
   let now=fs::metadata(&source)?;if now.dev()!=meta.dev()||now.ino()!=meta.ino()||now.modified()?!=meta.modified()?||now.len()!=size||source.canonicalize()?!=source{return Err(io::Error::other("Source package changed during installation"));}
   self.check(state,&dest)?;if !owned_dir()||dst.exists(){return Err(io::Error::other("Destination changed; nothing was overwritten"));}
   if !dest.external&&created_dir{
    let param=crate::cache::pkg_file(&tmp,"sce_sys/param.json")?;let parsed:Value=serde_json::from_slice(&param).map_err(io::Error::other)?;
    if parsed["titleId"].as_str().or_else(||parsed["title_id"].as_str())!=Some(tid.as_str()){return Err(io::Error::other("System metadata title mismatch"));}
    let icon=crate::cache::pkg_file(&tmp,"sce_sys/icon0.png")?;let sys=dir.join("sce_sys");fs::create_dir(&sys)?;let sm=fs::metadata(&sys)?;created_sys=Some((sm.dev(),sm.ino()));fs::set_permissions(&sys,fs::Permissions::from_mode(0o755))?;
    for (p,bytes) in [(sys.join("param.json"),param),(sys.join("icon0.png"),icon)]{
     self.check(state,&dest)?;if !owned_dir()||sys.canonicalize()?!=sys{return Err(io::Error::other("System metadata destination changed"));}
     let mut f=fs::OpenOptions::new().write(true).create_new(true).mode(0o644).open(&p)?;let fm=f.metadata()?;created_files.push((p,fm.dev(),fm.ino()));f.write_all(&bytes)?;f.sync_all()?;
    }
   }
   let receipt=json!({"owner":"TITAN CORE installer","title_id":tid,"size":size,"source":source,"destination":dest.mount,"operation":"copy","original_mode":meta.permissions().mode()&0o7777});
   let mut f=fs::OpenOptions::new().write(true).create_new(true).open(dir.join("install.json"))?;let fm=f.metadata()?;created_files.push((dir.join("install.json"),fm.dev(),fm.ino()));f.write_all(receipt.to_string().as_bytes())?;f.sync_all()?;drop(f);
   self.check(state,&dest)?;if !owned_dir()||dst.exists(){return Err(io::Error::other("Destination changed; source kept"));}
   // Commit only the verified staging copy; the download original stays untouched.
   fs::rename(&tmp,&dst)?;
   *self.job.lock().unwrap()=json!({"phase":"complete","title_id":tid,"name":name,"path":dst,"destination":dest.mount,"operation":"copy","done":size,"total":size});Ok(())
  })();
  if result.is_err()&&dest.check().is_ok()&&owned_dir(){
   for (p,dev,ino) in created_files.iter().rev(){if p.canonicalize().ok().as_ref()==Some(p)&&fs::symlink_metadata(p).is_ok_and(|m|m.is_file()&&!m.file_type().is_symlink()&&m.dev()==*dev&&m.ino()==*ino){let _=fs::remove_file(p);}}
   let sys=dir.join("sce_sys");if created_sys.is_some_and(|(dev,ino)|sys.canonicalize().ok().as_ref()==Some(&sys)&&fs::symlink_metadata(&sys).is_ok_and(|m|m.is_dir()&&m.dev()==dev&&m.ino()==ino)){let _=fs::remove_dir(sys);}
   if created_dir{let _=fs::remove_dir(&dir);}
  }result
 })();if let Err(e)=result{log!("PKG installation {}: {e}",path.display());let mut j=self.job.lock().unwrap();j["phase"]=json!("error");j["error"]=json!(e.to_string());}state.wake();}
}
pub fn external(path:&Path,directory:bool)->io::Result<PathBuf>{
 if path.components().any(|c|!matches!(c,std::path::Component::RootDir|std::path::Component::Normal(_))){return Err(io::Error::other("Invalid external path"));}
 let canonical=path.canonicalize()?;if canonical!=path{return Err(io::Error::other("External symlinks are not supported"));}
 let allowed=crate::storage::drives().iter().filter(|d|d.kind!="internal").any(|d|canonical.starts_with(&d.mount));if !allowed{return Err(io::Error::other("Select a connected USB or extended drive"));}
 let m=fs::metadata(&canonical)?;if directory&&!m.is_dir()||!directory&&(!m.is_file()||!canonical.extension().is_some_and(|x|x.eq_ignore_ascii_case("pkg"))){return Err(io::Error::other("Select a PKG file or folder"));}Ok(canonical)
}
pub fn browse(path:&str)->io::Result<Value>{let drives=crate::storage::drives();let external_drives=drives.iter().filter(|d|d.kind!="internal").map(|d|json!({"name":d.label,"path":d.mount,"directory":true})).collect::<Vec<_>>();if path.is_empty(){return Ok(json!({"path":"","entries":external_drives}));}let dir=external(Path::new(path),true)?;let mut entries=Vec::new();for entry in fs::read_dir(&dir)?.take(1025){let e=entry?;let m=e.file_type()?;if m.is_symlink(){continue;}let p=e.path();if m.is_dir()||p.extension().is_some_and(|x|x.eq_ignore_ascii_case("pkg")){entries.push(json!({"name":e.file_name().to_string_lossy(),"path":p,"directory":m.is_dir(),"size":if m.is_file(){e.metadata()?.len()}else{0}}));}}entries.sort_by_key(|e|(!e["directory"].as_bool().unwrap_or(false),e["name"].as_str().unwrap_or("").to_lowercase()));Ok(json!({"path":dir,"entries":entries,"limit":1025}))}
fn managed_roots()->Vec<PathBuf>{let mut roots:Vec<_>=crate::storage::drives().iter().map(|d|if d.kind=="internal"{PathBuf::from(ROOT)}else{Path::new(&d.mount).join("TITAN-CORE/Games")}).collect();roots.push(PathBuf::from(LEGACY_ROOT));roots}
pub fn roots()->Vec<String>{let mut roots=Vec::new();for root in managed_roots(){if root.canonicalize().ok().as_ref()!=Some(&root){continue;}let Ok(entries)=fs::read_dir(&root)else{continue};for e in entries.flatten(){let dir=e.path();let tid=e.file_name().to_string_lossy().into_owned();if !crate::scan::valid_tid(&tid)||dir.canonicalize().ok().as_ref()!=Some(&dir){continue;}let Ok(data)=fs::read(dir.join("install.json"))else{continue};let Ok(v)=serde_json::from_slice::<Value>(&data)else{continue};let pkg=dir.join(if root==Path::new(ROOT){"app.pkg".into()}else{format!("{tid}.pkg")});if v["owner"]=="TITAN CORE installer"&&v["title_id"]==tid&&pkg.canonicalize().ok().as_ref()==Some(&pkg)&&fs::symlink_metadata(&pkg).is_ok_and(|m|m.is_file()&&Some(m.len())==v["size"].as_u64()){roots.push(dir.to_string_lossy().into_owned());}}}roots.sort();roots}
/// Remove only an empty owned receipt after separately confirmed source deletion.
fn metadata_title_matches(path:&Path,tid:&str)->bool{fs::metadata(path).is_ok_and(|m|m.is_file()&&m.len()<=8<<20)&&fs::read(path).ok().and_then(|b|serde_json::from_slice::<Value>(&b).ok()).is_some_and(|v|v["titleId"].as_str().or_else(||v["title_id"].as_str())==Some(tid))}
pub fn forget_empty(pkg:&Path)->io::Result<()>{let Some(dir)=pkg.parent()else{return Ok(())};if !managed_roots().iter().any(|r|dir.parent()==Some(r.as_path()))||dir.canonicalize().ok().as_deref()!=Some(dir)||pkg.exists(){return Ok(());}let tid=dir.file_name().and_then(|n|n.to_str()).unwrap_or("");let expected=if dir.parent()==Some(Path::new(ROOT)){"app.pkg".to_owned()}else{format!("{tid}.pkg")};if !crate::scan::valid_tid(tid)||pkg.file_name().and_then(|n|n.to_str())!=Some(expected.as_str()){return Ok(());}let receipt=dir.join("install.json");let v:Value=serde_json::from_slice(&fs::read(&receipt)?).map_err(io::Error::other)?;if v["owner"]!="TITAN CORE installer"||v["title_id"]!=tid{return Ok(());}let entries=fs::read_dir(dir)?.collect::<Result<Vec<_>,_>>()?;if entries.len()==1&&entries[0].path()==receipt{fs::remove_file(receipt)?;fs::remove_dir(dir)?;}Ok(())}

/// UI classification only: require the same validated receipt roots used by discovery.
/// Keep system-installed flags separate so external packages retain their native controls.
pub fn external_installs()->Vec<PathBuf>{
 let bases:Vec<_>=crate::storage::drives().iter().filter(|d|d.kind!="internal").map(|d|Path::new(&d.mount).join("TITAN-CORE/Games")).collect();
 roots().iter().filter_map(|root|{
  let dir=Path::new(root);if !bases.iter().any(|base|dir.parent()==Some(base.as_path())){return None;}
  Some(dir.join(format!("{}.pkg",dir.file_name()?.to_str()?)))
 }).collect()
}
#[cfg(test)]pub fn external_install(pkg:&Path)->bool{external_installs().iter().any(|p|p==pkg)}
