use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use ts_rs::TS;
use zyntax_core::{ModelPricing, TokenUsage};

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("history database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FixRecord {
    #[ts(type = "number")]
    pub id: i64,

    #[ts(type = "number")]
    pub timestamp: i64,
    pub provider: String,
    pub model: String,
    pub persona_id: String,
    pub language_tag: String,
    pub translate: bool,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub original_chars: u32,
    pub corrected_chars: u32,

    pub changed: bool,
    pub elapsed_ms: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewFix {
    pub timestamp: i64,
    pub provider: String,
    pub model: String,
    pub persona_id: String,
    pub language_tag: String,
    pub translate: bool,
    pub usage: TokenUsage,
    pub original_chars: u32,
    pub corrected_chars: u32,
    pub changed: bool,
    pub elapsed_ms: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UsageSummary {
    pub fixes: u32,

    #[ts(type = "number")]
    pub input_tokens: u64,
    #[ts(type = "number")]
    pub output_tokens: u64,
}

impl UsageSummary {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens.saturating_add(self.output_tokens)
    }

    pub fn cost(&self, pricing: ModelPricing) -> f64 {
        (self.input_tokens as f64 * pricing.input_per_million
            + self.output_tokens as f64 * pricing.output_per_million)
            / 1_000_000.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Stats {
    pub total_fixes: u32,

    #[ts(type = "number | null")]
    pub last_fix: Option<i64>,
}

pub struct History {
    conn: Connection,
}

impl History {
    pub fn open(path: &Path) -> Result<Self, HistoryError> {
        let history = Self {
            conn: Connection::open(path)?,
        };
        history.migrate()?;
        Ok(history)
    }

    pub fn in_memory() -> Result<Self, HistoryError> {
        let history = Self {
            conn: Connection::open_in_memory()?,
        };
        history.migrate()?;
        Ok(history)
    }

    fn migrate(&self) -> Result<(), HistoryError> {
        self.conn.pragma_update(None, "journal_mode", "WAL")?;
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS fixes (
                 id              INTEGER PRIMARY KEY AUTOINCREMENT,
                 timestamp       INTEGER NOT NULL,
                 provider        TEXT    NOT NULL,
                 model           TEXT    NOT NULL,
                 persona_id      TEXT    NOT NULL,
                 language_tag    TEXT    NOT NULL,
                 translate       INTEGER NOT NULL,
                 input_tokens    INTEGER NOT NULL,
                 output_tokens   INTEGER NOT NULL,
                 original_chars  INTEGER NOT NULL,
                 corrected_chars INTEGER NOT NULL,
                 changed         INTEGER NOT NULL,
                 elapsed_ms      INTEGER NOT NULL
             );
             -- Every period query filters on timestamp, so this index is what
             -- keeps 'this month' from scanning the whole table.
             CREATE INDEX IF NOT EXISTS idx_fixes_timestamp ON fixes(timestamp);
             CREATE INDEX IF NOT EXISTS idx_fixes_model ON fixes(provider, model);",
        )?;
        Ok(())
    }

    pub fn record(&self, fix: &NewFix) -> Result<i64, HistoryError> {
        self.conn.execute(
            "INSERT INTO fixes (
                 timestamp, provider, model, persona_id, language_tag, translate,
                 input_tokens, output_tokens, original_chars, corrected_chars,
                 changed, elapsed_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                fix.timestamp,
                fix.provider,
                fix.model,
                fix.persona_id,
                fix.language_tag,
                fix.translate,
                fix.usage.input_tokens,
                fix.usage.output_tokens,
                fix.original_chars,
                fix.corrected_chars,
                fix.changed,
                fix.elapsed_ms,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn stats(&self) -> Result<Stats, HistoryError> {
        let row = self
            .conn
            .query_row("SELECT COUNT(*), MAX(timestamp) FROM fixes", [], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, Option<i64>>(1)?))
            })?;
        Ok(Stats {
            total_fixes: u32::try_from(row.0).unwrap_or(u32::MAX),
            last_fix: row.1,
        })
    }

    pub fn usage_between(&self, from: i64, to: i64) -> Result<UsageSummary, HistoryError> {
        let summary = self.conn.query_row(
            "SELECT COUNT(*),
                    COALESCE(SUM(input_tokens), 0),
                    COALESCE(SUM(output_tokens), 0)
             FROM fixes
             WHERE timestamp >= ?1 AND timestamp < ?2",
            params![from, to],
            |row| {
                Ok(UsageSummary {
                    fixes: u32::try_from(row.get::<_, i64>(0)?).unwrap_or(u32::MAX),
                    input_tokens: row.get::<_, i64>(1)?.max(0) as u64,
                    output_tokens: row.get::<_, i64>(2)?.max(0) as u64,
                })
            },
        )?;
        Ok(summary)
    }

    pub fn usage_total(&self) -> Result<UsageSummary, HistoryError> {
        self.usage_between(i64::MIN, i64::MAX)
    }

    pub fn usage_by_model(&self) -> Result<Vec<(String, String, UsageSummary)>, HistoryError> {
        self.usage_by_model_between(i64::MIN, i64::MAX)
    }

    pub fn usage_by_model_between(
        &self,
        from: i64,
        to: i64,
    ) -> Result<Vec<(String, String, UsageSummary)>, HistoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT provider, model, COUNT(*),
                    COALESCE(SUM(input_tokens), 0),
                    COALESCE(SUM(output_tokens), 0)
             FROM fixes
             WHERE timestamp >= ?1 AND timestamp < ?2
             GROUP BY provider, model
             ORDER BY SUM(input_tokens + output_tokens) DESC",
        )?;

        let rows = stmt
            .query_map(params![from, to], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    UsageSummary {
                        fixes: u32::try_from(row.get::<_, i64>(2)?).unwrap_or(u32::MAX),
                        input_tokens: row.get::<_, i64>(3)?.max(0) as u64,
                        output_tokens: row.get::<_, i64>(4)?.max(0) as u64,
                    },
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn usage_by_day(
        &self,
        from: i64,
        to: i64,
        utc_offset_seconds: i64,
    ) -> Result<Vec<(i64, UsageSummary)>, HistoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT ((timestamp + ?3) / 86400) * 86400 - ?3 AS day_start,
                    COUNT(*),
                    COALESCE(SUM(input_tokens), 0),
                    COALESCE(SUM(output_tokens), 0)
             FROM fixes
             WHERE timestamp >= ?1 AND timestamp < ?2
             GROUP BY day_start
             ORDER BY day_start",
        )?;

        let rows = stmt
            .query_map(params![from, to, utc_offset_seconds], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    UsageSummary {
                        fixes: u32::try_from(row.get::<_, i64>(1)?).unwrap_or(u32::MAX),
                        input_tokens: row.get::<_, i64>(2)?.max(0) as u64,
                        output_tokens: row.get::<_, i64>(3)?.max(0) as u64,
                    },
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn recent(&self, limit: u32) -> Result<Vec<FixRecord>, HistoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, provider, model, persona_id, language_tag, translate,
                    input_tokens, output_tokens, original_chars, corrected_chars,
                    changed, elapsed_ms
             FROM fixes
             ORDER BY timestamp DESC, id DESC
             LIMIT ?1",
        )?;

        let rows = stmt
            .query_map(params![limit], |row| {
                Ok(FixRecord {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    provider: row.get(2)?,
                    model: row.get(3)?,
                    persona_id: row.get(4)?,
                    language_tag: row.get(5)?,
                    translate: row.get(6)?,
                    input_tokens: row.get(7)?,
                    output_tokens: row.get(8)?,
                    original_chars: row.get(9)?,
                    corrected_chars: row.get(10)?,
                    changed: row.get(11)?,
                    elapsed_ms: row.get(12)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn clear(&self) -> Result<(), HistoryError> {
        self.conn.execute("DELETE FROM fixes", [])?;
        Ok(())
    }

    pub fn first_record_at(&self) -> Result<Option<i64>, HistoryError> {
        Ok(self
            .conn
            .query_row("SELECT MIN(timestamp) FROM fixes", [], |row| row.get(0))
            .optional()?
            .flatten())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix_at(timestamp: i64, input: u32, output: u32) -> NewFix {
        NewFix {
            timestamp,
            provider: "gemini".to_owned(),
            model: "gemini-2.5-flash".to_owned(),
            persona_id: "standard".to_owned(),
            language_tag: "auto".to_owned(),
            translate: false,
            usage: TokenUsage {
                input_tokens: input,
                output_tokens: output,
            },
            original_chars: 40,
            corrected_chars: 42,
            changed: true,
            elapsed_ms: 900,
        }
    }

    #[test]
    fn a_fresh_database_is_empty() {
        let history = History::in_memory().expect("open");
        assert_eq!(history.stats().unwrap(), Stats::default());
        assert_eq!(history.usage_total().unwrap(), UsageSummary::default());
        assert!(history.recent(10).unwrap().is_empty());
        assert_eq!(history.first_record_at().unwrap(), None);
    }

    #[test]
    fn records_accumulate_into_stats() {
        let history = History::in_memory().expect("open");
        history.record(&fix_at(1_000, 10, 20)).unwrap();
        history.record(&fix_at(2_000, 30, 40)).unwrap();

        let stats = history.stats().unwrap();
        assert_eq!(stats.total_fixes, 2);
        assert_eq!(stats.last_fix, Some(2_000));
    }

    #[test]
    fn totals_sum_both_token_directions() {
        let history = History::in_memory().expect("open");
        history.record(&fix_at(1_000, 10, 20)).unwrap();
        history.record(&fix_at(2_000, 30, 40)).unwrap();

        let usage = history.usage_total().unwrap();
        assert_eq!(usage.fixes, 2);
        assert_eq!(usage.input_tokens, 40);
        assert_eq!(usage.output_tokens, 60);
        assert_eq!(usage.total_tokens(), 100);
    }

    #[test]
    fn window_boundaries_are_inclusive_start_exclusive_end() {
        let history = History::in_memory().expect("open");
        history.record(&fix_at(100, 1, 1)).unwrap();
        history.record(&fix_at(200, 2, 2)).unwrap();
        history.record(&fix_at(300, 4, 4)).unwrap();

        let usage = history.usage_between(200, 300).unwrap();
        assert_eq!(usage.fixes, 1);
        assert_eq!(usage.input_tokens, 2);
    }

    #[test]
    fn an_empty_window_reports_zero_rather_than_failing() {
        let history = History::in_memory().expect("open");
        history.record(&fix_at(100, 1, 1)).unwrap();
        assert_eq!(
            history.usage_between(5_000, 6_000).unwrap(),
            UsageSummary::default()
        );
    }

    #[test]
    fn usage_is_grouped_per_model() {
        let history = History::in_memory().expect("open");
        history.record(&fix_at(100, 10, 10)).unwrap();

        let mut other = fix_at(200, 100, 100);
        other.model = "gemini-2.5-pro".to_owned();
        history.record(&other).unwrap();

        let by_model = history.usage_by_model().unwrap();
        assert_eq!(by_model.len(), 2);

        assert_eq!(by_model[0].1, "gemini-2.5-pro");
    }

    #[test]
    fn per_model_usage_can_be_scoped_to_a_window() {
        let history = History::in_memory().expect("open");
        history.record(&fix_at(100, 10, 10)).unwrap();

        let mut later = fix_at(500, 100, 100);
        later.model = "gemini-2.5-pro".to_owned();
        history.record(&later).unwrap();

        let window = history.usage_by_model_between(400, 600).unwrap();
        assert_eq!(window.len(), 1, "only the in-window model appears");
        assert_eq!(window[0].1, "gemini-2.5-pro");
    }

    #[test]
    fn per_model_costs_differ_from_a_blended_rate() {
        let cheap = UsageSummary {
            fixes: 1,
            input_tokens: 1_000_000,
            output_tokens: 0,
        };
        let dear = UsageSummary {
            fixes: 1,
            input_tokens: 1_000_000,
            output_tokens: 0,
        };

        let cheap_cost = cheap.cost(ModelPricing {
            input_per_million: 0.10,
            output_per_million: 0.40,
        });
        let dear_cost = dear.cost(ModelPricing {
            input_per_million: 3.00,
            output_per_million: 15.00,
        });

        assert!((cheap_cost + dear_cost - 3.10).abs() < 1e-9);
    }

    #[test]
    fn cost_uses_separate_input_and_output_prices() {
        let usage = UsageSummary {
            fixes: 1,
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
        };
        let pricing = ModelPricing {
            input_per_million: 0.30,
            output_per_million: 2.50,
        };
        assert!((usage.cost(pricing) - 2.80).abs() < 1e-9);
    }

    #[test]
    fn recent_returns_newest_first_and_respects_the_limit() {
        let history = History::in_memory().expect("open");
        for ts in [100, 200, 300] {
            history.record(&fix_at(ts, 1, 1)).unwrap();
        }

        let recent = history.recent(2).unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].timestamp, 300);
        assert_eq!(recent[1].timestamp, 200);
    }

    #[test]
    fn records_round_trip_every_field() {
        let history = History::in_memory().expect("open");
        let mut fix = fix_at(1_700_000_000, 11, 22);
        fix.translate = true;
        fix.changed = false;
        fix.language_tag = "de".to_owned();
        fix.persona_id = "creative".to_owned();
        history.record(&fix).unwrap();

        let record = &history.recent(1).unwrap()[0];
        assert_eq!(record.timestamp, 1_700_000_000);
        assert!(record.translate);
        assert!(!record.changed);
        assert_eq!(record.language_tag, "de");
        assert_eq!(record.persona_id, "creative");
        assert_eq!(record.input_tokens, 11);
        assert_eq!(record.output_tokens, 22);
        assert_eq!(record.elapsed_ms, 900);
    }

    #[test]
    fn usage_is_bucketed_by_day() {
        let history = History::in_memory().expect("open");

        history.record(&fix_at(1_700_000_000, 10, 10)).unwrap();
        history.record(&fix_at(1_700_003_600, 5, 5)).unwrap();
        history.record(&fix_at(1_700_100_000, 1, 1)).unwrap();

        let days = history.usage_by_day(0, i64::MAX, 0).unwrap();
        assert_eq!(days.len(), 2, "got {days:?}");
        assert_eq!(days[0].1.fixes, 2);
        assert_eq!(days[0].1.input_tokens, 15);
        assert_eq!(days[1].1.fixes, 1);
    }

    #[test]
    fn the_offset_moves_the_day_boundary() {
        let history = History::in_memory().expect("open");

        let late = 1_700_005_800;
        history.record(&fix_at(late, 1, 1)).unwrap();
        history.record(&fix_at(late + 7_200, 1, 1)).unwrap();

        let utc = history.usage_by_day(0, i64::MAX, 0).unwrap();
        let plus_two = history.usage_by_day(0, i64::MAX, 7_200).unwrap();
        assert_ne!(
            utc.len(),
            plus_two.len(),
            "the offset must change where the day breaks"
        );
    }

    #[test]
    fn day_buckets_respect_the_window() {
        let history = History::in_memory().expect("open");
        history.record(&fix_at(1_000, 1, 1)).unwrap();
        history.record(&fix_at(500_000, 1, 1)).unwrap();

        assert_eq!(history.usage_by_day(400_000, 600_000, 0).unwrap().len(), 1);
        assert!(history
            .usage_by_day(900_000, 950_000, 0)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn clearing_removes_everything() {
        let history = History::in_memory().expect("open");
        history.record(&fix_at(100, 1, 1)).unwrap();
        history.clear().unwrap();

        assert_eq!(history.stats().unwrap(), Stats::default());
        assert!(history.recent(10).unwrap().is_empty());
    }

    #[test]
    fn survives_reopening_the_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("history.db");

        {
            let history = History::open(&path).expect("open");
            history.record(&fix_at(100, 5, 5)).unwrap();
        }

        let history = History::open(&path).expect("reopen");
        assert_eq!(history.stats().unwrap().total_fixes, 1);
    }

    #[test]
    fn opening_twice_does_not_duplicate_the_schema() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("history.db");
        History::open(&path).expect("open");
        History::open(&path).expect("open again");
    }
}
