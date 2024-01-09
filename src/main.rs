use cgfs_with_wgpu::run;

fn main() {
    pollster::block_on(run());
}