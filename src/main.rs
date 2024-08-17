use serde::de::value::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{env, io};

struct Params {
    dir: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageJSON {
    name: String,
    version: String,
    dependencies: Value,
}

impl Params {
    fn new() -> Params {
        let mut last_key = String::from(""); // 记录上次解析出来的key
        let mut params_map: HashMap<String, String> = HashMap::new();
        for val in env::args() {
            // !args从外面传过来就不能迭代
            let mut val1 = val.clone(); // 复制一份，可能需要优化？
                                        // --dir /usr/your/path
            if val1.starts_with("--") {
                // 提取key 如dir
                last_key = val1.split_off(2);
            } else if (!last_key.is_empty()) {
                // 是值 放进map中
                params_map.insert(last_key.clone(), val1);
            }
        }

        Params {
            dir: params_map.get(&String::from("dir")).unwrap().clone(), // 通过clone获取一个新的所有权
        }
    }
}
// 解析结果
// {
//     dir： "/xxx",
//     dependency: {
//         a: "",
//         b: ""
//     }
// }
fn traversal_deps(package_path: &Path) -> Result<(), io::Error> {
    // 读取package.json
    let package_json_path = Path::new(package_path).join("package.json");
    let mut fs = File::open(package_json_path)?;
    let mut package_json_content = String::from("");

    fs.read_to_string(&mut package_json_content);

    let package_json: PackageJSON = serde_json::from_str(package_json_content.as_str()).unwrap();

    for map in package_json.dependencies.as_object() {
        for (key, val) in map {
            println!("key:{}, val:{}", key, val);
            // 继续遍历node_modules中的依赖
            let next_package_path = Path::new(package_path).join("node_modules").join(key);
            println!("next_package_json_path:{:?}", next_package_path);
            traversal_deps(&next_package_path);
        }
    }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    let params = Params::new();

    if params.dir.is_empty() {
        println!("Please input your params!");
        return Ok(());
    }
    println!("params: {}", params.dir);
    let package_path = Path::new(params.dir.as_str());

    traversal_deps(&package_path);
    Ok(())
}
// cargo run -- --dir /Users/liubeijing/Desktop/code/dx-desktop
