

class AutoFillSession {

    ws;
    root;
    input;
    qc1Button;
    qc2Button;
    cancelButton;

    close;
    open;
    error;
    message;

    get_qc1;
    get_qc2;

    constructor(id, get_qc1, get_qc2) {
        this.root = document.getElementById(id);
        this.input = this.root.querySelector("[tag=input]");
        this.qc1Button = this.root.querySelector("[tag=qc1-button]");
        this.qc2Button = this.root.querySelector("[tag=qc2-button]");
        this.cancelButton = this.root.querySelector("[tag=cancel-button]");
        this.ws = null;

        this.get_qc1 = get_qc1;
        this.get_qc2 = get_qc2;

        document.addEventListener("click", (event) => {
            if (this.qc1Button.contains(event.target) | this.qc2Button.contains(event.target) | this.cancelButton.contains(event.target)){
                return;
            }
            if (this.input.hasAttribute("data-readonly") && this.ws == null) {
                this.input.removeAttribute("data-readonly");
                this.input.value = "";
            }
        });

        this.qc1Button.addEventListener("click", () => {
            this.new_websocket(this.get_qc1());
        });

        this.qc2Button.addEventListener("click", async () => {
            await this.close_websocket();
            this.new_websocket(this.get_qc2());
        });

        this.cancelButton.addEventListener("click", async () => {
            await this.close_websocket();
            this.input.value = "";
            this.input.removeAttribute("data-readonly");
        });


        this.close = (event) => { this.event_close(event) };
        this.open = (event) => { this.event_open(event) };
        this.error = (event) => { this.event_error(event) };
        this.message = (event) => { this.event_message(event) };
    }

    async new_websocket(session_id) {
        if (session_id == undefined || session_id == null || session_id.trim().length == 0) {
            this.input.setAttribute("data-readonly", false);
            this.input.value = "Error: Session id cannot be empty";
            return;
        }
        await this.close_websocket();
        this.ws = new WebSocket("ws://" + location.host + "/api/consume_single/" + session_id);

        this.ws.addEventListener("close", this.close);
        this.ws.addEventListener("open", this.open);
        this.ws.addEventListener("error", this.error);
        this.ws.addEventListener("message", this.message);
        this.input.setAttribute("data-readonly", false);
        this.qc1Button.style.visibility = "hidden";
        this.qc2Button.style.visibility = "hidden";
        this.cancelButton.style.visibility = "visible";
    }

    async close_websocket() {
        if (this.ws != null) {
            let tmp_ws = this.ws;

            if (tmp_ws.readyState != WebSocket.CLOSED){
                let promise = close_promise(tmp_ws);
                tmp_ws.close();
                await promise;
            }

            tmp_ws.removeEventListener("close", this.close);
            tmp_ws.removeEventListener("open", this.open);
            tmp_ws.removeEventListener("error", this.error);
            tmp_ws.removeEventListener("message", this.message);
            this.ws = null;
        }

        this.qc1Button.style.visibility = "visible";
        this.qc2Button.style.visibility = "visible";
        this.cancelButton.style.visibility = "hidden";
        this.input.removeAttribute("data-readonly");
    }

    event_open(event) {
        this.input.value = "Connected";
    }

    event_close(event) {
        if (event.code != 1006) {
            console.log(event.reason);
            if (event.reason.trim().length > 0) {
                this.input.value = event.reason;
            } else {
                this.input.value = "Session ended for an unknown reason code: " + event.code;
            }
        } else {
            this.input.removeAttribute("data-readonly");
        }
        this.close_websocket();
    }

    async event_error(event) {
        console.error(event);
        await this.close_websocket();
        this.input.value = "Unknown error in websocket";
    }

    async event_message(event) {
        let json = JSON.parse(event.data);
        if (json.SessionQueuePos != null) {
            this.input.value = ("Connected in Queue: " + json.SessionQueuePos);
        } else if (json.NoSessionQueuePos != null) {
            this.input.value = ("Connecting in Queue: " + json.NoSessionQueuePos);
        } else if (json.Consumed != undefined || json.Consumed != null) {
            this.input.value = json.Consumed;
        }else{
            await this.close_websocket();
            this.input.value = "Malformed Message";
            this.input.setAttribute("data-readonly", false);
        }
    }
}

function close_promise(ws) {
    return new Promise(function (resolve, reject) {
        let close = (event) => {
            ws.removeEventListener("close", close);
            resolve()
        };
        ws.addEventListener("close", close);
        if (ws.readyState == WebSocket.CLOSED) {
            close()
        }
    });
}


class Session {
    constructor(session_id) {
        this.socket = new WebSocket("ws://" + location.host + "/api/open_session/" + session_id);

        this.socket.addEventListener("message", (event) => {
            this.data = JSON.parse(event.data)
        })

        this.socket.addEventListener("close", (event) => {
            this.data = null;
        })
    }

    sendData(data) {
        this.socket.send(data)
    }

    closeSession() {
        this.socket.close()
    }

    data() {
        return this.data;
    }
}