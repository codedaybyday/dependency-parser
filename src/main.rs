use std::env;
use std::collections::HashMap;

struct Params {
    dir: String
}

impl Params {
    fn new () -> Params{
        let mut last_key = String::from(""); // 记录上次解析出来的key
        let mut params_map: HashMap<String, String> = HashMap::new();
        for val in env::args() { // !args从外面传过来就不能迭代
            let mut val1 = val.clone(); // 复制一份，可能需要优化？
            println!("x: {:?}", val1);
            // --dir /usr/your/path
            if val1.starts_with("--") {
                // 提取key 如dir
                last_key = val1.split_off(2);
                // println!("last_key:{}", last_key)
            } else if (!last_key.is_empty()) {
                // 是值 放进map中
                params_map.insert(last_key.clone(), val1);
            }
        }

        Params {
            dir: params_map.get(&String::from("dir")).unwrap().clone() // 通过clone获取一个新的所有权
        }
    }
}

fn main() {
    let params = Params::new();

    print!("params1: {}", params.dir);
    // 从参数中获取dir字段
}
