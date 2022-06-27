use std::cmp::{min, max};
use std::fmt::Write;
use serde::*;


pub mod random {
    use rand::{Rng, thread_rng};

    /// 生成 U(low, high) 随机浮点数
    pub fn uniform(low: f32, high: f32) -> f32 {
        thread_rng().gen_range(low, high)
    }

    /// 生成 N(0, sigma) 随机数
    pub fn normal(sigma: f32) -> f32 {
        thread_rng().sample::<f32, _>(rand_distr::StandardNormal) * sigma
    }

    /// 生成 [i, j) 随机整数
    pub fn randint<T: rand_distr::uniform::SampleUniform>(low: T, high: T) -> T {
        thread_rng().gen_range(low, high)
    }
}
trait MutatableFloat {
    fn mutate(self, sigma: f32, min: f32, max: f32) -> Self;
}
impl MutatableFloat for f32 {
    fn mutate(self, sigma: f32, min: f32, max: f32) -> f32 {
        (self + random::normal(sigma)).clamp(min, max)
    }
}


/// RGBA 颜色类
#[derive(Copy, Clone, Debug)]
#[derive(Serialize, Deserialize)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
impl Color {
    /// 随机初始化
    fn rand_new() -> Color {
        Color {
            r: random::uniform(0., 255.),
            g: random::uniform(0., 255.),
            b: random::uniform(0., 255.),
            a: random::uniform(0., 1.),
        }
    }

    /// 随机变异. amp 是缩放系数
    fn mutate(&mut self, amp: f32) {
        self.r = self.r.mutate(20. * amp, 0., 255.);
        self.g = self.g.mutate(20. * amp, 0., 255.);
        self.b = self.b.mutate(20. * amp, 0., 255.);
        self.a = self.a.mutate(0.03 * amp, 0., 1.);
    }
}


/// 2D 坐标
#[derive(Copy, Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct Point2D {
    x: f32,
    y: f32,
}
impl Point2D {
    pub fn new(x: f32, y: f32) -> Point2D {
        Point2D { x, y }
    }
    pub fn rand_new(x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Point2D {
        Point2D::new(
            random::uniform(x_min, x_max),
            random::uniform(y_min, y_max),
        )
    }
    pub fn mutate(&mut self, sigma: f32, amp: f32) {
        self.x = self.x.mutate(sigma * amp, f32::NEG_INFINITY, f32::INFINITY);
        self.y = self.y.mutate(sigma * amp, f32::NEG_INFINITY, f32::INFINITY);
    }
}


/// 形如 y = kx + b 的 2D 直线
struct Line {
    k: f32,
    b: f32,
}
impl Line {
    /// 从两个点构造
    fn new(p1: &Point2D, p2: &Point2D) -> Line {
        let k = (p2.y - p1.y) / (p2.x - p1.x);  // 这里可能是 NaN 或者 Infinity, 但无所谓
        let b = p1.y - k * p1.x;
        Line { k, b }
    }

    /// 直线在 x 处的取值
    fn at(&self, x: f32) -> f32 {
        self.k * x + self.b
    }
}


/// RGB 像素类
#[derive(Copy, Clone, Debug)]
struct Pixel {
    r: f32,
    g: f32,
    b: f32,
}
impl Pixel {
    /// 新建一个颜色为 r, g, b 的像素
    fn new(r: f32, g: f32, b: f32) -> Pixel {
        Pixel { r, g, b }
    }

    /// 层叠另一个半透明颜色
    fn overlaid_by(&mut self, color: &Color) {
        let (my_weight, his_weight) = (1.0 - color.a, color.a);
        self.r = self.r * my_weight + color.r * his_weight;
        self.g = self.g * my_weight + color.g * his_weight;
        self.b = self.b * my_weight + color.b * his_weight;
    }

    /// L2 距离
    fn l2_dist(p1: &Pixel, p2: &Pixel) -> f32 {
        (p1.r - p2.r).powi(2) + (p1.g - p2.g).powi(2) + (p1.b - p2.b).powi(2)
    }
}

// 画布 (RGB 的)
pub struct Canvas {
    pub x_height: usize,
    pub y_width: usize,
    pixels: Vec<Pixel>,
}
impl Canvas {
    /// 新建一个 x_height × y_width 的纯黑画布
    pub fn new(x_height: usize, y_width: usize, color: (f32, f32, f32)) -> Canvas {
        let (r, g, b) = color;
        let n_pixels = x_height * y_width;
        let mut pixels = Vec::<Pixel>::with_capacity(n_pixels);
        for _ in 0..n_pixels {
            pixels.push(Pixel::new(r, g, b));
        }
        Canvas {
            x_height,
            y_width,
            pixels,
        }
    }

    /// 从图片中读取
    pub fn read_from_file(file_path: &str) -> Canvas {
        let im = image::open(file_path).unwrap().to_rgb8();
        let width = im.width() as usize;
        let height = im.height() as usize;
        let rgb_seq = im.into_raw();
        let n_pixels = width * height;
        let mut pixels =  Vec::<Pixel>::with_capacity(n_pixels);
        for i in 0..n_pixels {
            let r = rgb_seq[i * 3];
            let g = rgb_seq[i * 3 + 1];
            let b = rgb_seq[i * 3 + 2];
            pixels.push(Pixel::new(r as f32, g as f32, b as f32));
        }
        Canvas {
            x_height: height,
            y_width: width,
            pixels,
        }
    }

    /// 二维 i, j 坐标转为线性索引
    #[inline]
    fn idx_of(&self, i: usize, j: usize) -> usize {
        debug_assert!(i < self.x_height && j < self.y_width, "越界的 [i, j] 索引!");
        (self.y_width * i) + j
    }

    /// 画一条水平线
    fn draw_horizontal_line(&mut self, i: usize, j1: usize, j2: usize, color: &Color) -> () {
        let j_left = min(j1, j2);
        let j_right = max(j1, j2);
        debug_assert!(j_right < self.y_width, "超出范围的网格坐标!");
        for j in j_left..=j_right {
            let idx = self.idx_of(i, j);
            self.pixels[idx].overlaid_by(color);
        }
    }

    /// 对比两个 Canvas 的逐像素 L2 差异
    pub fn l2_diff(canvas1: &Canvas, canvas2: &Canvas) -> f32 {
        debug_assert!(canvas1.x_height == canvas2.x_height && canvas1.y_width == canvas2.y_width, "只有尺寸相同的两个 canvas 才能计算差异!");
        let mut total_diff = 0.0;
        for idx in 0..canvas1.pixels.len() {
            total_diff += Pixel::l2_dist(&canvas1.pixels[idx], &canvas2.pixels[idx]);
        }
        f32::sqrt(total_diff / (canvas1.pixels.len() as f32 * 3.0))
    }

    /// 将自己以 ASCII 格式输出
    pub fn print_as_ascii(&mut self, buf: &mut String) {
        write!(buf, "h={} w={}\n", self.x_height, self.y_width).unwrap();
        for pixel in &self.pixels {
            write!(buf, "{} {} {}\n", pixel.r, pixel.g, pixel.b).unwrap();
        }
    }

    /// 将自己以图片格式输出到文件
    pub fn write_to_file(&self, filename: &str) {
        let mut buffer = Vec::with_capacity(self.x_height * self.y_width * 3);
        for pixel in &self.pixels {
            buffer.push(pixel.r as u8);
            buffer.push(pixel.g as u8);
            buffer.push(pixel.b as u8);
        }
        image::save_buffer(filename, &buffer, self.y_width as u32, self.x_height as u32, image::ColorType::Rgb8).unwrap();
    }

}



/// 三种用于生成图片的基本图元
#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum Shape {
    Triangle {
        p1: Point2D,
        p2: Point2D,
        p3: Point2D,
        color: Color,
    },
    Circle {
        center: Point2D,
        radius: [f32; 1],
        color: Color,
    },
    Rectangle {
        p1: Point2D,
        p2: Point2D,
        color: Color,
    },
}
impl Shape {
    /// 根据指定名称, 随机初始化一个形状
    fn rand_new(type_name: &str, x_height: usize, y_width: usize) -> Shape {
        match type_name.to_lowercase().as_str() {
            "triangle" => Shape::Triangle {
                p1: Point2D::rand_new(0.0, x_height as f32, 0.0, y_width as f32),
                p2: Point2D::rand_new(0.0, x_height as f32, 0.0, y_width as f32),
                p3: Point2D::rand_new(0.0, x_height as f32, 0.0, y_width as f32),
                color: Color::rand_new(),
            },
            "circle" => Shape::Circle {
                center: Point2D::rand_new(0.0, x_height as f32, 0.0, y_width as f32),
                radius: [crate::random::uniform(0.0, 0.1 * min(x_height, y_width) as f32).clamp(1.0, f32::INFINITY)],  // FIXME: MAGIC_NUMBER: 0.1
                color: Color::rand_new(),
            },
            "rectangle" => Shape::Rectangle {
                p1: Point2D::rand_new(0.0, x_height as f32, 0.0, y_width as f32),
                p2: Point2D::rand_new(0.0, x_height as f32, 0.0, y_width as f32),
                color: Color::rand_new(),
            },
            _ => panic!("未知的 Shape 类别!"),
        }
    }

    /// 对当前形状进行变异
    ///  - `canvas_size`: 当前画布的短边长度
    ///  - `amp`: 变异的增益系数, 指定为 1.0 为默认
    fn mutate(&mut self, canvas_size: usize, amp: f32) {
        let pixel_sigma = (canvas_size as f32) * 0.03;    // 位置参数的变动, 单位为 pixel, 含义是正态分布的 1σ
        match self {
            Shape::Triangle { p1, p2, p3, color } => {
                p1.mutate(pixel_sigma, amp);
                p2.mutate(pixel_sigma, amp);
                p3.mutate(pixel_sigma, amp);
                color.mutate(amp);
            }
            Shape::Circle { center, radius, color } => {
                center.mutate(pixel_sigma, amp);
                color.mutate(amp);
                radius[0] = radius[0].mutate(pixel_sigma, f32::NEG_INFINITY, f32::INFINITY);
            },
            Shape::Rectangle { p1, p2, color } => {
                p1.mutate(pixel_sigma, amp);
                p2.mutate(pixel_sigma, amp);
                color.mutate(amp);
            }
            _ => panic!(),
        }
    }

    /// 把自己绘制在目标画布上
    fn draw_to(&self, canvas: &mut Canvas) {

        // 将浮点坐标对齐到网格, 并对越界的规范到边界
        let x_max = (canvas.x_height - 1) as f32;
        let y_max = (canvas.y_width - 1) as f32;
        let x2i = |x: f32| {x.round().clamp(0.0, x_max as f32) as usize};
        let y2j = |y: f32| {y.round().clamp(0.0, y_max as f32) as usize};

        match self {
            Shape::Triangle {p1, p2, p3, color} => {
                // 首先对三个点重命名, 使得 A.x ≤ B.x ≤ C.x
                let mut three_points = [p1, p2, p3];
                three_points.sort_by(|p1, p2| p1.x.partial_cmp(&p2.x).unwrap());
                let [p_a, p_b, p_c] = three_points;
                debug_assert!(p_a.x <= p_b.x && p_b.x <= p_c.x);
                //
                //       A                  A           <---  i_start \
                //      / \                / \                         }  Part I
                //     /   \      or      /   \                       /
                //   B `--_ \            /_-- ^` B      <---  i_mid  \
                //          `C         C                <---  i_end  /   Part II
                //
                let l_ab = Line::new(p_a, p_b);
                let l_ac = Line::new(p_a, p_c);
                let l_bc = Line::new(p_b, p_c);
                let i_start = x2i(p_a.x);
                let i_mid = x2i(p_b.x);
                let i_end = x2i(p_c.x);
                // 将三角形分成两部分, 按行绘制
                // Part I
                for i in i_start..i_mid {
                    let j_one_side = y2j(l_ab.at(i as f32));
                    let j_another_side = y2j(l_ac.at(i as f32));
                    canvas.draw_horizontal_line(i, j_one_side, j_another_side, color);
                }
                // Part II
                for i in i_mid..=i_end {
                    let j_one_side = y2j(l_bc.at(i as f32));
                    let j_another_side = y2j(l_ac.at(i as f32));
                    canvas.draw_horizontal_line(i, j_one_side, j_another_side, color);
                }
            },

            Shape::Circle { center, radius, color } => {
                let (r, cx, cy) = (radius[0], center.x, center.y);
                let i_start = x2i(center.x - r);
                let i_end = x2i(center.x + r);
                //
                //         ***          <---  i_start
                //     **       **
                //    **         **
                //     **       **
                //         ***          <---  i_end
                //
                let j_left = |i| { y2j(cy - f32::sqrt(r.powi(2) - (i as f32 - cx).powi(2) )) };
                let j_right = |i| { y2j(cy + f32::sqrt(r.powi(2) - (i as f32 - cx).powi(2) )) };
                // 按行绘制
                for i in i_start..=i_end {
                    canvas.draw_horizontal_line(i, j_left(i), j_right(i), color);
                }
            },

            Shape::Rectangle {p1, p2, color} => {
                // 找到这个矩形的四个边界
                let x_min = f32::min(p1.x, p2.x);
                let x_max = f32::max(p1.x, p2.x);
                let y_min = f32::min(p1.y, p2.y);
                let y_max = f32::max(p1.y, p2.y);
                //
                //  j_left       j_right
                //     * - - - - - *    <---  i_start
                //     |           |
                //     |           |
                //     * - - - - - *    <---  i_end
                //
                let i_start = x2i(x_min);
                let i_end = x2i(x_max);
                let j_left = y2j(y_min);
                let j_right = y2j(y_max);
                // 按行绘制
                for i in i_start..=i_end {
                    canvas.draw_horizontal_line(i, j_left, j_right, color);
                }
            },

            _ => panic!(),
        }
    }
}


/// 个体类. 每个个体就是一些 Shape 的有序列表, 越靠后的图元渲染在越上层
#[derive(Serialize, Deserialize)]
pub struct Individual {
    shapes: Vec<Shape>,
    fitness: Option<f32>,
    env_height: usize,
    env_width: usize,
    bg_color: (f32, f32, f32),
}
impl Individual {
    /// 初始化一个空白个体
    pub fn new(env_height: usize, env_width: usize, bg_color: (f32, f32, f32)) -> Individual {
        Individual {
            shapes: Vec::new(),
            fitness: None,
            env_height,
            env_width,
            bg_color,
        }
    }

    /// 复制一份自己
    pub fn clone(&self) -> Individual {
        let shapes = self.shapes.clone();
        Individual {
            shapes, ..*self
        }
    }

    /// 从 JSON 字符串中读取一个个体
    pub fn from_json(json: &str) -> Individual {
        serde_json::from_str(json).unwrap()
    }

    /// 将自身属性以 JSON 格式输出
    pub fn print_as_json(&self) {
        let json = serde_json::to_string(self).unwrap();
        println!("{}", json);
    }

    /// 个体包含的 Shape 数目
    pub fn n_shapes(&self) -> usize {
        self.shapes.len()
    }

    /// 令个体第 which 个 Shape 进行变异
    pub fn mutate_shape(&mut self, which: usize, canvas_size: usize, amp: f32) {
        debug_assert!(which < self.n_shapes(), "越界的下标!");
        self.shapes[which].mutate(canvas_size, amp);
        self.fitness = None;    // fitness 有待重新计算
    }

    /// 令个体添加一个 Shape. 类型指定, 但属性随机.
    pub fn add_shape(&mut self, type_name: &str, x_height: usize, y_width: usize) {
        self.shapes.push(Shape::rand_new(type_name, x_height, y_width));
        self.fitness = None;    // fitness 有待重新计算
    }

    /// 绘制自身到 Canvas 并返回这个 Canvas
    pub fn draw_self(&self) -> Canvas {
        let mut canvas = Canvas::new(self.env_height, self.env_width, self.bg_color);
        for shape in &self.shapes {
            shape.draw_to(&mut canvas);
        }
        canvas
    }

    /// 计算个体的适应度, 存储到 fitness 字段中
    pub fn calc_fitness(&mut self, target: &Canvas) {
        // 避免重复计算
        if self.fitness.is_some() { return }
        // 绘制自己的图像
        let selfie = self.draw_self();
        // 与目标图片 target 进行逐像素的比对
        let diff = Canvas::l2_diff(&selfie, &target);
        // 存储到 fitness 字段
        self.fitness = Some(diff);
    }

    /// 返回个体的 fitness 字段. 如果是 None 则 panic
    pub fn get_fitness(&self) -> f32 {
        self.fitness.expect("请先显式调用 calc_fitness 计算适应度!")
    }
}
