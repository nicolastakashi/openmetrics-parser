use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Deserialize, Debug)]
struct TestMeta {
    #[serde(alias = "shouldParse")]
    should_parse: bool,
}

fn read_child_file(parent: &Path, filename: &str) -> String {
    let mut child_path = PathBuf::new();
    child_path.push(parent);
    child_path.push(filename);

    assert!(child_path.exists());
    assert!(child_path.is_file());

    let child_str = fs::read_to_string(child_path);
    assert!(child_str.is_ok());

    child_str.unwrap()
}

#[test]
fn run_openmetrics_validation() {
    let tests = fs::read_dir("./OpenMetrics/tests/testdata/parsers");
    assert!(tests.is_ok());

    for test in tests.unwrap() {
        assert!(test.is_ok());
        let test = test.unwrap();
        let path = test.path();
        let test_name = path.file_name().unwrap();

        assert!(path.is_dir());

        let metrics_str = read_child_file(&path, "metrics");
        let test_meta_str = read_child_file(&path, "test.json");

        let meta = serde_json::from_str::<TestMeta>(&test_meta_str);
        assert!(meta.is_ok());
        let meta = meta.unwrap();

        println!("\n[TEST{:?}]", test_name);
        let parsed = crate::openmetrics::parse_openmetrics(&metrics_str);
        let metrics_str = metrics_str.replace(" ", ".").replace("\t", "->");

        if meta.should_parse {
            assert!(
                parsed.is_ok(),
                "\n{}\n Test should parse, but didn't ({:?})",
                metrics_str,
                parsed
            );
        } else {
            assert!(
                parsed.is_err(),
                "\n{}\n Test shouldn't parse, but did ({:?})",
                metrics_str,
                parsed
            );
        }
    }
}

#[test]
fn test_units_by_type() {
    // Counter, gauge, info and stateset units are covered by the conformance suite.
    let cases = [
        (
            "unknown",
            "# TYPE a_seconds unknown\n# UNIT a_seconds seconds\na_seconds 1\n# EOF\n",
            true,
        ),
        (
            "histogram",
            "# TYPE a_seconds histogram\n# UNIT a_seconds seconds\na_seconds_bucket{le=\"0.1\"} 1\na_seconds_bucket{le=\"+Inf\"} 3\na_seconds_sum 0.5\na_seconds_count 3\n# EOF\n",
            true,
        ),
        (
            "gaugehistogram",
            "# TYPE a_seconds gaugehistogram\n# UNIT a_seconds seconds\na_seconds_bucket{le=\"+Inf\"} 3\na_seconds_gcount 3\na_seconds_gsum 0.5\n# EOF\n",
            true,
        ),
        (
            // Taken from the complete exposition example in the OpenMetrics specification.
            "summary",
            "# TYPE acme_http_router_request_seconds summary\n# UNIT acme_http_router_request_seconds seconds\nacme_http_router_request_seconds_sum{path=\"/api/v1\",method=\"GET\"} 9036.32\nacme_http_router_request_seconds_count{path=\"/api/v1\",method=\"GET\"} 807283.0\n# EOF\n",
            true,
        ),
    ];

    for (name, exposition, should_parse) in cases {
        let parsed = crate::openmetrics::parse_openmetrics(exposition);
        assert_eq!(
            parsed.is_ok(),
            should_parse,
            "{}: unexpected result {:?}",
            name,
            parsed
        );
    }
}
