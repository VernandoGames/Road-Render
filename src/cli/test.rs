use rbx_types::{Matrix3, Vector3};
use structopt::StructOpt;

use crate::math_lib::axis_angle_conversion;

#[derive(Debug, StructOpt)]
pub struct TestCommand {
    // // Whether this test is true
    // #[structopt(long)]
    // pub bool_test: String,
    // // another test, but it's a string!
    // #[structopt(long)]
    // pub string_test: String,
}

impl TestCommand {
    pub fn run(self) -> anyhow::Result<()> {
        let matrix = Matrix3::new(
            Vector3::new(0.707106769f32, 0f32, 0.707106769f32),
            Vector3::new(0f32, 1f32, 0f32),
            Vector3::new(-0.707106769f32, 0f32, 0.707106769f32),
        );
        let aa = axis_angle_conversion::matrix3_to_axis_angle(matrix);
        println!("{:?}", aa);
        // println!("Receiving chunks");
        // let pb = ProgressBar::new(200);
        // for _ in 0..200 {
        // 	pb.inc(1);
        // 	thread::sleep(Duration::from_millis(5));
        // }
        // pb.finish_with_message("done");
        // println!("Chunks recevied, building image");
        // let pb = ProgressBar::new(200);
        // for _ in 0..200 {
        // 	pb.inc(1);
        // 	thread::sleep(Duration::from_millis(5));
        // }
        // pb.finish_with_message("done");
        Ok(())
    }
}
