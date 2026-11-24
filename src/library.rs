//! Native FPKG data stays authoritative; exFAT discovery never mounts/writes an image.
use serde_json::{json,Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path,PathBuf};

pub fn installed_package_path(path:&Path)->bool{
    let p=path.to_string_lossy();
    ["/user/app/","/mnt/ext0/user/app/","/mnt/ext0/ps5/user/app/","/mnt/ext1/user/app/","/mnt/ext1/ps5/user/app/"].iter().any(|root|p.strip_prefix(root).is_some_and(|suffix|{
        let mut parts=suffix.split('/');matches!((parts.next(),parts.next(),parts.next()),(Some(tid),Some("app.pkg"),None) if crate::scan::valid_tid(tid))
    }))
}
pub fn image_id(path: &Path) -> String {
    let hash=path.to_string_lossy().bytes().fold(0xcbf29ce484222325u64,|h,b|(h^b as u64).wrapping_mul(0x100000001b3));
    format!("exfat-{hash:016x}")
}
pub fn discover_exfat(roots: &[String]) -> Vec<Value> {
    fn visit(path:&Path,depth:u8,seen:&mut HashSet<PathBuf>,out:&mut Vec<Value>) {
        let Ok(entries)=std::fs::read_dir(path) else{return};
        for entry in entries.flatten() {
            if out.len()>=4096{return}
            if entry.file_name().to_string_lossy().starts_with('.') {continue}
            let Ok(kind)=entry.file_type() else{continue};let path=entry.path();
            if kind.is_symlink(){continue}
            if kind.is_dir() && depth>0 && !path.join("eboot.bin").is_file() {visit(&path,depth-1,seen,out);continue}
            if !kind.is_file() || !path.extension().is_some_and(|e|e.eq_ignore_ascii_case("exfat")){continue}
            let canonical=std::fs::canonicalize(&path).unwrap_or(path.clone());if !seen.insert(canonical){continue}
            let size=entry.metadata().ok().map(|m|m.len());
            let name=path.file_stem().unwrap_or_default().to_string_lossy();
            out.push(json!({"source_id":image_id(&path),"title_id":"","title_name":name,"version":"",
                "path":path.to_string_lossy(),"drive":if path.starts_with("/mnt"){path.to_string_lossy().split('/').take(3).collect::<Vec<_>>().join("/")}else{"/data".into()},"kind":"image","image_type":"exfatfs","engine":"images",
                "status":"engine_required","size":size,"metadata_source":"filename","mounted":false,
                "cache_size":null,"cache_built":null,"backport":null,"error":null}));
        }
    }
    let mut out=Vec::new();let mut seen=HashSet::new();
    for root in roots {visit(Path::new(root),2,&mut seen,&mut out)}
    out.sort_by(|a,b|a["path"].as_str().cmp(&b["path"].as_str()));out
}
/// Shadow-managed folders/images own their title's base mount and launch overlays.
pub fn shadow_owns(games:&[Value],tid:&str)->bool{
    games.iter().any(|g|g["title_id"]==tid && matches!(g["kind"].as_str(),Some("image"|"folder")))
}
/// Keep native FPKGs visible. Merge sources by path, never discard matching-title images.
/// Merge source inventory without repeatedly searching full engine and native
/// lists for every game on each /api/state poll. Hash indexes preserve the
/// original first-match semantics for duplicate title/path combinations.
pub fn merge(native:Vec<Value>,images:Vec<Value>,engine:Vec<Value>,_legacy_image_mode:bool)->Vec<Value>{
    let mut out=native;
    let shadow_titles:HashSet<&str>=engine.iter()
        .filter(|g|matches!(g["kind"].as_str(),Some("image"|"folder")))
        .filter_map(|g|g["title_id"].as_str()).collect();
    let mut remote_by_key:HashMap<(&str,&str),&Value>=HashMap::new();
    for g in &engine {
        if let (Some(tid),Some(path))=(g["title_id"].as_str(),g["path"].as_str()) {
            remote_by_key.entry((tid,path)).or_insert(g);
        }
    }
    for game in &mut out {
        game["engine"]=json!("native");
        let tid=game["title_id"].as_str().unwrap_or("");
        if shadow_titles.contains(tid) {
            let path=game["path"].as_str().unwrap_or("");
            if let Some(remote)=remote_by_key.get(&(tid,path)){
                let mut delegated=(*remote).clone();
                delegated["engine"]=json!("shadowmountplus");
                *game=delegated;
            }else{
                game["status"]=json!("shadow_owned");
                game["managed_by"]=json!("shadowmountplus");
            }
        }
    }
    let mut out_by_path:HashMap<String,usize>=HashMap::new();
    for (i,g) in out.iter().enumerate(){
        if let Some(path)=g["path"].as_str(){out_by_path.entry(path.to_owned()).or_insert(i);}
    }
    let mut by_path=std::collections::BTreeMap::new();
    for game in images{if let Some(path)=game["path"].as_str(){by_path.insert(path.to_string(),game);}}
    for mut game in engine {
        if !matches!(game["kind"].as_str(),Some("image"|"folder"))
            && !(game["kind"]=="pkg" && game["installed_pkg"]==true && game["installed"]==true){continue}
        if let Some(path)=game["path"].as_str().map(String::from){
            if let Some(&idx)=out_by_path.get(&path){
                let existing=&mut out[idx];
                if game["installed_pkg"]==true && existing["native_installed"]!=true {
                    game["source_id"]=json!(format!("installed-{}",image_id(Path::new(&path))));
                    game["engine"]=json!("shadowmountplus");
                    *existing=game;
                }
                continue;
            }
            if game["installed_pkg"]==true && game["status"]!="error"{
                game["status"]=json!("preparation_pending");
                if let Some(native)=out.iter().find(|n|n["engine"]=="native" && n["title_id"]==game["title_id"]){
                    game["status"]=native["status"].clone();
                    game["launch_source"]=native["path"].clone();
                }
            }
            game["source_id"]=json!(if game["installed_pkg"]==true {
                format!("installed-{}",image_id(Path::new(&path)))
            }else{
                image_id(Path::new(&path))
            });
            game["engine"]=json!("shadowmountplus");
            by_path.insert(path,game);
        }
    }
    out.extend(by_path.into_values());
    out
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn native_pkg_cannot_disappear_when_image_provider_is_empty(){
        let games=merge(vec![json!({"title_id":"PPSA12345","kind":"pkg","path":"/usb/g.pkg","status":"ready"})],vec![],vec![],false);
        assert_eq!(games.len(),1);assert_eq!(games[0]["status"],"ready");assert_eq!(games[0]["engine"],"native");
    }
    #[test]fn shared_title_id_does_not_hide_image_or_pkg(){
        let n=json!({"title_id":"PPSA12345","kind":"pkg","path":"/g.pkg","status":"ready"});
        let local=json!({"title_id":"","kind":"image","image_type":"exfatfs","path":"/g.exfat","status":"engine_required"});
        let remote=json!({"title_id":"PPSA12345","kind":"image","image_type":"exfatfs","path":"/g.exfat","status":"ready"});
        let games=merge(vec![n],vec![local],vec![remote],false);
        assert_eq!(games.len(),2);assert_eq!(games[0]["status"],"shadow_owned");assert_eq!(games[1]["status"],"ready");
    }
    #[test]fn native_package_inventory_is_not_shadow_mount_ownership(){
        assert!(!shadow_owns(&[json!({"title_id":"PPSA12345","kind":"pkg"})],"PPSA12345"));
        assert!(shadow_owns(&[json!({"title_id":"PPSA12345","kind":"folder"})],"PPSA12345"));
    }
    #[test]fn both_engines_ready_keeps_disjoint_native_and_image_sources_ready(){
        let games=merge(vec![json!({"title_id":"PPSA12345","kind":"pkg","path":"/a.pkg","status":"ready"})],vec![],vec![json!({"title_id":"PPSA54321","kind":"image","path":"/b.exfat","status":"ready"})],true);
        assert_eq!(games.len(),2);assert!(games.iter().all(|g|g["status"]=="ready"));
    }
    #[test]fn installed_pkg_is_added_alongside_native_source_with_same_title(){
        let native=json!({"title_id":"PPSA22715","kind":"pkg","path":"/data/homebrew/game.pkg","status":"ready"});
        let installed=json!({"title_id":"PPSA22715","kind":"pkg","installed_pkg":true,"installed":true,"path":"/user/app/PPSA22715/app.pkg","status":"ready"});
        let games=merge(vec![native],vec![],vec![installed],false);assert_eq!(games.len(),2);assert_eq!(games[0]["engine"],"native");assert_eq!(games[0]["status"],"ready");assert_eq!(games[1]["installed_pkg"],true);assert_eq!(games[1]["engine"],"shadowmountplus");assert_ne!(games[1]["source_id"],games[0]["source_id"]);
    }
    #[test]fn system_installed_pkg_paths_are_separate_from_source_files(){
        assert!(installed_package_path(Path::new("/user/app/PPSA22715/app.pkg")));
        assert!(installed_package_path(Path::new("/mnt/ext0/ps5/user/app/PPSA22715/app.pkg")));
        assert!(!installed_package_path(Path::new("/data/homebrew/app.pkg")));
        assert!(!installed_package_path(Path::new("/user/app/not-a-title/app.pkg")));
    }
    #[test]fn native_installed_preparation_is_never_overwritten_by_inventory_ready(){
        let native=json!({"title_id":"PPSA22715","kind":"pkg","path":"/user/app/PPSA22715/app.pkg","installed_pkg":true,"native_installed":true,"status":"building"});let remote=json!({"title_id":"PPSA22715","kind":"pkg","path":"/user/app/PPSA22715/app.pkg","installed_pkg":true,"installed":true,"status":"ready"});let games=merge(vec![native],vec![],vec![remote],false);assert_eq!(games.len(),1);assert_eq!(games[0]["status"],"building");assert_eq!(games[0]["engine"],"native");
    }
    #[test]fn inventory_alone_does_not_claim_launch_readiness(){let g=json!({"title_id":"PPSA22715","kind":"pkg","path":"/user/app/PPSA22715/app.pkg","installed_pkg":true,"installed":true,"status":"ready"});assert_eq!(merge(vec![],vec![],vec![g],false)[0]["status"],"preparation_pending");}
    #[test]fn uninstalled_pkg_inventory_cannot_be_presented_as_installed(){
        let games=merge(vec![],vec![],vec![json!({"title_id":"PPSA22715","kind":"pkg","path":"/tmp/uninstalled.pkg","installed_pkg":true,"installed":false})],false);assert!(games.is_empty());
    }
    #[test]fn same_physical_source_is_never_duplicated(){
        let g=json!({"title_id":"PPSA22715","kind":"pkg","path":"/user/app/PPSA22715/app.pkg","installed_pkg":true,"installed":true,"status":"ready"});assert_eq!(merge(vec![g.clone()],vec![],vec![g],false).len(),1);
    }
    #[test]fn discovers_exfat_case_insensitively_without_mounting_or_modifying(){
        let dir=std::env::temp_dir().join(format!("titan-exfat-{}",std::process::id()));std::fs::create_dir_all(dir.join("nested")).unwrap();
        std::fs::write(dir.join("nested/Owner.EXFAT"),b"fixture-only").unwrap();std::fs::write(dir.join("._Owner.exfat"),b"ignored").unwrap();
        let roots=vec![dir.to_string_lossy().into_owned(),dir.join("nested").to_string_lossy().into_owned()];let games=discover_exfat(&roots);
        assert_eq!(games.len(),1);assert_eq!(games[0]["title_name"],"Owner");assert_eq!(games[0]["title_id"],"");assert_eq!(games[0]["mounted"],false);
        assert_eq!(std::fs::read(dir.join("nested/Owner.EXFAT")).unwrap(),b"fixture-only");std::fs::remove_dir_all(dir).unwrap();
    }
}
