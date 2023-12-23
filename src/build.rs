use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    write!(
        &mut file,
        "static KEYWORDS: phf::Map<&'static str, Keyword> = {}",
        phf_codegen::Map::new()
            .entry("loop", "Keyword::Loop")
            .entry("continue", "Keyword::Continue")
            .entry("break", "Keyword::Break")
            .entry("fn", "Keyword::Fn")
            .entry("extern", "Keyword::Extern")
            .build()
    )
    .unwrap();
    write!(&mut file, ";\n").unwrap();
}