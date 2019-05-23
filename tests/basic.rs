use opendocument as target;

#[test]
fn load_save_cat() {
    let document = target::document::Document::load_from_path("./sample_data/example.odt").unwrap();
    let tmppath = tempfile::NamedTempFile::new().unwrap().into_temp_path();
    document.save_to_path(&tmppath).unwrap();
    let output = std::process::Command::new("lowriter")
        .arg("--cat")
        .arg(&tmppath)
        .output()
        .expect("failed to execute command");
    //println!("{:?}", tmppath);
    //let _ = tmppath.keep();
    assert_eq!(output.stdout, b"\xef\xbb\xbfasdf\n");
}
