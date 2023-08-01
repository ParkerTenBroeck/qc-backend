
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
    return fetch("/api/overwrite_post/" + id, {
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