//! A dojo's progress, read from the JSON its CLI prints.

use serde::{Deserialize, Serialize};

use crate::ProgressSpec;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub done: u64,
    pub total: u64,
    pub next: Option<String>,
    pub unit: String,
}

impl Snapshot {
    pub fn fraction(&self) -> f32 {
        if self.total == 0 { 0.0 } else { (self.done as f32 / self.total as f32).clamp(0.0, 1.0) }
    }
}

pub fn extract(json: &str, spec: &ProgressSpec) -> Option<Snapshot> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let done = v.pointer(&spec.done)?.as_u64()?;
    let total = v.pointer(&spec.total)?.as_u64()?;
    let next = spec
        .next
        .as_deref()
        .and_then(|p| v.pointer(p))
        .and_then(|n| n.as_str())
        .map(|s| s.chars().take(120).collect::<String>())
        .filter(|s| !s.trim().is_empty());
    Some(Snapshot { done: done.min(total), total, next, unit: spec.unit.clone() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Catalog;

    #[test]
    fn reads_poddojo_status() {
        let spec = Catalog::bundled().unwrap().apps[0].progress.clone().unwrap();
        let json = r#"{"lessons_done": 9, "lessons_total": 40, "resume": {"title": "Ingress"}}"#;
        let s = extract(json, &spec).unwrap();
        assert_eq!((s.done, s.total, s.next.as_deref()), (9, 40, Some("Ingress")));
        assert!((s.fraction() - 0.225).abs() < 1e-6);
        let none = r#"{"lessons_done": 0, "lessons_total": 40, "resume": null}"#;
        assert_eq!(extract(none, &spec).unwrap().next, None);
        assert!(extract("not json", &spec).is_none());
        assert!(extract("{}", &spec).is_none());
    }
}
