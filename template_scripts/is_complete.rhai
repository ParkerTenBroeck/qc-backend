
let values = params[0];

if values.contains("qc_answers"){
    for val in values.qc_answers.values(){
        if val.contains('f') || val.contains('F'){
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
    "qc_answers",
    "qc1_initial",
    "qc2_initial",
    "ram_size",
    "ram_type",
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

for val in values.qc_answers.values(){
    if val.contains('i') || val.contains('I'){
        return "Incomplete";
    }
}

return ();




















true