use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{env, error::Error, io};

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
    fn new() -> Result<Params, String> {
        let args: Vec<String> = env::args().collect();
        let mut last_key = None; // 记录上次解析出来的key
        let mut params_map: HashMap<String, String> = HashMap::new();
        for val in args.iter() {
            // !args从外面传过来就不能迭代: 原因是new(args())中args()返回是个临时值，执行完成就被销毁了，需要先保存一下
            // let mut val1 = val.clone(); // 复制一份，可能需要优化？
            // --dir /usr/your/path
            if val.starts_with("--") {
                // 提取key 如dir
                // last_key = val1.split_off(2);
                last_key = Some(val.trim_start_matches("--").to_string());
            } else if let Some(key) = &last_key {
                // 是值 放进map中
                params_map.insert(key.clone(), val.clone());
                last_key = None;
            }
        }

        let dir = params_map.get("dir").cloned().unwrap();
        let output = params_map.get("output").cloned().unwrap();

        if dir.is_empty() || output.is_empty() {
            return Err(String::from("Miss params: --dir or --output must input"));
        }

        Ok(Params {
            dir, // 通过clone获取一个新的所有权
            output,
        })
    }
}
// 解析结果
// {
//     dir： "/xxx",
//     dependency: {
//         a: "1.0,0",
//         b: "2.0.0"
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
    let mut package_json_content = String::new();
    let mut dependency_node = DependencyNode {
        dir: String::from(package_path.to_str().unwrap()),
        dependencies: HashMap::new(),
    };

    fs.read_to_string(&mut package_json_content)?;

    let package_json: PackageJSON = serde_json::from_str(package_json_content.as_str())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?; // 通过map_err将错误映射成更友好的形式

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
        deps_tree.push(dependency_node);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let params = Params::new().map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?; // 将错误进行转换

    // if params.dir.is_empty() || params.output.is_empty() {
    //     println!("Please input your params: --dir or --output is invalid");
    //     return Ok(());
    // }
    println!("params: {}", params.dir);
    let package_path = Path::new(params.dir.as_str());
    let mut deps_tree: Vec<DependencyNode> = vec![];
    traversal_deps(&package_path, &mut deps_tree)?;

    let output_path = Path::new(&params.output);
    let mut fs = File::create(output_path)?;
    // 将解析文本写入到文件
    let json_str = serde_json::to_string_pretty(&deps_tree)?;
    fs.write(json_str.as_bytes())?;

    Ok(())
}
// cargo run -- --dir /Users/liubeijing/Desktop/code/dx-desktop
