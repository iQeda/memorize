// Standalone smoke test: open a copy of an .anki2 collection and dump deck info.
// Run with:
//   cargo run --manifest-path src-tauri/Cargo.toml --example smoke -- /tmp/memorize-test/collection.anki2

use anki::collection::CollectionBuilder;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let path: PathBuf = std::env::args()
        .nth(1)
        .expect("usage: smoke <path-to-collection.anki2>")
        .into();
    println!("opening: {}", path.display());

    let mut col = CollectionBuilder::new(&path).build()?;

    let tree = col.deck_tree(None)?;
    let mut total = 0usize;
    walk(&tree, 0, &mut total);
    println!("\n{} decks (incl. nested)", total);

    let _ = col.close(None);
    Ok(())
}

fn walk(node: &anki_proto::decks::DeckTreeNode, level: u32, total: &mut usize) {
    if node.deck_id != 0 {
        let indent = "  ".repeat(level as usize);
        let short = node.name.split("::").last().unwrap_or(&node.name);
        println!(
            "{}- {} (new={} learn={} review={})",
            indent, short, node.new_count, node.learn_count, node.review_count
        );
        *total += 1;
    }
    for child in &node.children {
        walk(child, level + 1, total);
    }
}
