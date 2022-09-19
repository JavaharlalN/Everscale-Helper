use clap::App;
use clap::Arg;
use clap::ArgMatches;
use serde_json::Value;
use ton_abi::contract::AbiVersion;
use ton_abi::param_type::read_type;
use ton_abi::Function;
use ton_abi::Param;

type Result<T> = std::result::Result<T, String>;

fn main() {
    let matches = match get_matches() {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    let path = matches.value_of("PATH").unwrap();
    let function_json: Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let function = construct_function(
        AbiVersion::parse(matches.value_of("VERSION").unwrap()).unwrap(),
        function_json["name"].as_str().unwrap().to_string(),
        vec![], // TODO: what is header?
        read_params(function_json.clone(), "inputs"),
        read_params(function_json, "outputs"),
    )
    .unwrap();
    let id = calc_function_id(function, false).unwrap();
    println!("id: {}, hex: {:x}", id, id);
}

fn calc_function_id(function: Function, output: bool) -> Result<u32> {
    if output {
        Ok(function.get_output_id())
    } else {
        Ok(function.get_input_id())
    }
}

fn construct_function(
    abi_version: AbiVersion,
    name: String,
    header: Vec<Param>,
    inputs: Vec<Param>,
    outputs: Vec<Param>,
) -> Result<Function> {
    let mut function = Function {
        abi_version,
        name,
        header,
        inputs,
        outputs,
        input_id: 0,
        output_id: 0,
    };
    let id = function.get_function_id();
    function.input_id = id & 0x7FFFFFFF;
    function.output_id = id | 0x80000000;
    Ok(function)
}

fn get_matches() -> Result<ArgMatches> {
    let matches = App::new("Function identifier")
        .author("Javaharlal Nehru")
        .about("Console tool for calculating function id")
        .arg(Arg::new("PATH")
                .help("Path to json file with function.")
                .short('p')
                .long("--path")
                .takes_value(true)
                .required(false)
                .default_value("function.json"),)
        .arg(Arg::new("VERSION")
                .help("Abi version.")
                .short('a')
                .long("--abi-version")
                .takes_value(true)
                .required(false)
                .default_value("2.3"))
        .try_get_matches();

    match matches {
        Ok(m) => return Ok(m),
        Err(e) => return Err(e.to_string()),
    }
}

fn read_params(function_json: Value, field: &str) -> Vec<Param> {
    let params_json = function_json[field].as_array().unwrap();
    let mut params = vec![];
    for v in params_json {
        let name = v["name"].as_str().unwrap();
        let kind = read_type(v["type"].as_str().unwrap()).unwrap();
        params.push(Param {
            name: name.to_string(),
            kind,
        })
    }
    params
}
