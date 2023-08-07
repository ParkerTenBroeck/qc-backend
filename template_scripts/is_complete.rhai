
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
    "creationdate",
    "lastupdated",
    "buildlocation",
    "buildtype",
    "drivetype",
    "itemserial",
    "asmserial",
    "oemserial",
    "makemodel",
    "msoinstalled",
    "operatingsystem",
    "processorgen",
    "processortype",
    "qc1",
    "qc1initial",
    "qc2",
    "qc2initial",
    "ramsize",
    "ramtype",
    "salesorder",
    "drivesize",
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