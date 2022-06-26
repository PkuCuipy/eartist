mod util;

use std::cmp::min;
use std::fs::File;
use std::io::*;
use util::*;


fn main() {

    // let canvas_height = 50;
    // let canvas_width = 30;
    // let canvas_size = min(canvas_height, canvas_width);
    //
    // let mut ind = Individual::new();
    // for _ in 0..10 {
    //     ind.add_shape("triangle",canvas_height, canvas_width);
    // }
    //
    // let mut canvas1 = Canvas::new(canvas_height, canvas_width);
    // ind.draw_self(&mut canvas1);
    // ind.print_as_json();
    //
    // let mut canvas2 = Canvas::new(canvas_height, canvas_width);
    // ind.mutate(9, canvas_size, 1.0);
    // ind.draw_self(&mut canvas2);
    // ind.print_as_json();

    // 从 .json 文件读取个体
    let mut json_str = String::new();
    File::open("./src/data/ind_cuipy.json").unwrap().read_to_string(&mut json_str).unwrap();
    let ind_cuipy = Individual::from_json(json_str.as_str());

    // 绘制该个体在 canvas 上
    let mut canvas3 = Canvas::new(200, 200);
    ind_cuipy.draw_self(&mut canvas3);

    // 将 canvas 输出到 String
    let mut output = String::new();
    canvas3.print_as_ascii(&mut output);

    // String 输出到文件
    let mut fout = File::create("./src/result/cuipy_image_ascii.txt").unwrap();
    write!(fout, "{}", output);

}
