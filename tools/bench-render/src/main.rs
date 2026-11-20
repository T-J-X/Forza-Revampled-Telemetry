use fh6_win::FastLayeredRenderer;

fn main() {
    let renderer = FastLayeredRenderer::new(3840, 2160);
    let (mean, p99) = renderer.bench_dirty_rect(1400, 500, 1000);
    println!("tiny-skia spike: mean={:.3} ms, p99={:.3} ms @4K dirty 1400x500", mean, p99);
    println!("Decision: tiny-skia chosen as default renderer backend");
}
