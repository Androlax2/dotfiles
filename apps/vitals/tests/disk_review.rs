use std::fs;
use std::path::PathBuf;
use serde_json::json;
use vitals::{collect::announce_disk_review, store::Store};

const NOW: i64 = 2_000_000_000;

struct Review {
    folder: PathBuf,
    path: PathBuf,
    store: Store,
}

impl Review {
    fn new() -> Self {
        let folder = std::env::temp_dir().join(format!("disk-review-{}", cockpit::util::token_hex(6)));
        fs::create_dir(&folder).unwrap();
        Self { path: folder.join("latest.json"), folder, store: Store::in_memory().unwrap() }
    }

    fn report(&self, completed_at: i64, complete: bool) {
        fs::write(&self.path, json!({"schema_version": 1, "completed_at": completed_at,
            "complete": complete, "advisory_only": true, "games": [], "checkouts": [],
            "errors": [], "warning": "untrusted project #(command)"}).to_string()).unwrap();
    }
}

impl Drop for Review {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.folder).unwrap();
    }
}

#[test]
fn a_review_is_announced_once_and_a_newer_review_is_announced_again() {
    let review = Review::new();
    review.report(NOW - 100, true);
    let mut sent = Vec::new();
    let mut send = |title: &str, detail: &str| { sent.push((title.to_string(), detail.to_string())); true };
    announce_disk_review(&review.store, &review.path, NOW, false, &mut send).unwrap();
    announce_disk_review(&review.store, &review.path, NOW, false, &mut send).unwrap();
    review.report(NOW - 50, true);
    announce_disk_review(&review.store, &review.path, NOW, false, &mut send).unwrap();
    assert_eq!(sent.len(), 2);
    assert!(sent[0].1.contains(review.path.to_str().unwrap()));
    assert!(!sent[0].1.contains("untrusted"));
    assert_eq!(review.store.state("disk_review_notified_at").unwrap().as_deref(), Some("1999999950"));
}

#[test]
fn gaming_defers_a_review_without_acknowledging_it() {
    let review = Review::new();
    review.report(NOW - 100, true);
    let mut count = 0;
    let mut send = |_: &str, _: &str| { count += 1; true };
    announce_disk_review(&review.store, &review.path, NOW, true, &mut send).unwrap();
    assert!(review.store.state("disk_review_notified_at").unwrap().is_none());
    announce_disk_review(&review.store, &review.path, NOW, false, &mut send).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn failed_delivery_is_retried_without_losing_the_report() {
    let review = Review::new();
    review.report(NOW - 100, true);
    announce_disk_review(&review.store, &review.path, NOW, false, &mut |_, _| false).unwrap();
    assert!(review.store.state("disk_review_notified_at").unwrap().is_none());
    let mut sent = false;
    announce_disk_review(&review.store, &review.path, NOW, false, &mut |_, _| { sent = true; true }).unwrap();
    assert!(sent);
}

#[test]
fn missing_review_is_quiet() {
    let review = Review::new();
    announce_disk_review(&review.store, &review.path, NOW, false, &mut |_, _| panic!("missing review was announced")).unwrap();
}

#[test]
fn invalid_reports_fail_visibly_without_acknowledging() {
    let review = Review::new();
    for content in ["invalid".to_string(), "{}".to_string(),
                    json!({"schema_version": 1, "advisory_only": true, "completed_at": NOW + 1,
                        "complete": true, "games": [], "checkouts": [], "errors": []}).to_string(),
                    json!({"schema_version": 1, "advisory_only": true, "completed_at": 0,
                        "complete": true, "games": [], "checkouts": [], "errors": []}).to_string()] {
        fs::write(&review.path, content).unwrap();
        let result = announce_disk_review(&review.store, &review.path, NOW, false, &mut |_, _| panic!("invalid review was announced"));
        assert!(result.is_err());
        assert!(review.store.state("disk_review_notified_at").unwrap().is_none());
    }
}

#[test]
fn partial_review_is_announced_as_partial() {
    let review = Review::new();
    review.report(NOW - 100, false);
    let mut detail = String::new();
    announce_disk_review(&review.store, &review.path, NOW, false, &mut |_, message| { detail = message.to_string(); true }).unwrap();
    assert!(detail.contains("partielle"));
}
