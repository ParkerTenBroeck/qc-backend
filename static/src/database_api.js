
async function new_post(request_body) {
    return fetch("/api/new_post", {
        method: "POST",
        body: request_body,
        headers: {
            "Content-type": "application/json; charset=UTF-8"
        }
    })
}


async function update_post(id, request_body) {
    return fetch("/api/update_post/" + id, {
        method: "POST",
        body: request_body,
        headers: {
            "Content-type": "application/json; charset=UTF-8"
        }
    })
}

async function get_post(post) {
    return fetch("/api/get_post/"+post, {
        method: "GET",
        headers: {
            "Content-type": "application/json; charset=UTF-8"
        }
    })
}

async function search(limit, query, sortby, ascending, offset) {
    return fetch("/api/search", {
        method: "POST",
        mode: "cors",
        body: new URLSearchParams({
            "limit": (limit != null) ? limit : "",
            "search": (query != null) ? query : "",
            "order_table": (sortby != null) ? sortby : "",
            "ascending": ascending,
            "offset": (offset != null) ? offset : "",
        }),
    })
}

function to_db_date(date) {
    let year = date.getUTCFullYear();
    year = (year<0 ? "-" : "+") + Math.abs(year).toString().padStart(6, "0")
    

    let month = (date.getUTCMonth() + 1).toString().padStart(2, "0");
    let day_date = date.getUTCDate().toString().padStart(2, "0");

    let hour = date.getUTCHours().toString().padStart(2, "0");
    let minute = date.getUTCMinutes().toString().padStart(2, "0");
    let seconds = date.getUTCSeconds().toString().padStart(2, "0");

    let milis = date.getUTCMilliseconds().toString().padStart(3, "0");
    let micros = "000000";
    return year + "-" + month + "-" + day_date + "T" + hour + ":" + minute + ":" + seconds + "." + milis + micros + "Z";
}

function start_of_db_date(date) {
    let year = date.getUTCFullYear();
    year = (year<0 ? "-" : "+") + Math.abs(year).toString().padStart(6, "0")
    

    let month = (date.getUTCMonth() + 1).toString().padStart(2, "0");
    let day_date = date.getUTCDate().toString().padStart(2, "0");

    let hour = "00";
    let minute = "00";
    let seconds = "00";

    let milis = "000";
    let micros = "000000";
    return year + "-" + month + "-" + day_date + "T" + hour + ":" + minute + ":" + seconds + "." + milis + micros + "Z";
}

function end_of_db_date(date) {
    let year = date.getUTCFullYear();
    year = (year<0 ? "-" : "+") + Math.abs(year).toString().padStart(6, "0")
    

    let month = (date.getUTCMonth() + 1).toString().padStart(2, "0");
    let day_date = date.getUTCDate().toString().padStart(2, "0");

    let hour = "23";
    let minute = "59";
    let seconds = "59";

    let milis = "999";
    let micros = "999999";
    return year + "-" + month + "-" + day_date + "T" + hour + ":" + minute + ":" + seconds + "." + milis + micros + "Z";
}

function on_date_wildcard(date) {
    let year = date.getUTCFullYear();
    year = (year<0 ? "-" : "+") + Math.abs(year).toString().padStart(6, "0")
    

    let month = (date.getUTCMonth() + 1).toString().padStart(2, "0");
    let day_date = date.getUTCDate().toString().padStart(2, "0");

    // let hour = date.getUTCHours().toString().padStart(2, "0");
    // let minute = date.getUTCMinutes().toString().padStart(2, "0");
    // let seconds = date.getUTCSeconds().toString().padStart(2, "0");

    // let milis = date.getUTCMilliseconds().toString().padStart(3, "0");
    // let micros = "000000";
    return year + "-" + month + "-" + day_date + "%";
}