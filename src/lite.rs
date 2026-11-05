// RustPlanetX - расширение RustGIS для PlanetX Lite.
// Copyright (C) 2026 ООО «Информ++». Лицензия Apache 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Ссылка на страницу PlanetX Lite с позой камеры.
//!
//! Формат - `crates/ui/src/share.rs` PlanetX Lite:
//! `?pose=широта,долгота,расстояние,азимут,наклон`. Расстояние камеры
//! берётся из масштаба карты так, чтобы метр на пиксель экрана в Lite
//! был тот же, что на карте RustGIS.

/// Страница Lite на GitHub Pages.
pub const PAGE: &str = "https://valery35.github.io/planetx-lite/";

/// Метров на логический пиксель в знаменателе масштаба RustGIS
/// (`rustgis_geom::PIXEL_M`, 0.28 мм OGC).
const PIXEL_M: f64 = 0.00028;
/// Угол обзора камеры Lite по вертикали, градусы (`globe_core::camera`).
const FOV_Y: f64 = 45.0;
/// Высота вида в логических пикселях. Размер окна расширению не виден,
/// берётся обычная высота карты на экране компьютера.
const VIEW_PX: f64 = 900.0;
/// Ближе и дальше этого камера Lite не ставится, метры.
const NEAREST: f64 = 50.0;
const FARTHEST: f64 = 4.0e7;

/// Расстояние камеры Lite для знаменателя масштаба карты scale.
/// Неверный масштаб - None.
pub fn distance(scale: f64) -> Option<f64> {
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let ground = scale * PIXEL_M * VIEW_PX;
    Some((ground / (2.0 * (FOV_Y.to_radians() / 2.0).tan())).clamp(NEAREST, FARTHEST))
}

/// Число с digits знаками после запятой без нулей в конце, как
/// `format_pose` Lite.
fn number(x: f64, digits: usize) -> String {
    let s = format!("{x:.digits$}");
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    if s == "-0" { "0".to_string() } else { s }
}

/// Ссылка на Lite: взгляд сверху на точку lat, lon с расстояния
/// distance метров, север вверху.
pub fn link(lat: f64, lon: f64, distance: f64) -> String {
    format!("{PAGE}?pose={},{},{},0,0", number(lat, 7), number(lon, 7), number(distance, 1))
}

/// Ссылка по ответу `host.view` `{center: [lon, lat], scale}`.
/// Центр вне Земли или неверный масштаб - None.
pub fn link_of_view(view: &serde_json::Value) -> Option<String> {
    let lon = view["center"][0].as_f64()?;
    let lat = view["center"][1].as_f64()?;
    if !(lat.is_finite() && lon.is_finite()) || lat.abs() > 90.0 {
        return None;
    }
    // Долгота к -180..180: карта Web Mercator может уйти за край мира.
    let lon = (lon + 180.0).rem_euclid(360.0) - 180.0;
    Some(link(lat, lon, distance(view["scale"].as_f64()?)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn link_matches_lite_format() {
        assert_eq!(link(10.0, -20.5, 1000.0), "https://valery35.github.io/planetx-lite/?pose=10,-20.5,1000,0,0");
        assert_eq!(link(58.01049999, 56.2294, 12_345.678), "https://valery35.github.io/planetx-lite/?pose=58.0105,56.2294,12345.7,0,0");
        assert_eq!(link(-0.0, 0.0, 100.0), "https://valery35.github.io/planetx-lite/?pose=0,0,100,0,0");
    }

    #[test]
    fn distance_keeps_ground_per_pixel() {
        // Метр на пиксель в Lite - 2·d·tan(fov/2) / высота вида.
        for scale in [1_000.0, 25_000.0, 1.0e6] {
            let d = distance(scale).unwrap_or(0.0);
            let lite = 2.0 * d * (FOV_Y.to_radians() / 2.0).tan() / VIEW_PX;
            assert!((lite / (scale * PIXEL_M) - 1.0).abs() < 1e-9, "{scale}");
        }
        assert_eq!(distance(1.0), Some(NEAREST));
        assert_eq!(distance(1.0e12), Some(FARTHEST));
        assert_eq!(distance(0.0), None);
        assert_eq!(distance(f64::NAN), None);
    }

    #[test]
    fn view_to_link() {
        let view = json!({"center": [56.2294, 58.0105], "scale": 25_000.0, "projection": "EPSG:3857"});
        assert!(link_of_view(&view).is_some_and(|l| l.starts_with("https://valery35.github.io/planetx-lite/?pose=58.0105,56.2294,")));
        let wrapped = json!({"center": [376.0, 10.0], "scale": 25_000.0});
        assert!(link_of_view(&wrapped).is_some_and(|l| l.contains("pose=10,16,")));
        assert!(link_of_view(&json!({"center": [0.0, 95.0], "scale": 1000.0})).is_none());
        assert!(link_of_view(&json!({})).is_none());
    }
}
