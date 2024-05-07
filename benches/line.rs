use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, text::Line, widgets::Widget};

fn line_render(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("line_render");
    group.sample_size(1000);

    let line = &Line::from(vec![
        "This".red(),
        " ".green(),
        "is".italic(),
        " ".blue(),
        "SPARTA!!".bold(),
    ])
    .right_aligned();

    for width in [0, 3, 4, 6, 7, 10, 42] {
        let area = Rect::new(0, 0, width, 1);

        group.bench_function(width.to_string(), |bencher| {
            let mut buffer = Buffer::empty(area);
            bencher.iter(|| black_box(line).render(area, &mut buffer));
        });
    }

    group.finish();
}

criterion_group!(benches, line_render);
criterion_main!(benches);
