// Sample pairs shown on the picker page. Each exercises a different
// aspect of the viewer: mixed edits + a move, code refactor, pure
// reorder, total rewrite.

pub struct SamplePair {
    pub name: &'static str,
    pub description: &'static str,
    pub left: Vec<String>,
    pub right: Vec<String>,
}

pub fn sample_pairs() -> Vec<SamplePair> {
    vec![
        SamplePair {
            name: "Release notes",
            description: "Mixed edit: edits, dissimilar replacements, a stack of deletions, an insertion, a move with a small edit, a colour/color one-word edit, and a blank block.",
            left: strings(&[
                "Release notes",
                "Version 0.3.0 released today.",
                "Highlights",
                "Faster startup, cleaner menus, better search.",
                "banana",
                "Old block one here.",
                "Old block two here.",
                "Old block three here.",
                "Old block four here.",
                "The final section header that pairs well.",
                "Please pick a colour below.",
                "",
                "The old parser was rewritten. Startup went from roughly 800ms to 120ms on a cold cache. The new parser is a hand-written tokenizer plus a small state machine. Every old test was ported and all of them pass.",
                "Paragraph that will move to the bottom of the file in the new version, so the move detection has something to find. It needs to be long enough to pass the exact gate of eight non-whitespace characters — and it is, because this sentence keeps going for a while.",
                "Thanks for reading.",
            ]),
            right: strings(&[
                "Release notes",
                "Version 0.4.0 released today.",
                "Highlights",
                "Faster startup, cleaner menus, better search.",
                "kangaroo",
                "Two new blocks arrived here.",
                "Another new block arrived here.",
                "The final section header that pairs well indeed.",
                "Please pick a color below.",
                "",
                "The old parser was rewritten. Startup went from roughly 800ms to 130ms on a cold cache. The new parser is a hand-written tokenizer plus a small state machine. Every old test was ported and all of them pass.",
                "Thanks for reading.",
                "This is a brand new block one.",
                "This is a brand new block two.",
                "Paragraph that will move to the bottom of the file in the new version, so the move detection has something to find. It needs to be long enough to pass the exact gate of eight non-whitespace characters — and it is, because this sentence keeps going for quite a while indeed.",
            ]),
        },
        SamplePair {
            name: "Code refactor",
            description: "A short Rust module where three function names got longer, the file header got reworded, and no lines were added or removed. Pure edits.",
            left: strings(&[
                "// Calculator module",
                "//",
                "// Simple arithmetic helpers.",
                "fn add(a: i32, b: i32) -> i32 {",
                "    a + b",
                "}",
                "fn sub(a: i32, b: i32) -> i32 {",
                "    a - b",
                "}",
                "fn mul(a: i32, b: i32) -> i32 {",
                "    a * b",
                "}",
                "fn div(a: i32, b: i32) -> i32 {",
                "    a / b",
                "}",
            ]),
            right: strings(&[
                "// Arithmetic helpers",
                "//",
                "// Simple arithmetic utilities.",
                "fn add(a: i32, b: i32) -> i32 {",
                "    a + b",
                "}",
                "fn subtract(a: i32, b: i32) -> i32 {",
                "    a - b",
                "}",
                "fn multiply(a: i32, b: i32) -> i32 {",
                "    a * b",
                "}",
                "fn divide(a: i32, b: i32) -> i32 {",
                "    a / b",
                "}",
            ]),
        },
        SamplePair {
            name: "Reorder only",
            description: "The same five paragraphs, with one rotated from the bottom to the top. Exercises move detection without any edits.",
            left: strings(&[
                "First paragraph with some content here.",
                "Second paragraph with more content here.",
                "Third paragraph with yet more content here.",
                "Fourth paragraph with even more content here.",
                "Fifth paragraph with still more content here.",
            ]),
            right: strings(&[
                "Fifth paragraph with still more content here.",
                "First paragraph with some content here.",
                "Second paragraph with more content here.",
                "Third paragraph with yet more content here.",
                "Fourth paragraph with even more content here.",
            ]),
        },
        SamplePair {
            name: "Total rewrite",
            description: "Every block replaced with unrelated text. No similarity, so everything comes out as collapsed delete/insert rows.",
            left: strings(&[
                "Alpha beta gamma.",
                "Delta epsilon zeta.",
                "Eta theta iota.",
                "Kappa lambda mu.",
            ]),
            right: strings(&[
                "Nu xi omicron.",
                "Pi rho sigma.",
                "Tau upsilon phi.",
                "Chi psi omega.",
            ]),
        },
    ]
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}