pub mod main {
    pub mod barchart;
    pub mod block;
    pub mod line;
    pub mod list;
    pub mod paragraph;
    pub mod rect;
    pub mod sparkline;
}
pub use main::*;

criterion::criterion_main!(
    barchart::benches,
    block::benches,
    line::benches,
    list::benches,
    paragraph::benches,
    rect::benches,
    sparkline::benches
);
