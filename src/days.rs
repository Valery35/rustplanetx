// RustPlanetX - расширение RustGIS для PlanetX Lite.
// Copyright (C) 2026 ООО «Информ++». Лицензия Apache 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Дни рядов NASA GIBS, перенос `crates/data/src/gibs.rs` PlanetX Lite.
//!
//! Часов в песочнице нет. Последний день берётся из ряда дат
//! DescribeDomains, ползунок панели отсчитывает дни назад от него.

/// День календаря.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Day {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

/// Дней от 1970-01-01, алгоритм Говарда Хиннанта.
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = i64::from(y) - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// День по номеру дня от 1970-01-01.
fn civil_from_days(z: i64) -> Day {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
    Day { year, month, day }
}

fn month_days(year: i32, month: u32) -> u32 {
    let next = if month == 12 { days_from_civil(year + 1, 1, 1) } else { days_from_civil(year, month + 1, 1) };
    (next - days_from_civil(year, month, 1)) as u32
}

impl Day {
    /// День из строки ГГГГ-ММ-ДД.
    pub fn parse(text: &str) -> Option<Day> {
        let mut parts = text.splitn(3, '-');
        let year = parts.next()?.parse().ok()?;
        let month: u32 = parts.next()?.parse().ok()?;
        let day: u32 = parts.next()?.get(..2)?.parse().ok()?;
        ((1..=12).contains(&month) && day >= 1 && day <= month_days(year, month)).then_some(Day { year, month, day })
    }

    fn number(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    /// День через count шагов unit (D, M или Y). День месяца не больше
    /// длины месяца.
    pub fn add(self, count: i64, unit: char) -> Day {
        if unit == 'D' {
            return civil_from_days(self.number() + count);
        }
        let months = i64::from(self.month) - 1 + if unit == 'M' { count } else { 12 * count };
        let year = self.year + months.div_euclid(12) as i32;
        let month = (months.rem_euclid(12) + 1) as u32;
        Day { year, month, day: self.day.min(month_days(year, month)) }
    }

    /// Строка ГГГГ-ММ-ДД.
    pub fn text(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Промежуток ряда: начало, конец, шаг, единица шага D, M или Y.
pub type Interval = (Day, Day, i64, char);

/// Ряд дат слоя из ответа DescribeDomains. Шаги меньше суток не
/// разбираются. Ответ считается враждебным.
pub fn parse_domains(data: &str) -> Vec<Interval> {
    let Some(start) = data.find("<Domain>") else {
        return Vec::new();
    };
    let body = &data[start + 8..];
    let Some(end) = body.find("</Domain>") else {
        return Vec::new();
    };
    body[..end]
        .split(',')
        .filter_map(|part| {
            let mut it = part.trim().split('/');
            let first = Day::parse(it.next()?)?;
            let last = Day::parse(it.next()?)?;
            let period = it.next()?.strip_prefix('P')?;
            let unit = period.chars().last()?;
            let count: i64 = period.get(..period.len() - unit.len_utf8())?.parse().ok()?;
            (matches!(unit, 'D' | 'M' | 'Y') && count > 0 && first <= last).then_some((first, last, count, unit))
        })
        .collect()
}

/// Последний день ряда start..end с шагом не позже day.
fn floor(interval: Interval, day: Day) -> Day {
    let (start, end, count, unit) = interval;
    let found = if unit == 'D' {
        let steps = (day.number() - start.number()).div_euclid(count);
        start.add(steps * count, 'D')
    } else {
        let mut months = i64::from(day.year - start.year) * 12 + i64::from(day.month) - i64::from(start.month);
        if unit == 'Y' {
            months -= months.rem_euclid(12);
        }
        let step = count * if unit == 'Y' { 12 } else { 1 };
        let found = start.add(months.div_euclid(step) * step, 'M');
        if found > day { start.add((months.div_euclid(step) - 1) * step, 'M') } else { found }
    };
    found.min(end)
}

/// Последний день ряда.
pub fn last(intervals: &[Interval]) -> Option<Day> {
    intervals.iter().map(|i| i.1).max()
}

/// День ряда для дня day: последний день ряда не позже day, раньше
/// начала ряда - первый. Пустой ряд - None.
pub fn pick(intervals: &[Interval], day: Day) -> Option<Day> {
    let mut ordered = intervals.to_vec();
    ordered.sort();
    let first = ordered.first()?.0;
    let mut best = None;
    for interval in &ordered {
        if interval.0 > day {
            break;
        }
        best = Some(floor(*interval, day));
    }
    Some(best.unwrap_or(first))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOMAINS: &str = "<Domains><DimensionDomain><ows:Identifier>time</ows:Identifier>\
        <Domain>2015-11-24/2022-07-27/P1D,2022-08-10/2024-05-31/P1D,2026-07-16/2026-11-02/P1D</Domain></DimensionDomain></Domains>";

    fn day(text: &str) -> Day {
        Day::parse(text).unwrap_or(Day { year: 0, month: 1, day: 1 })
    }

    #[test]
    fn domains_and_pick() {
        let r = parse_domains(DOMAINS);
        assert_eq!(r.len(), 3);
        assert_eq!(last(&r), Some(day("2026-11-02")));
        // В пропуске ряда - последний день до него.
        assert_eq!(pick(&r, day("2022-08-01")), Some(day("2022-07-27")));
        assert_eq!(pick(&r, day("2026-11-01")), Some(day("2026-11-01")));
        // Раньше начала ряда - первый день.
        assert_eq!(pick(&r, day("2000-01-01")), Some(day("2015-11-24")));
        assert_eq!(pick(&[], day("2000-01-01")), None);
    }

    #[test]
    fn monthly_series() {
        let r = parse_domains("<Domain>2002-09-01/2026-09-01/P1M</Domain>");
        assert_eq!(pick(&r, day("2026-10-15")), Some(day("2026-09-01")));
        assert_eq!(pick(&r, day("2024-03-31")), Some(day("2024-03-01")));
    }

    #[test]
    fn hostile_domains() {
        for bad in ["", "<Domain>", "<Domain>x/y/P1D</Domain>", "<Domain>2020-01-01/2019-01-01/P1D</Domain>", "<Domain>2020-01-01/2021-01-01/P0D</Domain>", "<Domain>2020-01-01/2021-01-01/PЖ</Domain>"] {
            assert!(parse_domains(bad).is_empty(), "{bad}");
        }
    }

    #[test]
    fn steps_back() {
        assert_eq!(day("2026-03-01").add(-1, 'D').text(), "2026-02-28");
        assert_eq!(day("2024-03-31").add(-1, 'M').text(), "2024-02-29");
        assert_eq!(Day::parse("2026-02-30"), None);
    }
}
