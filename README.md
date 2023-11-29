
# Example

```rust
use readers::*;
use std::io::BufReader;
fn main() -> std::io::Result<()> {

    std::fs::write("1", b"Hello,")?;
    std::fs::write("2", b"Rust!")?;
    let f1 = std::fs::File::open("1")?;
    let f2 = std::fs::File::open("2")?;
    let mut readers = StreamReaders::new();
    readers.push(f1)?;
    readers.push(f2)?;
    let mut reader = BufReader::new(readers);
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    assert_eq!("Hello,Rust!", buf.as_str());
    Ok(())
}
```

