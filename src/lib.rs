// RustPlanetX - расширение RustGIS для PlanetX Lite.
// Copyright (C) 2026 ООО «Информ++». Лицензия Apache 2.0.
// SPDX-License-Identifier: Apache-2.0

//! RustPlanetX - мост из RustGIS в PlanetX Lite.
//!
//! Две части, решение автора от 5 ноября 2026 года:
//! - `open-in-lite` - ссылка на страницу Lite с видом карты. Открыть
//!   браузер расширение не может: команды окна RustGIS (`ui.*`) песочнице
//!   не даны. Ссылка встаёт в панель и в строку состояния, откуда её
//!   копируют.
//! - каталог источников Lite деревом панели, `add-source` ставит
//!   выбранный источник слоем тайлов (`map.addTiles`).
//!
//! Состояния между вызовами нет: день GIBS и выбранный узел каталога
//! RustGIS хранит в значениях панели и шлёт с каждым запросом.
//!
//! Вызовы хоста - в модуле `plugin`, он и SDK собираются только под
//! wasm32. Импорты хоста на Windows не линкуются, а остальное
//! проверяется `cargo test` на своей машине.

pub mod catalog;
pub mod days;
pub mod lite;
#[cfg(target_arch = "wasm32")]
mod plugin;

use serde_json::{Value, json};

use catalog::{BODIES, READY, THEMES};

/// `plugin.json`, ради переводов.
pub const MANIFEST: &str = include_str!("../plugin.json");

/// Текст виджета about, как в `plugin.json`.
pub const ABOUT: &str = "A 3D globe in the browser: site analysis, multimedia tours, KML and .planetx scenes. Copy the link into the browser.";

/// Переводы `plugin.json` на язык lang. Манифест разбирается один раз
/// на вызов.
pub struct Tr {
    table: Value,
}

impl Tr {
    pub fn new(lang: &str) -> Tr {
        let manifest: Value = serde_json::from_str(MANIFEST).unwrap_or(Value::Null);
        let i18n = &manifest["i18n"];
        // Код с областью (pt-BR) - язык без области, если своей таблицы нет.
        let base = lang.split(['-', '_']).next().unwrap_or("");
        let table = if i18n[lang].is_object() { i18n[lang].clone() } else { i18n[base].clone() };
        Tr { table }
    }

    pub fn t(&self, s: &str) -> String {
        self.table[s].as_str().unwrap_or(s).to_string()
    }

    pub fn fill(&self, s: &str, args: &[(&str, &str)]) -> String {
        args.iter().fold(self.t(s), |out, (k, v)| out.replace(&format!("{{{k}}}"), v))
    }
}

/// Узел дерева панели `{id, label}`.
fn node(id: String, label: impl Into<String>) -> Value {
    json!({"id": id, "label": label.into()})
}

/// Узел группы с детьми и их количеством справа.
fn group(id: &str, label: String, children: Vec<Value>) -> Value {
    json!({"id": id, "label": label, "count": children.len(), "children": children})
}

/// Дерево каталога - узлы панели `nodes`: подосновы, GIBS на дату,
/// Wayback, тела.
pub fn tree(tr: &Tr, releases: &[catalog::Release]) -> Value {
    let ready = READY.iter().map(|r| node(format!("ready:{}", r.0), r.1)).collect();
    let gibs = THEMES.iter().map(|t| node(format!("gibs:{}", t.0), tr.t(t.1))).collect();
    let wayback = releases.iter().map(|r| node(format!("wayback:{}:{}", r.number, r.day), r.day.clone())).collect();
    let bodies = BODIES.iter().map(|b| node(format!("body:{}", b.0), body_name(tr, b))).collect();
    json!([
        group("group:ready", tr.t("Ready basemaps"), ready),
        group("group:gibs", tr.t("NASA GIBS on the day"), gibs),
        group("group:wayback", tr.t("Esri Wayback releases"), wayback),
        group("group:bodies", tr.t("Imagery of bodies"), bodies),
    ])
}

/// Название снимков тела: тело и мозаика.
pub fn body_name(tr: &Tr, b: &catalog::Body) -> String {
    format!("{} - {}", tr.t(catalog::body_title(b.0)), b.1)
}

/// Что поставить по id узла каталога.
#[derive(Debug, Clone, PartialEq)]
pub enum Pick {
    /// Узел группы, слоя нет.
    Group,
    /// Готовые параметры `map.addTiles` и имя слоя.
    Tiles(Value, String),
    /// Тема GIBS, её день ещё выбирается.
    Gibs(&'static catalog::Theme),
}

/// Разбор id узла каталога. Неизвестный id - None.
pub fn pick(tr: &Tr, id: &str) -> Option<Pick> {
    let (kind, key) = id.split_once(':').unwrap_or((id, ""));
    match kind {
        "group" => Some(Pick::Group),
        "ready" => READY.iter().find(|r| r.0 == key).map(|r| Pick::Tiles(catalog::ready_tiles(r), r.1.to_string())),
        "body" => BODIES.iter().find(|b| b.0 == key).map(|b| {
            let name = body_name(tr, b);
            Pick::Tiles(catalog::body_tiles(b, &name), name)
        }),
        "gibs" => catalog::theme(key).map(Pick::Gibs),
        "wayback" => {
            let (number, day) = key.split_once(':').unwrap_or((key, ""));
            let number: u64 = number.parse().ok()?;
            let name = format!("Esri Wayback {day}").trim_end().to_string();
            Some(Pick::Tiles(catalog::wayback_tiles(number, &name), name))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_is_valid() {
        let m: Value = serde_json::from_str(MANIFEST).unwrap_or(Value::Null);
        assert_eq!(m["id"], "rustplanetx");
        let commands: Vec<&str> = m["commands"].as_array().map(|a| a.iter().filter_map(|c| c["id"].as_str()).collect()).unwrap_or_default();
        for id in ["panel-state", "open-in-lite", "add-source"] {
            assert!(commands.contains(&id), "{id}");
        }
        // Версия манифеста - версия крейта.
        assert_eq!(m["version"], env!("CARGO_PKG_VERSION"));
        let about = m["panel"]["items"].as_array().and_then(|a| a.iter().find(|i| i["id"] == "about")).map(|i| i["text"].clone());
        assert_eq!(about, Some(json!(ABOUT)));
        assert!(m["i18n"]["ru"][ABOUT].is_string());
    }

    #[test]
    fn every_label_has_russian() {
        let ru = Tr::new("ru");
        let mut labels: Vec<&str> = THEMES.iter().map(|t| t.1).chain(BODIES.iter().map(|b| catalog::body_title(b.0))).collect();
        labels.extend([
            "Ready basemaps",
            "NASA GIBS on the day",
            "Esri Wayback releases",
            "Imagery of bodies",
            "Choose a source in the catalogue.",
            "The catalogue has no source '{id}'.",
            "Added {name}.",
            "PlanetX Lite: {link}",
            "The map's centre is not a place on the Earth.",
            "RustPlanetX has no command '{id}'.",
        ]);
        for s in labels {
            assert!(ru.table[s].is_string(), "{s}");
        }
        assert_eq!(ru.fill("Added {name}.", &[("name", "OpenTopoMap")]), "Добавлен слой OpenTopoMap.");
        assert_eq!(Tr::new("ru-RU").t("Mars"), "Марс");
        assert_eq!(Tr::new("en").t("Mars"), "Mars");
    }

    #[test]
    fn tree_ids_are_unique_and_picked() {
        let tr = Tr::new("en");
        let releases = vec![catalog::Release { number: 10, day: "2014-02-20".into() }];
        let mut ids = Vec::new();
        fn walk(n: &Value, ids: &mut Vec<String>) {
            ids.push(n["id"].as_str().unwrap_or("").to_string());
            n["children"].as_array().into_iter().flatten().for_each(|c| walk(c, ids));
        }
        tree(&tr, &releases).as_array().into_iter().flatten().for_each(|n| walk(n, &mut ids));
        // Узлов не больше 5000 - предел дерева RustGIS.
        assert!(ids.len() < 5000);
        let n = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), n);
        // Каждый узел дерева разбирается.
        for id in &ids {
            assert!(pick(&tr, id).is_some(), "{id}");
        }
    }

    #[test]
    fn picks() {
        let tr = Tr::new("ru");
        let Some(Pick::Tiles(p, name)) = pick(&tr, "body:mars") else {
            panic!("body:mars");
        };
        assert_eq!(name, "Марс - Mars Viking MDIM2.1");
        assert_eq!(p["maxZoom"], 7);
        let Some(Pick::Tiles(p, name)) = pick(&tr, "wayback:10:2014-02-20") else {
            panic!("wayback");
        };
        assert_eq!(name, "Esri Wayback 2014-02-20");
        assert!(p["url"].as_str().is_some_and(|u| u.contains("/tile/10/")));
        assert!(matches!(pick(&tr, "gibs:smoke"), Some(Pick::Gibs(_))));
        for bad in ["", "ready:", "ready:nope", "wayback:x:1", "zzz:1"] {
            assert_eq!(pick(&tr, bad), None, "{bad}");
        }
    }
}
