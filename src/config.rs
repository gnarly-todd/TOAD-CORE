//! Settings from /data/titan-core/config.toml (DESIGN.md "Settings").
//! ponytail: a flat TOML subset (key = string | bool | integer | array of strings, # comments),
//! no tables; a toml crate when the file needs more.
use serde_json::{json, Value};
pub const CONFIG_PATH: &str = "/data/titan-core/config.toml";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FakelibMode {
    Emulators,
    Full,
    Off,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Settings {
    pub scan_paths: Vec<String>,
    pub inject_ampr: bool,
    /// Enable verified upstream DLC/PlayGo emulator injection where requested by a game.
    pub add_emulators: bool,
    /// Title IDs which do not receive the bundled PlayGo stub (game-owned copies are kept).
    pub playgo_off: Vec<String>,
    pub auto_import_installed: bool,
    pub game_fakelib: FakelibMode,
    pub backports: bool,
    pub scan_interval_secs: u64,
    /// Seconds a newly modified source must remain unchanged before it is processed.
    pub stability_wait_secs: u64,
    pub low_space_gb: u64,
    pub notify: bool,
    /// Write /data/titan-core/titan-core.log (the payload's stdout always gets the lines).
    pub log: bool,
    pub debug: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            scan_paths: Vec::new(),
            inject_ampr: true,
            add_emulators: true,
            playgo_off: Vec::new(),
            auto_import_installed: true,
            game_fakelib: FakelibMode::Emulators,
            backports: true,
            scan_interval_secs: 10,
            stability_wait_secs: 10,
            low_space_gb: 20,
            notify: true,
            log: true,
            debug: false,
        }
    }
}


const HELP: &[(&str, &str)] = &[
    ("scanpath", "replaces the default scan roots (/data/homebrew, /data/etaHEN/games, <drive>/homebrew, ...)"),
    ("auto_import_installed", "prepare supported installed PS5 PKGs in their original system location for native launch"),
    ("inject_ampr", "true = copy /data/titan-core/ampr/* on top of folder-game fakelib; PKG games always require AMPR"),
    ("add_emulators", "true = add upstream DLC and PlayGo emulators when needed without replacing newer game copies"),
    ("playgo_off", "list of title IDs which do not get the bundled PlayGo stub; game-owned copies stay unchanged"),
    ("game_fakelib", "game's own fakelib: emulators | full | off"),
    ("backports", "true = apply backports/<TID>* from the game's scan root, then the other roots, then /data/homebrew"),
    ("scan_interval_seconds", "scan loop period, 1..3600"),
    ("stability_wait_seconds", "skip source files recently changed (still being copied), 0..3600, 0 disables"),
    ("low_space_gb", "warn when /data free space is below this"),
    ("notify", "true = system toasts"),
    ("log", "true = write /data/titan-core/titan-core.log (starts over past 1 MiB)"),
    ("debug", "true = klog capture to /data/titan-core/klog.txt and the /file/data/... route"),
];

fn flag(v: &str) -> Result<bool, String> {
    match v {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        _ => Err(format!("expected 0 or 1, got {v:?}")),
    }
}

fn num(v: &str) -> Result<u64, String> {
    v.parse().map_err(|_| format!("expected a number, got {v:?}"))
}

/// Applies one key=value. Unknown keys are an error too.
fn set(s: &mut Settings, key: &str, v: &str) -> Result<(), String> {
    match key {
        "scanpath" if v.starts_with('/') => {
            // A repeated root would list every game in it again as its own duplicate.
            let v = v.trim_end_matches('/');
            let v = if v.is_empty() { "/" } else { v };
            if !s.scan_paths.iter().any(|p| p == v) {
                s.scan_paths.push(v.to_string());
            }
        }
        "scanpath" => return Err(format!("expected an absolute path, got {v:?}")),
        "inject_ampr" => s.inject_ampr = flag(v)?,
        "add_emulators" => s.add_emulators = flag(v)?,
        "playgo_off" if crate::scan::valid_tid(v) => {
            if !s.playgo_off.iter().any(|t| t == v) {
                s.playgo_off.push(v.to_string());
            }
        }
        "playgo_off" => return Err(format!("expected a title ID such as PPSA01556, got {v:?}")),
        "auto_import_installed" => s.auto_import_installed = flag(v)?,
        "game_fakelib" => {
            s.game_fakelib = match v {
                "emulators" => FakelibMode::Emulators,
                "full" => FakelibMode::Full,
                "off" => FakelibMode::Off,
                _ => return Err(format!("expected emulators|full|off, got {v:?}")),
            }
        }
        "backports" => s.backports = flag(v)?,
        "scan_interval_seconds" => match num(v)? {
            n @ 1..=3600 => s.scan_interval_secs = n,
            n => return Err(format!("{n} not in 1..3600")),
        },
        "stability_wait_seconds" => match num(v)? {
            n @ 0..=3600 => s.stability_wait_secs = n,
            n => return Err(format!("{n} not in 0..3600")),
        },
        "low_space_gb" => s.low_space_gb = num(v)?,
        "notify" => s.notify = flag(v)?,
        "log" => s.log = flag(v)?,
        "debug" => s.debug = flag(v)?,
        _ => return Err("unknown key".into()),
    }
    Ok(())
}

fn fakelib_str(m: FakelibMode) -> &'static str {
    match m {
        FakelibMode::Emulators => "emulators",
        FakelibMode::Full => "full",
        FakelibMode::Off => "off",
    }
}

/// A TOML string at the start of `v` ("basic" with \\ \" \n \t escapes, or 'literal') and the rest.
fn toml_str(v: &str) -> Result<(String, &str), String> {
    if let Some(r) = v.strip_prefix('\'') {
        let end = r.find('\'').ok_or("unclosed string")?;
        return Ok((r[..end].to_string(), &r[end + 1..]));
    }
    let r = v.strip_prefix('"').ok_or_else(|| format!("expected a string, got {v:?}"))?;
    let mut out = String::new();
    let mut chars = r.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Ok((out, &r[i + 1..])),
            '\\' => match chars.next().map(|(_, e)| e) {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                e => return Err(format!("unsupported escape \\{}", e.unwrap_or(' '))),
            },
            c => out.push(c),
        }
    }
    Err("unclosed string".into())
}

/// `v` without a trailing `# comment` outside strings.
fn strip_comment(v: &str) -> &str {
    let (mut quote, mut esc) = (None, false);
    for (i, c) in v.char_indices() {
        if esc {
            esc = false;
            continue;
        }
        match (quote, c) {
            (None, '#') => return &v[..i],
            (None, '"' | '\'') => quote = Some(c),
            (Some('"'), '\\') => esc = true,
            (Some(q), c) if c == q => quote = None,
            _ => {}
        }
    }
    v
}

/// A TOML value as the texts `set` takes: bool -> "1"/"0", integer, string, or one text per array item.
fn toml_value(v: &str) -> Result<Vec<String>, String> {
    let v = strip_comment(v).trim();
    if let Some(inner) = v.strip_prefix('[') {
        let mut rest = inner.strip_suffix(']').ok_or("unclosed array")?.trim();
        let mut out = Vec::new();
        while !rest.is_empty() {
            let (item, r) = toml_str(rest)?;
            out.push(item);
            rest = r.trim_start();
            rest = match rest.strip_prefix(',') {
                Some(r) => r.trim_start(),
                None if rest.is_empty() => rest,
                None => return Err(format!("expected , in the array, got {rest:?}")),
            };
        }
        return Ok(out);
    }
    if v.starts_with('"') || v.starts_with('\'') {
        let (text, rest) = toml_str(v)?;
        if !rest.trim().is_empty() {
            return Err(format!("unexpected {:?} after the string", rest.trim()));
        }
        return Ok(vec![text]);
    }
    match v {
        "true" => Ok(vec!["1".into()]),
        "false" => Ok(vec!["0".into()]),
        n if !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit() || b == b'_') => Ok(vec![n.replace('_', "")]),
        _ => Err(format!("expected a string, true/false, a number or an array, got {v:?}")),
    }
}

fn toml_quote(v: &str) -> String {
    format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Config text of the last parse; warnings are logged only when the text changed, since
/// load() runs every scan loop.
static LAST_TEXT: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

fn parse(text: &str) -> Settings {
    let quiet = {
        let mut last = LAST_TEXT.lock().unwrap_or_else(|e| e.into_inner());
        let same = *last == text;
        if !same {
            *last = text.to_owned();
        }
        same
    };
    let mut s = Settings::default();
    let mut lines = text.lines().enumerate();
    while let Some((n, line)) = lines.next() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            if !quiet {
                log!("config {CONFIG_PATH}:{}: not key = value: {line:?}", n + 1);
            }
            continue;
        };
        let (k, mut v) = (k.trim(), v.trim().to_string());
        // A multi-line array: join lines up to the closing ].
        if v.starts_with('[') {
            while !strip_comment(&v).trim_end().ends_with(']') {
                let Some((_, more)) = lines.next() else { break };
                v = format!("{} {}", strip_comment(&v).trim_end(), more.trim());
            }
        }
        let r = toml_value(&v).and_then(|vals| {
            if k == "scanpath" {
                s.scan_paths.clear();
            } else if k == "playgo_off" {
                s.playgo_off.clear();
            } else if vals.len() != 1 {
                return Err("expected one value, not an array".into());
            } else {
                return set(&mut s, k, &vals[0]);
            }
            // Kraken 0.3.1: one bad array element does not discard the rest.
            for x in &vals {
                if let (Err(e), false) = (set(&mut s, k, x), quiet) {
                    log!("config {CONFIG_PATH}:{}: {k}: {e}; skipped", n + 1);
                }
            }
            Ok(())
        });
        if let (Err(e), false) = (r, quiet) {
            log!("config {CONFIG_PATH}:{}: {k}: {e}; using the default", n + 1);
        }
    }
    s
}

fn render(s: &Settings) -> String {
    let mut out = String::from("# TITAN CORE settings (TOML). The web UI edits this file.\n");
    for &(k, help) in HELP {
        out += &format!("\n# {help}\n");
        match k {
            // ponytail: no scanpath = default roots, shown as an example only.
            "scanpath" if s.scan_paths.is_empty() => out += "# scanpath = [\"/data/homebrew\"]\n",
            "scanpath" => {
                let items: Vec<String> = s.scan_paths.iter().map(|p| toml_quote(p)).collect();
                out += &format!("scanpath = [{}]\n", items.join(", "));
            }
            "auto_import_installed" => out += &format!("{k} = {}\n", s.auto_import_installed),
            "inject_ampr" => out += &format!("{k} = {}\n", s.inject_ampr),
            "add_emulators" => out += &format!("{k} = {}\n", s.add_emulators),
            "playgo_off" => {
                let items: Vec<String> = s.playgo_off.iter().map(|t| toml_quote(t)).collect();
                out += &format!("playgo_off = [{}]\n", items.join(", "));
            }
            "game_fakelib" => out += &format!("{k} = {}\n", toml_quote(fakelib_str(s.game_fakelib))),
            "backports" => out += &format!("{k} = {}\n", s.backports),
            "scan_interval_seconds" => out += &format!("{k} = {}\n", s.scan_interval_secs),
            "stability_wait_seconds" => out += &format!("{k} = {}\n", s.stability_wait_secs),
            "low_space_gb" => out += &format!("{k} = {}\n", s.low_space_gb),
            "notify" => out += &format!("{k} = {}\n", s.notify),
            "log" => out += &format!("{k} = {}\n", s.log),
            _ => out += &format!("{k} = {}\n", s.debug),
        }
    }
    out
}

/// Reads the config; creates it with the defaults when missing.
pub fn load() -> Settings {
    match std::fs::read_to_string(CONFIG_PATH) {
        Ok(t) => parse(&t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let s = Settings::default();
            if let Err(e) = save(&s) {
                log!("config {CONFIG_PATH}: cannot create: {e}");
            }
            s
        }
        Err(e) => {
            log!("config {CONFIG_PATH}: cannot read: {e}; using the defaults");
            Settings::default()
        }
    }
}

/// Writes the config atomically (tmp + rename).
pub fn save(s: &Settings) -> std::io::Result<()> {
    let path = std::path::Path::new(CONFIG_PATH);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = format!("{CONFIG_PATH}.tmp");
    std::fs::write(&tmp, render(s))?;
    std::fs::rename(&tmp, path)
}

pub fn to_json(s: &Settings) -> Value {
    json!({
        "scanpath": s.scan_paths,
        "inject_ampr": s.inject_ampr,
        "add_emulators": s.add_emulators,
        "playgo_off": s.playgo_off,
        "auto_import_installed": s.auto_import_installed,
        "game_fakelib": fakelib_str(s.game_fakelib),
        "backports": s.backports,
        "scan_interval_seconds": s.scan_interval_secs,
        "stability_wait_seconds": s.stability_wait_secs,
        "low_space_gb": s.low_space_gb,
        "notify": s.notify,
        "log": s.log,
        "debug": s.debug,
    })
}

/// Missing keys take the default; bools may be true/false or 0/1.
pub fn from_json(v: &Value) -> Result<Settings, String> {
    let obj = v.as_object().ok_or("settings: expected a JSON object")?;
    let mut s = Settings::default();
    for (k, val) in obj {
        let vals: Vec<&Value> = match (k.as_str(), val) {
            ("scanpath" | "playgo_off", Value::Array(a)) => a.iter().collect(),
            ("scanpath" | "playgo_off", _) => return Err(format!("{k}: expected an array of strings")),
            _ => vec![val],
        };
        for x in vals {
            let text = match x {
                Value::String(t) => t.trim().to_string(),
                Value::Bool(b) => (if *b { "1" } else { "0" }).to_string(),
                Value::Number(n) => n.to_string(),
                _ => return Err(format!("{k}: unsupported value {x}")),
            };
            set(&mut s, k, &text).map_err(|e| format!("{k}: {e}"))?;
        }
    }
    Ok(s)
}

#[cfg(test)]mod import_tests{use super::*;
#[test]fn legacy_settings_default_to_authorized_auto_import(){let s=from_json(&json!({"inject_ampr":true})).unwrap();assert!(s.auto_import_installed);}
#[test]fn import_can_be_disabled_and_survives_config_roundtrip(){let s=from_json(&json!({"auto_import_installed":false})).unwrap();assert!(!s.auto_import_installed);assert_eq!(parse(&render(&s)),s);assert_eq!(to_json(&s)["auto_import_installed"],false);}}

#[cfg(test)]
mod kraken031_tests {
    use super::*;
    #[test]
    fn playgo_exclusions_roundtrip() {
        let s = from_json(&json!({"playgo_off":["PPSA01556","PPSA01556"]})).unwrap();
        assert_eq!(s.playgo_off, vec!["PPSA01556"]);
        assert_eq!(parse(&render(&s)).playgo_off, s.playgo_off);
        assert_eq!(to_json(&s)["playgo_off"], json!(["PPSA01556"]));
    }
    #[test]
    fn malformed_array_item_does_not_erase_valid_titles() {
        let s = parse("playgo_off = [\"PPSA01556\", \"not-valid\", \"PPSA12345\"]");
        assert_eq!(s.playgo_off, vec!["PPSA01556", "PPSA12345"]);
    }
}
