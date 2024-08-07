# Voyager
> A tool for extracting annotation information from aCRF

# List all annotation informations
```rust
#[test]
fn fetch_annotation() {
    let acrf = Path::new(r"D:\projects\rusty\acrf\acrf.pdf");
    let result = voyager::fetch(acrf).unwrap();
    result.iter().for_each(|a| {
        println!("{:?}", a);
    })
}
```

# Export annotation information to excel
```rust
use voyager::{Exporter};

#[test]
fn export_test() {
    let acrf = Path::new(r"D:\projects\rusty\acrf\acrf.pdf");
    let annotations = voyager::fetch(acrf).unwrap();
    let dest = Path::new(r"D:\projects\rusty\acrf");
    let mut worker = Exporter::new();
    worker.add_annotations(&annotations);
    worker.save(dest).unwrap();
}
```