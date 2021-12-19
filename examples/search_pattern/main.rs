use automation::image::{Direction, Finder, Pattern, Screenshot};

fn main() {
    let pattern = Pattern::from_file_buf(include_bytes!("pattern.png")).unwrap();
    let screenshot = Screenshot::from_file_buf(include_bytes!("screenshot.png")).unwrap();

    let finder = Finder::new(&screenshot);
    match finder.find(&pattern, Direction::Left) {
        Some(pos) => println!("Pattern found, position: {:?}", pos),
        None => println!("Pattern not found"),
    }
}
