use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{env, io};

struct Params {
    dir: String,
    output: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageJSON {
    name: String,
    version: String,
    dependencies: Option<Value>,
}

#[derive(Debug, Serialize)]
struct DependencyNode {
    dir: String,
    dependencies: HashMap<String, String>,
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
            output: params_map.get(&String::from("output")).unwrap().clone(),
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
fn traversal_deps(
    package_path: &Path,
    deps_tree: &mut Vec<DependencyNode>,
) -> Result<(), io::Error> {
    // 读取package.json
    let package_json_path = Path::new(package_path).join("package.json");

    if !package_path.exists() {
        println!("file not exists:{:?}", package_json_path);
        return Ok(());
    }
    let mut fs = File::open(package_json_path)?;
    let mut package_json_content = String::from("");
    let mut dependency_node = DependencyNode {
        dir: String::from(package_path.to_str().unwrap()),
        dependencies: HashMap::new(),
    };

    fs.read_to_string(&mut package_json_content);

    let package_json: PackageJSON = serde_json::from_str(package_json_content.as_str()).unwrap();

    if let Some(Value::Object(dependencies)) = &package_json.dependencies {
        for (key, val) in dependencies {
            dependency_node
                .dependencies
                .insert(key.to_string(), val.to_string()); // !

            // println!("key:{}, val:{}", key, val);
            // 继续遍历node_modules中的依赖
            let next_package_path = Path::new(package_path).join("node_modules").join(key);
            println!("next_package_json_path:{:?}", next_package_path);
            traversal_deps(&next_package_path, deps_tree)?;
        }
        deps_tree.append(&mut vec![dependency_node]);
    }
    // !循环一定需要所有权?
    // for map in package_json.dependencies.as_object() {
    //     for (key, val) in map {
    //         println!("key:{}, val:{}", key, val);
    //         // 继续遍历node_modules中的依赖
    //         let next_package_path = Path::new(package_path).join("node_modules").join(key);
    //         println!("next_package_json_path:{:?}", next_package_path);
    //         traversal_deps(&next_package_path);
    //     }
    // }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    let params = Params::new();

    if params.dir.is_empty() || params.output.is_empty() {
        println!("Please input your params: --dir or --output is invalid");
        return Ok(());
    }
    println!("params: {}", params.dir);
    let package_path = Path::new(params.dir.as_str());
    let mut deps_tree: Vec<DependencyNode> = vec![];
    traversal_deps(&package_path, &mut deps_tree)?;

    // println!("deps_tree:{:?}", serde_json::to_string(&deps_tree));

    let output_path = Path::new(&params.output);
    let mut fs = File::create(output_path)?;
    // 将解析文本写入到文件
    let json_str = serde_json::to_string_pretty(&deps_tree)?;
    fs.write(json_str.as_bytes())?;

    Ok(())
}
// cargo run -- --dir /Users/liubeijing/Desktop/code/dx-desktop
