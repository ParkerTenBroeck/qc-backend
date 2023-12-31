var order_table_glob;
var ascending_glob;

async function generateTableFromJizml(jsonList) {
    var tbody = document.getElementById("display_list_body");

    tbody.innerHTML = '';

    jsonList = await jsonList;
    if (jsonList == null && jsonList.length > 0) {
        return;
    }

    // Loop through each JSON object in the list
    for (var i = 0; i < jsonList.length; i++) {
        var jsonObj = jsonList[i];

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
        jsonObj.total_passes = total_passes;
        jsonObj.total_fails = total_fails;
        jsonObj.total_incomplete = total_incomplete;
        jsonObj.total_nas = total_nas;

        // Create a table row for the JSON object
        var row = document.createElement('tr');

        // Loop through each key and create a table cell
        for (var k = 0; k < key_map.length; k++) {

            var cell = document.createElement('td');
            let val = key_map[k];
            cell.appendChild(val[1](jsonObj, jsonObj[val[0]]));
            row.appendChild(cell);
        }

        // var cell = document.createElement('td');
        // cell.appendChild(document.createTextNode(total_passes));
        // row.appendChild(cell);
        // var cell = document.createElement('td');
        // cell.appendChild(document.createTextNode(total_fails));
        // row.appendChild(cell);
        // var cell = document.createElement('td');
        // cell.appendChild(document.createTextNode(total_nas));
        // row.appendChild(cell);
        // var cell = document.createElement('td');
        // cell.appendChild(document.createTextNode(total_incomplete));
        // row.appendChild(cell);

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

let search_flag = false;

async function make_search() {

    //this should be made using atomics... whatever whos counting ;)
    let before = search_flag;
    search_flag = true;
    if (before){
        console.log("returned");
        return;
    }

    while(search_flag){
        search_flag = false;
        var search_par = document.getElementById("databse_search_parameters").value;

        let easy_yes_no = document.getElementsByClassName("easy-qurry-yes-no");
        for (let i = 0; i < easy_yes_no.length; i ++){
            let item = easy_yes_no[i];
            let search = "(" + item.getAttribute("value") + ")";
    
            let active_value = item.querySelector(".active");
            if (active_value){
                if (search_par.length != 0){
                    search_par += "&";
                }
                if (active_value.getAttribute("invert") == "true"){
                    search_par += "!";
                }
                search_par += search;
            }
    
        }    
    
        console.log(search_par);
    
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
}

function converte_base2_size_to_base10(size){
    let size_catagory = Math.floor((Math.log(size) / Math.log(2)) / 10)
    let base_value = Math.pow(10, 3*size_catagory);
    let scalar_component = size/Math.pow(2, size_catagory*10);
    return base_value * scalar_component;
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