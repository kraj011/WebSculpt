use gaussian_renderer::run;

fn main() {
    pollster::block_on(run());
}
