use assert_cmd::Command;
use dcp::config::VERSION;
use predicates::prelude::*;
use rand::{thread_rng, Rng};
use std::error::Error;
use std::fs::remove_dir_all;

const PRG: &str = "dcp";
const TEST_CONTENT_DIR: &str = "./target/tmp/test_runs";
const DEFAULT_IMAGE: &str = "quay.io/tyslaton/sample-catalog:v0.0.4";
const IMAGE_NO_TAG: &str = "quay.io/tyslaton/sample-catalog";
const SCRATCH_BASE_IMAGE: &str = "quay.io/tflannag/bundles:resolveset-v0.0.2";

// generate_temp_path takes the constant TEST_CONTENT_DIR and
// returns a new string with an appended 5 digit string
fn generate_temp_path() -> String {
    let random_string = thread_rng().gen_range(10000..99999);
    format!("{}/{}", TEST_CONTENT_DIR, random_string)
}

// clean_up_test_dir removes the testing directory specified completely.
//
// WARNING: This function is deleting directories recursively, if you are
//          using it, be absolutely sure you know what you're doing.
fn clean_up_test_dir(path: &str) {
    if path.starts_with(TEST_CONTENT_DIR) {
        match remove_dir_all(path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("failed to delete testing dir {}:{}", path, e);
            }
        }
    }
}

type TestResult = Result<(), Box<dyn Error>>;

// --------------------------------------------------
#[test]
fn prints_version() -> TestResult {
    let expected_version_output: String = format!("dcp {}", VERSION);

    // version is defined and suceeds with the desired output
    Command::cargo_bin(PRG)?
        .args(&["-V"])
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_version_output));

    Ok(())
}

// --------------------------------------------------
#[test]
fn accepts_download_path() -> TestResult {
    let path = &generate_temp_path();

    // content_path is defined and succeeds
    Command::cargo_bin(PRG)?
        .args(&["--download-path", path])
        .args(&[DEFAULT_IMAGE])
        .assert()
        .success();

    // verify that content was written to the desired download_path
    assert_eq!(std::path::Path::new(path).exists(), true);

    clean_up_test_dir(path);

    Ok(())
}

// --------------------------------------------------
#[test]
fn accepts_content_path() -> TestResult {
    let path = &generate_temp_path();
    let content_path = "configs";

    // content_path is defined and succeeds
    Command::cargo_bin(PRG)?
        .args(&["--download-path", path])
        .args(&["--content-path", content_path, DEFAULT_IMAGE])
        .assert()
        .success();

    // verify that content_path grabbed the desired content
    let specific_content = &format!("{}/{}", path, content_path);
    assert_eq!(std::path::Path::new(specific_content).exists(), true);

    clean_up_test_dir(path);

    Ok(())
}

// --------------------------------------------------
#[test]
fn fails_invalid_content_path() -> TestResult {
    let path = &generate_temp_path();
    let content_path = "manifests";

    // --content-path has been specified but fails as
    // there's no "manifests" directory in the
    // DEFAULT_IMAGE container image.
    Command::cargo_bin(PRG)?
        .args(&["--download-path", path])
        .args(&["--content-path", content_path, DEFAULT_IMAGE])
        .assert()
        .failure();

    clean_up_test_dir(path);

    Ok(())
}

// --------------------------------------------------
#[test]
fn accepts_image() -> TestResult {
    let path = &generate_temp_path();

    // image is defined and succeeds
    Command::cargo_bin(PRG)?
        .args(&["--download-path", path])
        .args(&[DEFAULT_IMAGE])
        .assert()
        .success();

    // image is not defined and fails
    Command::cargo_bin(PRG)?
        .args(&["--download-path", path])
        .assert()
        .failure();

    clean_up_test_dir(path);

    Ok(())
}

// --------------------------------------------------
#[test]
fn defaults_tag_to_latest() -> TestResult {
    let path = &generate_temp_path();

    // image is defined and succeeds
    Command::cargo_bin(PRG)?
        .args(&["--download-path", path])
        .args(&[IMAGE_NO_TAG])
        .assert()
        .success();

    clean_up_test_dir(path);

    Ok(())
}

// --------------------------------------------------
#[test]
fn fails_on_just_tag() -> TestResult {
    let path = &generate_temp_path();

    // image is defined and succeeds
    Command::cargo_bin(PRG)?
        .args(&["--download-path", path])
        .args(&[":v0.0.4"])
        .assert()
        .failure();

    clean_up_test_dir(path);

    Ok(())
}

// --------------------------------------------------
#[test]
fn accepts_scratch_base_images() -> TestResult {
    let path = &generate_temp_path();
    let content_path: &str = "manifests";

    Command::cargo_bin(PRG)?
        .args(&["--download-path", path])
        .args(&["--content-path", content_path])
        .args(&[SCRATCH_BASE_IMAGE])
        .assert()
        .success();

    clean_up_test_dir(path);

    Ok(())
}
