var order_table_glob;
var ascending_glob;

async function generateTableFromJizml(jsonList) {
    var tbody = document.getElementById("display_list_body");

    tbody.innerHTML = '';

    jsonList = await jsonList;
    if (jsonList == null && jsonList.length > 0) {
        return;
    }
    // Get the keys from the first JSON object for the table headers
    var keys = Object.keys(jsonList[0]);
    var key_map = [
        "id",
        "creation_date",
        "build_type",
        "build_location",
        "qc1_initial",
        "qc2_initial",
        "rctpackage",
        "item_serial",
        "asm_serial",
        "oem_serial",
        "make_model",
        "operating_system",
        "mso_installed",
        "processor_type",
        "processor_gen",
        "drive_type",
        "drive_size",
        "ram_type",
        "ram_size"
    ];
    // Loop through each JSON object in the list
    for (var i = 0; i < jsonList.length; i++) {
        var jsonObj = jsonList[i];
        // Create a table row for the JSON object
        var row = document.createElement('tr');

        // Loop through each key and create a table cell
        for (var k = 0; k < key_map.length; k++) {

            var cell = document.createElement('td');
            if (key_map[k] == "creation_date") {
                var time = jsonObj[key_map[k]];
                var formatted = new Date(time).toLocaleDateString("en-US");
                cell.appendChild(document.createTextNode(formatted));
            } else if (key_map[k] == "mso_installed") {
                cell.appendChild(document.createTextNode(jsonObj[key_map[k]] ? "Yes" : "No"));
            } else if (key_map[k] == "id") {
                var a = document.createElement('a');
                var linkText = document.createTextNode(jsonObj[key_map[k]]);
                a.appendChild(linkText);
                a.title = jsonObj[key_map[k]];
                a.href = "/qc_form/" + jsonObj[key_map[k]];
                cell.appendChild(a);
            } else {
                cell.appendChild(document.createTextNode(jsonObj[key_map[k]]));
            }
            row.appendChild(cell);
        }

        var total_passes = 0;
        var total_fails = 0;
        var total_nas = 0;
        var total_incomplete = 0;
        let answers = jsonObj["qc_answers"];
        for (var key in answers) {
            let val = answers[key];
            // console.log(key);
            for(var char_i = 0; char_i < val.length; char_i ++){
                let char = val.charAt(char_i);
                switch (char){
                    case 'p':
                    case 'P':
                        total_passes += 1;
                        break;
                    case 'f':
                    case 'F':
                        total_fails += 1;
                        break;
                    case 'n':
                    case 'N':
                        total_nas += 1;
                        break;
                    case 'i':
                    case 'I':
                        total_incomplete += 1;
                        break;
                }
            }
        }

        var cell = document.createElement('td');
        cell.appendChild(document.createTextNode(total_passes));
        row.appendChild(cell);
        var cell = document.createElement('td');
        cell.appendChild(document.createTextNode(total_fails));
        row.appendChild(cell);
        var cell = document.createElement('td');
        cell.appendChild(document.createTextNode(total_nas));
        row.appendChild(cell);
        var cell = document.createElement('td');
        cell.appendChild(document.createTextNode(total_incomplete));
        row.appendChild(cell);

        // Append the row to the table body
        tbody.appendChild(row);
    }

    updateVisibleComumns()
}

async function order_radio(button) {
    if (button.classList.contains("active")) {
        ascending_glob = button.children[0].getAttribute("ascending");
        order_table_glob = null;
        // I hate this but ya
        setTimeout(function () {
            button.classList.remove("active");
            button.classList.remove("focus");
        }.bind(this), 10);
    } else {
        ascending_glob = button.children[0].getAttribute("ascending");
        order_table_glob = button.parentElement.getAttribute("order_table");
    }
    await make_search();
}

async function make_search() {
    var search_par = document.getElementById("databse_search_parameters").value;
    console.log(search_par);
    var limit = document.getElementById("table_entry_limit").value;
    limit = parseInt(limit);
    limit = (limit == null | limit < 0 | isNaN(limit)) ? 0 : limit;
    var offset = parseInt(document.getElementById("table_entry_page").value) * limit;
    console.log(offset);

    let res = await search(limit, search_par, order_table_glob, ascending_glob, offset);
    console.log(res.status);
    if (res.status != 200){
        console.error(res);
        alert(JSON.stringify(await res.json()));
    }else{
        generateTableFromJizml(await res.json());
    }
}

function updateVisibleComumns(current) {
    var table = document.getElementById("qurry_list");
    var checkboxes = document.getElementsByClassName("row_visibility_button");

    for (var c = 0; c < checkboxes.length; c++) {

        for (var i = 0; i < table.rows.length; i++) {
            var row = table.rows[i];
            if (checkboxes[c].classList.contains("active") ^ (c === current)) {
                row.cells[c + 1].style.display = "";
            } else {
                row.cells[c + 1].style.display = "none";
            }
        }
    }
}
make_search()