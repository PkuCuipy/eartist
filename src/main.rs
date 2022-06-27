mod util;

use std::cmp::{min, Ordering};
use std::fs::File;
use std::io::*;
use util::*;


fn main() {

    // 读取目标图片
    let target = Canvas::read_from_file("./src/data/target2.jpg");
    let x_height = target.x_height;
    let y_width = target.y_width;
    let canvas_size = min(x_height, y_width);

    // 设定种群超参数
    const BG_COLOR: (f32, f32, f32) = (0., 0., 0.);
    const POP_SIZE: usize = 4;      // 种群大小. 取值范围 [1, ∞)
    const PROP_AMOUNT: usize = 4;   // 每个个体的产仔数. 取值范围 [1, ∞)
    const N_GUARD: usize = 2;       // 上一轮的前 N_GUARD 个个体也参与本轮竞争 (而非产仔后立刻抛弃). 取值范围 [0, POP_SIZE]
    const AMP: f32 = 1.0;           // 变异剧烈程度
    const MUTATE_RATIO: f32 = 0.1;  // 最多多少比例的图形发生变异
    const ADD_SHAPE_PR: f32 = 0.5;  // 每个新个体尝试新增一个图形的概率

    // 创建最初的随机种群
    let mut last_population: Vec<Individual> = Vec::new();
    for _ in 0..POP_SIZE {
        last_population.push(Individual::new(x_height, y_width, BG_COLOR));
    }

    // 开始迭代
    for gen in 1..=1000000 {
        println!("第 {} 轮开始迭代", gen);

        let mut new_generation: Vec<Individual> = Vec::with_capacity(POP_SIZE * PROP_AMOUNT + N_GUARD);

        // 每个个体产生 PROP_AMOUNT 个变异幼崽
        for ind in &last_population {
            for _ in 0..PROP_AMOUNT {
                let mut child = ind.clone();
                // 对自己至多 MUTATE_RATIO 的图形进行突变
                let mutate_amount = random::randint(0, (ind.n_shapes() as f32 * MUTATE_RATIO) as usize + 1);
                for _ in 0..mutate_amount {
                    child.mutate_shape(random::randint(0usize, ind.n_shapes()), canvas_size, AMP);
                }
                // 以 ADD_SHAPE_PR 的概率新增一个图形
                if random::uniform(0., 1.) < ADD_SHAPE_PR {
                    child.add_shape("triangle", x_height, y_width); // FIXME: NOW ONLY TRIANGLE
                }
                new_generation.push(child);
            }
        }

        // 把保底的 N_GUARD 个上一轮个体也添加进去
        for ind in &last_population[0..N_GUARD] {
            new_generation.push(ind.clone());
        }

        // 计算所有个体的适应度
        for ind in &mut new_generation {
            ind.calc_fitness(&target);
        }

        // 把这一代的个体按照适应度进行排序, 留下前 POP_SIZE 个
        new_generation.sort_by(|ind1, ind2| {
            let f1 = ind1.get_fitness();
            let f2 = ind2.get_fitness();
            f1.partial_cmp(&f2).unwrap()
        });
        last_population = new_generation;
        last_population.truncate(POP_SIZE);

        // 打印最优个体的适应度
        let gen_best = &last_population.first().unwrap();
        println!("best fitness = {}", gen_best.get_fitness());
        println!("his n_shapes = {}", gen_best.n_shapes());

        // 保存图像到文件
        if gen % 200 == 0 {
            let canv = gen_best.draw_self();
            canv.write_to_file(format!("./src/result/generation_best/{}.png", gen).as_str());
        }

    }

    // // 从 .json 文件读取个体
    // let mut json_str = String::new();
    // File::open("./src/data/ind_cuipy.json").unwrap().read_to_string(&mut json_str).unwrap();
    // let ind_cuipy = Individual::from_json(json_str.as_str());
    //
    // // 绘制该个体在 canvas 上
    // let mut canvas3 = Canvas::new(200, 200);
    // ind_cuipy.draw_self(&mut canvas3);
    //
    // // 将 canvas 输出到 String
    // let mut output = String::new();
    // canvas3.print_as_ascii(&mut output);
    //
    // // String 输出到文件
    // let mut fout = File::create("./src/result/cuipy_image_ascii.txt").unwrap();
    // write!(fout, "{}", output);

}
