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
        "creationdate",
        "buildtype",
        "buildlocation",
        "qc1initial",
        "qc2initial",
        "rctpackage",
        "itemserial",
        "asmserial",
        "oemserial",
        "makemodel",
        "operatingsystem",
        "msoinstalled",
        "processortype",
        "processorgen",
        "drivetype",
        "drivesize",
        "ramtype",
        "ramsize"
    ];
    // Loop through each JSON object in the list
    for (var i = 0; i < jsonList.length; i++) {
        var jsonObj = jsonList[i];
        // Create a table row for the JSON object
        var row = document.createElement('tr');

        // Loop through each key and create a table cell
        for (var k = 0; k < key_map.length; k++) {

            var cell = document.createElement('td');
            if (key_map[k] == "creationdate") {
                var time = jsonObj[key_map[k]];
                var formatted = new Date(time).toLocaleDateString("en-US");
                cell.appendChild(document.createTextNode(formatted));
            } else if (key_map[k] == "msoinstalled") {
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

        var totals = [0, 0, 0, 0];
        var qc_check_1 = jsonObj["qc1"];
        var qc_check_2 = jsonObj["qc2"];

        for (var key in qc_check_1) {
            totals[qc_check_1[key]]++;
        }
        for (var key in qc_check_2) {
            totals[qc_check_2[key]]++;
        }

        var incomplete = totals[0];
        var passes = totals[1];
        var fails = totals[2];
        var nas = totals[3];

        var cell = document.createElement('td');
        cell.appendChild(document.createTextNode(passes));
        row.appendChild(cell);
        var cell = document.createElement('td');
        cell.appendChild(document.createTextNode(fails));
        row.appendChild(cell);
        var cell = document.createElement('td');
        cell.appendChild(document.createTextNode(nas));
        row.appendChild(cell);
        var cell = document.createElement('td');
        cell.appendChild(document.createTextNode(incomplete));
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