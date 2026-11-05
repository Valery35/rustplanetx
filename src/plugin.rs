// RustPlanetX - расширение RustGIS для PlanetX Lite.
// Copyright (C) 2026 ООО «Информ++». Лицензия Apache 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Команды расширения и вызовы хоста RustGIS.

use rustgis_plugin::{Panel, Request, Value, host, json};

use crate::catalog;
use crate::days::{self, Day};
use crate::{Pick, Tr};

fn handle(req: Request) -> Result<Value, String> {
    let tr = Tr::new(&req.lang);
    match req.id.as_str() {
        "panel-state" => panel_state(&tr),
        "open-in-lite" => open_in_lite(&tr),
        "add-source" => add_source(&req, &tr),
        other => Err(tr.fill("RustPlanetX has no command '{id}'.", &[("id", other)])),
    }
}

rustgis_plugin::export!(handle);

/// Ответ `host.fetch`.
enum Fetched {
    /// Документ получен: текст и разобранный JSON, если это JSON.
    Ready(String, Value),
    /// Веб-версия RustGIS качает документ и повторит вызов, когда он
    /// придёт. Команда сразу возвращается.
    Pending,
}

fn fetch(url: &str) -> Result<Fetched, String> {
    let v = host::call("host.fetch", json!({"url": url}))?;
    if v["pending"] == true {
        return Ok(Fetched::Pending);
    }
    Ok(Fetched::Ready(v["text"].as_str().unwrap_or("").to_string(), v["json"].clone()))
}

/// Последний день ряда облаков GIBS. Ok(None) - документ ещё качается.
fn reference_day() -> Result<Option<Day>, String> {
    let clouds = catalog::theme(catalog::REFERENCE).ok_or("no reference layer")?;
    match fetch(&catalog::domains_url(clouds))? {
        Fetched::Pending => Ok(None),
        Fetched::Ready(text, _) => days::last(&days::parse_domains(&text)).map(Some).ok_or_else(|| "GIBS sent no dates".into()),
    }
}

/// Ссылка на Lite по виду карты.
fn view_link(tr: &Tr) -> Result<String, String> {
    crate::lite::link_of_view(&host::view()?).ok_or_else(|| tr.t("The map's centre is not a place on the Earth."))
}

fn open_in_lite(tr: &Tr) -> Result<Value, String> {
    let link = view_link(tr)?;
    host::notify(&tr.fill("PlanetX Lite: {link}", &[("link", &link)]));
    host::log(&link);
    Ok(Panel::new().text("link", link.clone()).answer(json!({"url": link})))
}

fn panel_state(tr: &Tr) -> Result<Value, String> {
    // Начальный текст виджета RustGIS не переводит, он ставится здесь.
    let mut panel = Panel::new().text("about", tr.t(crate::ABOUT)).text("link", view_link(tr).unwrap_or_else(|e| e));
    // Ползунок дня GIBS подписан датами ряда облаков.
    match reference_day() {
        Ok(Some(last)) => panel = panel.slider("day", 0, catalog::DAYS_BACK, &catalog::slider_labels(last)),
        Ok(None) => return Ok(Value::Null),
        Err(e) => host::log(&format!("GIBS: {e}")),
    }
    let releases = match fetch(catalog::WAYBACK_CONFIG) {
        Ok(Fetched::Ready(_, config)) => catalog::parse_wayback(&config),
        Ok(Fetched::Pending) => return Ok(Value::Null),
        Err(e) => {
            host::log(&format!("Wayback: {e}"));
            Vec::new()
        }
    };
    let mut answer = panel.answer(Value::Null);
    answer["panel"]["items"]["catalog"] = json!({"nodes": crate::tree(tr, &releases)});
    Ok(answer)
}

fn add_source(req: &Request, tr: &Tr) -> Result<Value, String> {
    // Источник из параметров (скрипт, командная строка) или из дерева.
    let id = req.params["source"].as_str().or_else(|| req.panel_str("catalog")).ok_or_else(|| tr.t("Choose a source in the catalogue."))?;
    let unknown = || tr.fill("The catalogue has no source '{id}'.", &[("id", id)]);
    let (params, name) = match crate::pick(tr, id).ok_or_else(unknown)? {
        // Узел группы только раскрывает список.
        Pick::Group => return Ok(Value::Null),
        Pick::Tiles(params, name) => (params, name),
        Pick::Gibs(t) => {
            let Some(day) = gibs_day(req, t)? else {
                return Ok(Value::Null);
            };
            // У слоя без ряда дат день в имени не пишется.
            let name = if t.5 { format!("{} {day}", tr.t(t.1)) } else { tr.t(t.1) };
            (catalog::gibs_tiles(t, &name, &day), name)
        }
    };
    host::run("map.addTiles", params)?;
    host::notify(&tr.fill("Added {name}.", &[("name", &name)]));
    Ok(json!({"added": name}))
}

/// День темы t по ползунку панели: ряд облаков даёт отсчёт, ряд темы -
/// её день не позже. Ok(None) - документ ещё качается. Без отсчёта -
/// default, последний день GIBS.
fn gibs_day(req: &Request, t: &catalog::Theme) -> Result<Option<String>, String> {
    if !t.5 {
        return Ok(Some("default".into()));
    }
    let last = match reference_day() {
        Ok(Some(last)) => last,
        Ok(None) => return Ok(None),
        Err(e) => {
            host::log(&format!("GIBS: {e}"));
            return Ok(Some("default".into()));
        }
    };
    let want = catalog::slider_value_day(last, req.panel_i64("day"));
    if t.0 == catalog::REFERENCE {
        return Ok(Some(want.text()));
    }
    match fetch(&catalog::domains_url(t)) {
        Ok(Fetched::Ready(text, _)) => Ok(Some(catalog::theme_day(Some(&text), want))),
        Ok(Fetched::Pending) => Ok(None),
        Err(e) => {
            host::log(&format!("GIBS {}: {e}", t.2));
            Ok(Some("default".into()))
        }
    }
}
