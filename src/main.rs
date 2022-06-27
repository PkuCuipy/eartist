mod util;

use std::cmp::min;
use std::fs::File;
use std::io::*;
use util::*;

/* TODO list:
    - 编译为 Wasm, 在 Web 端实现可调的超参数交互 (main 中的 const 变量绝大多数应实现交互可改)
    - 允许 [定期/交互性] 移除个体中 [面积过小] 的图形
    - 允许 [定期/交互性] 扫描个体的全部图形, 尝试移除 (比如如果移除后 fitness 上升, 则移除之)
*/


fn main() {

    // 读取目标图片
    let target = Canvas::read_from_file("./src/data/target.jpg");
    let x_height = target.x_height;
    let y_width = target.y_width;
    let canvas_size = min(x_height, y_width);

    // 设定种群超参数
    const POP_SIZE: usize = 4;          // 种群大小. 取值范围 [1, ∞)
    const PROP_AMOUNT: usize = 4;       // 每个个体的产仔数. 取值范围 [1, ∞)
    const N_GUARD: usize = 2;           // 上一轮的前 N_GUARD 个个体也参与本轮竞争 (而非产仔后立刻抛弃). 取值范围 [0, POP_SIZE]

    const BG_COLOR: (f32, f32, f32) = (0., 0., 0.);  // 背景色

    const MUTATE_RATIO: f32 = 0.1;      // 最多多少比例的图形发生变异
    const MUTATE_AMP: f32 = 1.0;        // 变异剧烈程度

    const PR_ADD_SHAPE: f32 = 0.5;      // 每个新个体尝试新增一个图形的概率
    const PR_TRIANGLE: f32 = 1.0;       // 使用三角形的概率权重
    const PR_CIRCLE: f32 = 1.0;         // 使用圆形的概率权重
    const PR_RECTANGLE: f32 = 1.0;      // 使用长方形的概率权重
    // 在生成时, 首先按照 PR_ADD_SHAPE 决定 ｢是否生成｣. 如果 ｢是｣, 再根据三个图形的概率权重抽取其中一个进行生成.
    // 这里后三个变量在交互上可以实现为 ｢等边三角图｣
    assert!(PR_TRIANGLE + PR_CIRCLE + PR_RECTANGLE > 0.0, "这三个不能全为 0!");
    assert!(PR_TRIANGLE >= 0.0 && PR_CIRCLE >= 0.0 && PR_RECTANGLE >= 0.0, "概率权重不能为负数!");


    // 创建最初的随机种群
    let mut last_population: Vec<Individual> = Vec::new();
    for _ in 0..POP_SIZE {
        last_population.push(Individual::new(x_height, y_width, BG_COLOR));
    }

    // 开始迭代
    for gen in 1..=100000 {
        println!("第 {} 轮开始迭代", gen);

        let mut new_generation: Vec<Individual> = Vec::with_capacity(POP_SIZE * PROP_AMOUNT + N_GUARD);

        // 每个个体产生 PROP_AMOUNT 个变异幼崽
        for ind in &last_population {
            for _ in 0..PROP_AMOUNT {
                let mut child = ind.clone();
                // 对自己至多 MUTATE_RATIO 的图形进行突变
                let mutate_amount = random::randint(0, (ind.n_shapes() as f32 * MUTATE_RATIO) as usize + 1);
                for _ in 0..mutate_amount {
                    child.mutate_shape(random::randint(0usize, ind.n_shapes()), canvas_size, MUTATE_AMP);
                }
                // 以 ADD_SHAPE_PR 的概率新增一个图形
                if random::uniform(0., 1.) < PR_ADD_SHAPE {     // 决定是否新增一个图形
                    let shape_chosen = random::weighted_choice(
                        &["triangle", "circle", "rectangle"],           // 按照权重随机抽取一个图形
                        &[PR_TRIANGLE, PR_CIRCLE, PR_RECTANGLE]);
                    child.add_shape(shape_chosen);
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

        // 保存图像到文件 (考虑到越到后期越难进化, 保存频率逐渐降低)
        if (gen <= 100) ||
           (gen <= 1000 && gen % 10 == 0) ||
           (gen <= 10000 && gen % 100 == 0) ||
           (gen <= 100000 && gen % 1000 == 0) ||
           (gen % 10000 == 0)
        {
            let canv = gen_best.draw_self();
            canv.write_to_file(format!("./src/result/generation_best/{}.png", gen).as_str());
        }

    }

}



