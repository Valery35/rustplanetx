// RustPlanetX - расширение RustGIS для PlanetX Lite.
// Copyright (C) 2026 ООО «Информ++». Лицензия Apache 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Каталог источников PlanetX Lite слоями тайлов RustGIS.
//!
//! Адреса взяты из PlanetX Lite: готовые подосновы - `READY`
//! в `crates/ui/src/basemap.rs`, снимки тел - `crates/scene/src/planets.rs`,
//! темы GIBS - `THEMES` в `crates/data/src/gibs.rs`, облака и ночные огни -
//! `crates/data/src/clouds.rs` и `crates/ui/src/app.rs`, выпуски Wayback -
//! `crates/data/src/wayback.rs`.
//!
//! Строка каталога - узел дерева панели. Его id - вид и ключ через
//! двоеточие: `ready:esri_topo`, `body:mars`, `gibs:smoke`,
//! `wayback:10`. Узлы групп - `group:…`, слоя у них нет.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::days::{self, Day};

/// Готовая подоснова: ключ, название, адрес, предельный уровень, подпись.
pub type Ready = (&'static str, &'static str, &'static str, u32, &'static str);

const USGS_CREDIT: &str = "USGS The National Map";

macro_rules! esri {
    ($path:literal) => {
        concat!("https://server.arcgisonline.com/ArcGIS/rest/services/", $path, "/MapServer/tile/{z}/{y}/{x}")
    };
}

macro_rules! usgs {
    ($path:literal) => {
        concat!("https://basemap.nationalmap.gov/arcgis/rest/services/", $path, "/MapServer/tile/{z}/{y}/{x}")
    };
}

/// Готовые подосновы Lite в порядке его меню.
pub const READY: [Ready; 19] = [
    ("opentopomap", "OpenTopoMap", "https://a.tile.opentopomap.org/{z}/{x}/{y}.png", 17, "© OpenStreetMap contributors, SRTM, © OpenTopoMap (CC-BY-SA)"),
    ("osm_hot", "OSM Humanitarian", "https://a.tile.openstreetmap.fr/hot/{z}/{x}/{y}.png", 19, "© OpenStreetMap contributors, HOT, OpenStreetMap France"),
    ("osm_fr", "OSM France", "https://a.tile.openstreetmap.fr/osmfr/{z}/{x}/{y}.png", 19, "© OpenStreetMap France, © OpenStreetMap contributors"),
    ("cyclosm", "CyclOSM", "https://a.tile-cyclosm.openstreetmap.fr/cyclosm/{z}/{x}/{y}.png", 19, "CyclOSM, © OpenStreetMap contributors"),
    ("esri_topo", "Esri World Topo Map", esri!("World_Topo_Map"), 19, "Esri and the GIS User Community"),
    ("esri_street", "Esri World Street Map", esri!("World_Street_Map"), 19, "Esri and its data partners"),
    ("esri_light_gray", "Esri Light Gray Canvas", esri!("Canvas/World_Light_Gray_Base"), 16, "Esri, HERE, Garmin, © OpenStreetMap contributors"),
    ("esri_dark_gray", "Esri Dark Gray Canvas", esri!("Canvas/World_Dark_Gray_Base"), 16, "Esri, HERE, Garmin, © OpenStreetMap contributors"),
    ("esri_natgeo", "Esri National Geographic", esri!("NatGeo_World_Map"), 16, "National Geographic, Esri, GEBCO, NOAA, iPC"),
    ("esri_ocean", "Esri Ocean", esri!("Ocean/World_Ocean_Base"), 13, "Esri, GEBCO, NOAA, National Geographic"),
    ("esri_terrain", "Esri World Terrain", esri!("World_Terrain_Base"), 13, "Esri, USGS, NPS"),
    ("esri_shaded", "Esri Shaded Relief", esri!("World_Shaded_Relief"), 13, "Esri"),
    ("esri_physical", "Esri Physical", esri!("World_Physical_Map"), 8, "Esri, US National Park Service"),
    ("esri_hillshade", "Esri World Hillshade", esri!("Elevation/World_Hillshade"), 16, "Esri, Airbus DS, USGS, NGA, NASA, CGIAR"),
    (
        "nasa_blue_marble",
        "NASA Blue Marble",
        "https://gibs.earthdata.nasa.gov/wmts/epsg3857/best/BlueMarble_ShadedRelief_Bathymetry/default/GoogleMapsCompatible_Level8/{z}/{y}/{x}.jpeg",
        8,
        "NASA Blue Marble, NASA GIBS",
    ),
    ("usgs_imagery", "USGS Imagery (USA)", usgs!("USGSImageryOnly"), 16, USGS_CREDIT),
    ("usgs_imagery_topo", "USGS Imagery Topo (USA)", usgs!("USGSImageryTopo"), 16, USGS_CREDIT),
    ("usgs_topo", "USGS Topo (USA)", usgs!("USGSTopo"), 16, USGS_CREDIT),
    ("usgs_relief", "USGS Shaded Relief (USA)", usgs!("USGSShadedReliefOnly"), 16, USGS_CREDIT),
];

/// Снимки тела: ключ, название, адрес, предельный уровень, подпись.
pub type Body = (&'static str, &'static str, &'static str, u32, &'static str);

macro_rules! body {
    ($key:literal, $name:literal, $level:literal, $credit:literal) => {
        ($key, $name, concat!("https://raw.githubusercontent.com/Valery35/planetx-terrain/main/imagery/", $key, "/{z}/{x}/{y}.jpg"), $level, $credit)
    };
}

/// Тела в порядке меню «Тело» Lite.
pub const BODIES: [Body; 18] = [
    (
        "moon",
        "Moon LRO LROC WAC 100 m",
        "https://raw.githubusercontent.com/Valery35/planetx-bodies-imagery/main/moon/{z}/{x}/{y}.png",
        7,
        "NASA, GSFC, Arizona State University, LRO LROC WAC, USGS",
    ),
    (
        "mars",
        "Mars Viking MDIM2.1",
        "https://raw.githubusercontent.com/Valery35/planetx-bodies-imagery/main/mars/{z}/{x}/{y}.png",
        7,
        "NASA, USGS, Viking MDIM2.1, OpenPlanetaryMap",
    ),
    body!("mercury", "MESSENGER MDIS MD3 color", 6, "NASA, JHU APL, Carnegie Institution, MESSENGER MDIS, USGS"),
    body!("venus", "Magellan C3-MDIR colorized topography", 5, "NASA, JPL, Magellan, USGS"),
    body!("jupiter", "Cassini PIA07782", 4, "NASA, JPL, Space Science Institute, Cassini"),
    body!("io", "Galileo SSI and Voyager color merge", 5, "NASA, JPL, Galileo SSI, Voyager, USGS"),
    body!("europa", "Voyager and Galileo SSI", 6, "NASA, JPL, Galileo SSI, Voyager, USGS"),
    body!("ganymede", "Voyager and Galileo SSI color", 5, "NASA, JPL, Galileo SSI, Voyager, USGS"),
    body!("callisto", "Voyager and Galileo SSI", 6, "NASA, JPL, Galileo SSI, Voyager, USGS"),
    body!("mimas", "Cassini PIA17214", 5, "NASA, JPL-Caltech, Space Science Institute, Cassini"),
    body!("enceladus", "Cassini ISS", 5, "NASA, JPL, Space Science Institute, Cassini ISS, USGS"),
    body!("tethys", "Cassini ISS", 5, "NASA, JPL, Space Science Institute, Cassini ISS, USGS"),
    body!("dione", "Cassini ISS and Voyager", 6, "NASA, JPL, Space Science Institute, Cassini ISS, USGS"),
    body!("rhea", "Cassini ISS and Voyager", 5, "NASA, JPL, Space Science Institute, DLR, Cassini ISS, USGS"),
    body!("titan", "Cassini ISS", 4, "NASA, JPL-Caltech, Space Science Institute, Cassini ISS, USGS"),
    body!("iapetus", "Cassini ISS and Voyager", 5, "NASA, JPL, Space Science Institute, Cassini ISS, USGS"),
    body!("ceres", "Dawn FC", 5, "NASA, JPL-Caltech, UCLA, MPS, DLR, IDA, Dawn FC, USGS"),
    body!("vesta", "Dawn FC HAMO", 6, "NASA, JPL-Caltech, UCLA, MPS, DLR, IDA, Dawn FC, USGS"),
];

/// Название тела по ключу, английское - ключ перевода.
pub fn body_title(key: &str) -> &'static str {
    match key {
        "moon" => "Moon",
        "mars" => "Mars",
        "mercury" => "Mercury",
        "venus" => "Venus",
        "jupiter" => "Jupiter",
        "io" => "Io",
        "europa" => "Europa",
        "ganymede" => "Ganymede",
        "callisto" => "Callisto",
        "mimas" => "Mimas",
        "enceladus" => "Enceladus",
        "tethys" => "Tethys",
        "dione" => "Dione",
        "rhea" => "Rhea",
        "titan" => "Titan",
        "iapetus" => "Iapetus",
        "ceres" => "Ceres",
        _ => "Vesta",
    }
}

/// Слой GIBS: ключ, название (ключ перевода), слой, расширение тайла,
/// предельный уровень, есть ли у слоя ряд дат.
pub type Theme = (&'static str, &'static str, &'static str, &'static str, u32, bool);

/// Темы GIBS Lite и два его слоя на дату: облака дня и ночные огни 2016.
pub const THEMES: [Theme; 24] = [
    ("clouds", "Clouds of the day (VIIRS)", "VIIRS_SNPP_CorrectedReflectance_TrueColor", "jpg", 9, true),
    ("smoke", "Smoke (aerosol index)", "OMPS_Aerosol_Index", "png", 6, true),
    ("aerosol", "Aerosol optical depth", "MODIS_Combined_Value_Added_AOD", "png", 6, true),
    ("co", "Carbon monoxide", "AIRS_L2_Carbon_Monoxide_500hPa_Volume_Mixing_Ratio_Day", "png", 6, true),
    ("co_emission", "Carbon monoxide emission, month", "MERRA2_Carbon_Monoxide_Emission_Monthly", "png", 6, true),
    ("rain", "Precipitation rate", "IMERG_Precipitation_Rate", "png", 6, true),
    ("soil", "Soil moisture", "SMAP_L3_Passive_Enhanced_Day_Soil_Moisture", "png", 6, true),
    ("snow", "Snow cover", "MODIS_Terra_NDSI_Snow_Cover", "png", 8, true),
    ("snow_8day", "Snow extent, 8 days", "MODIS_Terra_L3_Snow_Extent_8Day", "png", 8, true),
    ("snow_mass", "Snow mass", "SMAP_L4_Snow_Mass", "png", 6, true),
    ("sea_ice", "Sea ice concentration", "GHRSST_L4_MUR_Sea_Ice_Concentration", "png", 7, true),
    ("vapor", "Water vapor", "MODIS_Terra_Water_Vapor_5km_Day", "png", 6, true),
    ("chlorophyll", "Chlorophyll a", "OCI_PACE_Chlorophyll_a", "png", 7, true),
    ("salinity", "Sea surface salinity", "SMAP_L3_Sea_Surface_Salinity_CAP_8Day_RunningMean", "png", 6, true),
    ("flood", "Floods, 3 days", "MODIS_Combined_Flood_3-Day", "png", 9, true),
    ("no2", "Nitrogen dioxide", "TROPOMI_L2_Nitrogen_Dioxide_Tropospheric_Column", "png", 6, true),
    ("so2", "Sulfur dioxide", "OMPS_SO2_Planetary_Boundary_Layer", "png", 6, true),
    ("methane", "Methane, month", "AIRS_L3_Methane_400hPa_Volume_Mixing_Ratio_Monthly_Day", "png", 6, true),
    ("co2", "Carbon dioxide", "OCO-2_Carbon_Dioxide_Total_Column_Average", "png", 8, true),
    ("ozone", "Ozone", "OMI_Ozone_TOMS_Total_Column", "png", 6, true),
    ("ndvi", "Vegetation index NDVI, 16 days", "MODIS_Terra_L3_NDVI_16Day", "png", 9, true),
    ("dust", "Dust", "AIRS_L2_Dust_Score_Day", "png", 6, true),
    ("night", "Night (VIIRS day-night band)", "VIIRS_SNPP_DayNightBand_AtSensor_M15", "jpeg", 8, true),
    ("lights", "Night lights 2016 (Black Marble)", "VIIRS_Night_Lights", "png", 8, false),
];

const GIBS_ROOT: &str = "https://gibs.earthdata.nasa.gov";
pub const GIBS_CREDIT: &str = "NASA GIBS";
/// Непрозрачность темы поверх снимка, как в Lite.
const GIBS_OPACITY: f64 = 0.8;

/// Ряд дат, по которому подписан ползунок дня: облака дня.
pub const REFERENCE: &str = "clouds";

/// Тема по ключу.
pub fn theme(key: &str) -> Option<&'static Theme> {
    THEMES.iter().find(|t| t.0 == key)
}

/// Адрес ряда дат DescribeDomains темы.
pub fn domains_url(t: &Theme) -> String {
    format!("{GIBS_ROOT}/wmts/epsg3857/best/1.0.0/{}/default/GoogleMapsCompatible_Level{}/all/1900-01-01--2100-01-01.xml", t.2, t.4)
}

/// Шаблон адреса тайла темы на день day (ГГГГ-ММ-ДД или default).
pub fn gibs_url(t: &Theme, day: &str) -> String {
    // Ночные огни 2016 - один снимок на 2016-01-01, как в Lite.
    let day = if t.5 { day } else { "2016-01-01" };
    format!("{GIBS_ROOT}/wmts/epsg3857/best/{}/default/{day}/GoogleMapsCompatible_Level{}/{{z}}/{{y}}/{{x}}.{}", t.2, t.4, t.3)
}

/// День ползунка: last - день ряда облаков, back - дней назад. Последний
/// объявленный день ряда может быть ещё не обработан, отсчёт - от
/// предыдущего, как READY_LAG в Lite.
pub fn slider_day(last: Day, back: i64) -> Day {
    last.add(-1 - back.max(0), 'D')
}

/// Наибольшее количество дней назад на ползунке.
pub const DAYS_BACK: i64 = 365;

/// День значения ползунка value: слева старый день, справа - последний
/// готовый, как шкала времени. Без значения - последний.
pub fn slider_value_day(last: Day, value: Option<i64>) -> Day {
    slider_day(last, DAYS_BACK - value.unwrap_or(DAYS_BACK).clamp(0, DAYS_BACK))
}

/// Подписи ползунка дня от last, по значениям 0..=DAYS_BACK.
pub fn slider_labels(last: Day) -> Vec<String> {
    (0..=DAYS_BACK).map(|v| slider_value_day(last, Some(v)).text()).collect()
}

/// Корень Wayback и файл выпусков.
pub const WAYBACK_CONFIG: &str = "https://s3-us-west-2.amazonaws.com/config.maptiles.arcgis.com/waybackconfig.json";
const WAYBACK_ROOT: &str = "https://wayback.maptiles.arcgis.com/arcgis/rest/services/World_Imagery";
pub const WAYBACK_CREDIT: &str = "Wayback - Esri, Vantor, Earthstar Geographics, and the GIS User Community";

/// Выпуск Wayback: номер и дата ГГГГ-ММ-ДД.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub number: u64,
    pub day: String,
}

/// Дата ГГГГ-ММ-ДД из строки или None.
fn day_in(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let digits = |r: std::ops::Range<usize>, b: &[u8]| b.get(r).is_some_and(|s| s.iter().all(u8::is_ascii_digit));
    (0..bytes.len()).find_map(|i| {
        let tail = bytes.get(i..)?;
        (digits(0..4, tail) && tail.get(4) == Some(&b'-') && digits(5..7, tail) && tail.get(7) == Some(&b'-') && digits(8..10, tail))
            .then(|| String::from_utf8_lossy(tail.get(..10).unwrap_or_default()).into_owned())
    })
}

/// Выпуски файла выпусков Wayback, новые сверху. Выпуски одного дня -
/// один, с большим номером. Перенос `parse_config` Lite.
pub fn parse_wayback(config: &Value) -> Vec<Release> {
    let Some(raw) = config.as_object() else {
        return Vec::new();
    };
    let mut by_day: BTreeMap<String, u64> = BTreeMap::new();
    for (key, item) in raw {
        let (Ok(number), Some(day)) = (key.parse::<u64>(), item["itemTitle"].as_str().and_then(day_in)) else {
            continue;
        };
        if by_day.get(&day).is_none_or(|old| number > *old) {
            by_day.insert(day, number);
        }
    }
    by_day.into_iter().rev().map(|(day, number)| Release { number, day }).collect()
}

/// Шаблон адреса тайла выпуска Wayback.
pub fn wayback_url(number: u64) -> String {
    format!("{WAYBACK_ROOT}/WMTS/1.0.0/default028mm/MapServer/tile/{number}/{{z}}/{{y}}/{{x}}")
}

/// Параметры `map.addTiles`.
fn tiles(url: &str, name: &str, credit: &str, max_zoom: u32) -> Value {
    json!({"url": url, "kind": "raster", "name": name, "attribution": credit, "maxZoom": max_zoom})
}

pub fn ready_tiles(r: &Ready) -> Value {
    tiles(r.2, r.1, r.4, r.3)
}

pub fn body_tiles(b: &Body, name: &str) -> Value {
    tiles(b.2, name, b.4, b.3)
}

pub fn gibs_tiles(t: &Theme, name: &str, day: &str) -> Value {
    let mut p = tiles(&gibs_url(t, day), name, GIBS_CREDIT, t.4);
    p["overlay"] = json!(true);
    p["opacity"] = json!(GIBS_OPACITY);
    p
}

pub fn wayback_tiles(number: u64, name: &str) -> Value {
    tiles(&wayback_url(number), name, WAYBACK_CREDIT, 19)
}

/// День темы t для дня want по её ряду дат domains (ответ
/// DescribeDomains). Ряда нет или он пуст - default, последний день GIBS.
pub fn theme_day(domains: Option<&str>, want: Day) -> String {
    let intervals = domains.map(days::parse_domains).unwrap_or_default();
    days::pick(&intervals, want).map_or_else(|| "default".to_string(), Day::text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique() {
        let mut keys: Vec<&str> = READY.iter().map(|r| r.0).chain(BODIES.iter().map(|b| b.0)).chain(THEMES.iter().map(|t| t.0)).collect();
        let n = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), n);
    }

    #[test]
    fn addresses_are_templates() {
        let urls = READY.iter().map(|r| r.2.to_string()).chain(BODIES.iter().map(|b| b.2.to_string())).chain(THEMES.iter().map(|t| gibs_url(t, "2026-11-01")));
        for url in urls {
            assert!(url.starts_with("https://") && url.contains("{z}") && url.contains("{x}") && url.contains("{y}"), "{url}");
        }
        assert!(BODIES.iter().all(|b| body_title(b.0) != "Vesta" || b.0 == "vesta"));
    }

    #[test]
    fn gibs_addresses_as_in_lite() {
        let clouds = theme("clouds").copied().unwrap_or(THEMES[0]);
        assert_eq!(
            gibs_url(&clouds, "2026-11-01"),
            "https://gibs.earthdata.nasa.gov/wmts/epsg3857/best/VIIRS_SNPP_CorrectedReflectance_TrueColor/default/2026-11-01/GoogleMapsCompatible_Level9/{z}/{y}/{x}.jpg"
        );
        let lights = theme("lights").copied().unwrap_or(THEMES[0]);
        assert!(gibs_url(&lights, "2026-11-01").contains("/VIIRS_Night_Lights/default/2016-01-01/"));
        assert!(domains_url(&clouds).ends_with("/VIIRS_SNPP_CorrectedReflectance_TrueColor/default/GoogleMapsCompatible_Level9/all/1900-01-01--2100-01-01.xml"));
    }

    #[test]
    fn slider_counts_back_from_ready_day() {
        let last = Day::parse("2026-11-02").unwrap_or(Day { year: 2026, month: 11, day: 2 });
        let labels = slider_labels(last);
        assert_eq!(labels.len() as i64, DAYS_BACK + 1);
        // Справа последний готовый день, слева - на год раньше.
        assert_eq!(labels[DAYS_BACK as usize], "2026-11-01");
        assert_eq!(labels[DAYS_BACK as usize - 1], "2026-10-31");
        assert_eq!(labels[0], "2025-11-01");
        assert_eq!(slider_value_day(last, None).text(), "2026-11-01");
        assert_eq!(slider_value_day(last, Some(9999)).text(), "2026-11-01");
        assert_eq!(slider_value_day(last, Some(-5)).text(), "2025-11-01");
        assert_eq!(theme_day(None, last), "default");
        assert_eq!(theme_day(Some("<Domain>2002-09-01/2026-09-01/P1M</Domain>"), last), "2026-09-01");
    }

    #[test]
    fn wayback_releases() {
        let config = json!({
            "10": {"itemTitle": "World Imagery (Wayback 2014-02-20)"},
            "64776": {"itemTitle": "World Imagery (Wayback 2026-10-30)"},
            "64700": {"itemTitle": "World Imagery (Wayback 2026-10-30)"},
            "x": {"itemTitle": "World Imagery (Wayback 2020-01-01)"},
            "5": {"itemTitle": "без даты"},
        });
        let r = parse_wayback(&config);
        assert_eq!(r, vec![Release { number: 64776, day: "2026-10-30".into() }, Release { number: 10, day: "2014-02-20".into() }]);
        assert!(parse_wayback(&json!([1, 2])).is_empty());
        assert_eq!(wayback_url(10), "https://wayback.maptiles.arcgis.com/arcgis/rest/services/World_Imagery/WMTS/1.0.0/default028mm/MapServer/tile/10/{z}/{y}/{x}");
    }
}
