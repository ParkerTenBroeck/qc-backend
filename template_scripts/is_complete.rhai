
let values = params[0];

if values.contains("qc1"){
    for val in values.qc1.values(){
        if val == 2{
            return "Fail";
        }
    }
}


if values.contains("qc2"){
    for val in values.qc2.values(){
        if val == 2{
            return "Fail";
        }
    }
}

let fields = [
    "creation_date",
    "last_updated",
    "build_location",
    "build_type",
    "drive_type",
    "item_serial",
    "asm_serial",
    "oem_serial",
    "make_model",
    "mso_installed",
    "operating_system",
    "processor_gen",
    "processor_type",
    "qc1",
    "qc1_initial",
    "qc2",
    "qc2_initial",
    "ram_size",
    "ram_type",
    "sales_order",
    "drive_size",
];

for key in fields{
    let value = values.get(key);
    let type = type_of(value);
    switch type{
        "()" => {
            return "Incomplete";
        }
        "string" => {
            if value.len() == 0{
                print(key);
                return "Incomplete";
            }
        }
        "map" => {
            // we check the maps later
        }
        _ => {}
    }
}

if values.qc1.len() != values.qc2.len(){
    return "Incomplete";
}

for val in values.qc1.values(){
    if val == 0{
        return "Incomplete";
    }
}

for val in values.qc2.values(){
    if val == 0{
        return "Incomplete";
    }
}

return ();




















true