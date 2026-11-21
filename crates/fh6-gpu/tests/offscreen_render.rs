//! Renders the test cluster on the real GPU (or WARP) without a window and checks the pixels.
use fh6_gpu::test_cluster::{self, TestClusterInput};
use fh6_gpu::{GpuContext, OffscreenTarget, QuadBatch};

fn input(width: f32, height: f32) -> TestClusterInput {
    TestClusterInput {
        speed: 88.0,
        gear: 3,
        rpm: 7000.0,
        max_rpm: 8000.0,
        shift_start: 0.72,
        redline: 0.94,
        throttle: 1.0,
        brake: 0.0,
        anchor: [width * 0.5, height * 0.5],
        scale: height / 2160.0 * 1.45,
        accent: [0.0, 0.824, 1.0, 1.0],
    }
}

#[test]
fn test_cluster_renders_with_transparent_background() {
    let ctx = GpuContext::headless().expect("a DX12 adapter or WARP is available on Windows");
    let (w, h) = (1280u32, 720u32);
    let mut target = OffscreenTarget::new(&ctx, w, h);
    let mut batch = QuadBatch::new();
    test_cluster::build(&mut batch, &input(w as f32, h as f32));
    let image = target.render_to_image(&ctx, 1.0, &batch).expect("readback");

    assert_eq!((image.width, image.height), (w, h));
    // OLED rule: everything outside the cluster is fully transparent (0,0,0,0).
    for (x, y) in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1), (20, h / 2)] {
        assert_eq!(image.pixel(x, y), Some([0, 0, 0, 0]), "pixel ({x},{y}) must be transparent");
    }
    // The cluster covers a bounded area: some pixels, nowhere near the whole frame.
    let covered = image.covered_pixels();
    let total = (w * h) as usize;
    assert!(covered > total / 50, "cluster drew too little: {covered}");
    assert!(covered < total / 2, "cluster must not fill the screen: {covered}");
    // Premultiplied: no channel exceeds alpha.
    assert!(
        image.pixels.chunks_exact(4).all(|p| p[0] <= p[3] && p[1] <= p[3] && p[2] <= p[3]),
        "colour exceeds alpha somewhere: not premultiplied"
    );
}

#[test]
fn opacity_zero_draws_nothing() {
    let ctx = GpuContext::headless().expect("a DX12 adapter or WARP is available on Windows");
    let mut target = OffscreenTarget::new(&ctx, 640, 360);
    let mut batch = QuadBatch::new();
    test_cluster::build(&mut batch, &input(640.0, 360.0));
    let image = target.render_to_image(&ctx, 0.0, &batch).expect("readback");
    assert_eq!(image.covered_pixels(), 0, "hidden means a fully transparent frame");
}

#[test]
fn telemetry_changes_the_image() {
    let ctx = GpuContext::headless().expect("a DX12 adapter or WARP is available on Windows");
    let mut target = OffscreenTarget::new(&ctx, 640, 360);
    let render = |target: &mut OffscreenTarget, inp: TestClusterInput| {
        let mut batch = QuadBatch::new();
        test_cluster::build(&mut batch, &inp);
        target.render_to_image(&ctx, 1.0, &batch).expect("readback")
    };
    let base = input(640.0, 360.0);
    let a = render(&mut target, base);
    let b = render(&mut target, TestClusterInput { gear: 5, ..base });
    let c = render(&mut target, TestClusterInput { speed: 142.0, ..base });
    let d = render(&mut target, TestClusterInput { rpm: 1000.0, ..base });
    assert_ne!(a.pixels, b.pixels, "gear must change the image");
    assert_ne!(a.pixels, c.pixels, "speed must change the image");
    assert_ne!(a.pixels, d.pixels, "rpm must change the image");
}
