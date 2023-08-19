
function getqc1_initial() {
    return document.getElementById("qc1_initial").value;
}
function getqc2_initial() {
    return document.getElementById("qc2_initial").value;
}
const autoFilloem_serial = new AutoFillSession("all_oem_serial", getqc1_initial, getqc2_initial);
const autoFillitem_serial = new AutoFillSession("all_item_serial", getqc1_initial, getqc2_initial);
const autoFillasm_serial = new AutoFillSession("all_asm_serial", getqc1_initial, getqc2_initial);

const urlParams = new URLSearchParams(window.location.search);
let download_id_on_save = urlParams.has('download_id_on_save');


const qcform = document.querySelector("#qc-form");


async function save_form() {
    if (!check_form()){
        return;
    }
    let post = await new_post(JSON.stringify(form_to_json()));

    if (post.status != 201){
        alert("Failed to create post");
        console.error(post);
        return;
    }


    post = await post.json();
    
    edit_id = post.id;
    if (download_id_on_save){
        download_id(post.id);
    }

    if (post.id != null){
        window.location.replace("/qc_form/" + post.id);
    }

}

async function update_form() {
    if (!check_form()){
        return;
    }
    let res = await update_post(edit_id, JSON.stringify(form_to_json()));

    if (res.status != 202){
        alert("Failed to update post");
        console.error(res);
    }
    let json = await res.json();
    update_form_values(json);
}

async function load_existing_post() {
    update_form_values(await ((await get_post(edit_id)).json()));
}

function download_id(id){
    var element = document.createElement('a');
    element.setAttribute('href','data:text/plain;charset=utf-8,'+encodeURIComponent(id));
    element.setAttribute('download','qc_form.txt');


    element.style.display = 'none';
    document.body.appendChild(element);

    element.click();

    document.body.removeChild(element);
}

function update_form_values(json) {
    let qc_answers = null;
    Object.entries(json).forEach((entry) => {
        let [key, value] = entry;
        switch (key) {
            case "id":
                var id = parseInt(value);
                if (id != null && id > 0 && !isNaN(id)) {
                    edit_id = id;
                }
                break;
            case "qc_answers":
                qc_answers = value;
                break;
            case "metadata":
                metadata = value;
                break
            case "creation_date":
            case "last_updated":
                
                document.getElementById(key).value = new Date(value).toLocaleString();
                break;
            default:
                try {
                    console.log(typeof value, value);
                    if (typeof value === 'string'){
                        value = value.trim();
                    }
                    document.getElementById(key).value = value;
                    if (key === "build_type" && json.id == null){
                        update_build_type();
                    }
                } catch (e) {
                    console.error(e, key, value)
                }
        }
    });

    if (qc_answers != null){
        update_qc_questions(qc_answers, json.id != null);
    }
}

function update_qc_questions(qc, only_show_given){
    
    let questions = document.getElementsByClassName("qc-check-answer");
    
    for(let i = 0; i < questions.length; i ++){
        let question = questions[i];
        let question_id = question.getAttribute("question_id");
        
        if(qc[question_id] != null){
            let qc1_answer = qc[question_id][0];
            let q1 = question.getElementsByClassName('qc1');
            q1[0].checked = false;
            q1[1].checked = false;
            q1[2].checked = false;
            q1[3].checked = false;
            question.querySelector(".qc1[value='"+qc1_answer+"']").checked = true;

            
            let qc2_answer = qc[question_id][1];
            let q2 = question.getElementsByClassName('qc2');
            q2[0].checked = false;
            q2[1].checked = false;
            q2[2].checked = false;
            q2[3].checked = false;
            question.querySelector(".qc2[value='"+qc2_answer+"']").checked = true;
        }

        if (only_show_given){
            let hide = qc[question_id] == null;
            let question = document.querySelectorAll("[question_id='"+question_id+"']");
            
            console.log(question_id + " " + hide)
            for(let i = 0; i < question.length; i ++){
                if (hide){
                    question[i].setAttribute("hidden", "");
                }else{
                    question[i].removeAttribute("hidden");
                }
            }
        }
    }

    if (only_show_given){
        hide_empty_section();
    }
}

function check_form() {
    if (autoFilloem_serial.is_open() | autoFillitem_serial.is_open() | autoFillasm_serial.is_open()){
        alert("Autofill session active")
        return false;
    }
    if (qcform.checkValidity() === false){
        qcform.reportValidity();
        return false;
    }
    return true;
}

function form_to_json(){

    let form = {};
    form.qc_answers = collect_qc_questions();

    let form_items = document.getElementsByClassName("qc-form-item");

    for(let i = 0; i < form_items.length; i ++){
        let item = form_items[i];
        if (item.value != null && item.value.length > 0){
            if (item.value != null && item.value.length > 0){
                if (item.id == "mso_installed"){
                    form[item.id] = item.value == "true";
                }else{
                    form[item.id] = item.value;
                }
            }
        }else{
            form[item.id] = null;
        }
    }

    if (form.tech_notes == null){
        form.tech_notes = "";
    }
    form.metadata = metadata;

    return form;
}

function collect_qc_questions(){
    let questions = document.getElementsByClassName("qc-check-answer");
    let answers = {};

    for(let i = 0; i < questions.length; i ++){
        let question = questions[i];
        if (question.hasAttribute("hidden")){
            continue;
        }
        let key = question.getAttribute("question_id");
        if (key == null || key.trim().length == 0){
            continue;
        }
        let qc1_val = question.querySelector('.qc1:checked').value;
        if (qc1_val == null){
            qc1_val = "i";
        }
        let qc2_val = question.querySelector('.qc2:checked').value;
        if (qc2_val == null){
            qc2_val = "i";
        }

        answers[key] = qc1_val + qc2_val;
    }
    return answers;
}

function update_build_type() {
    
    let build_type = document.getElementById("build_type").value;

    if(!build_type){
        return;
    }

    let whitelists = document.querySelectorAll("[whitelist_build_types]");
    for(let i = 0; i < whitelists.length; i ++){
        let whitelist = whitelists[i];
        let allowed = whitelist.getAttribute("whitelist_build_types").split(" ").includes(build_type.trim());
        
        if (allowed){
            whitelist.removeAttribute("hidden");
        }else{
            whitelist.setAttribute("hidden", "");
        }
    }
    hide_empty_section();
}

function hide_empty_section(){
    let section = 0;
    while(true){
        let header = document.querySelectorAll("[qc_check_section_heading='"+section+"']");
        if (header.length <= 0){
            break;
        }
        let visible = document.querySelectorAll("[qc_question_section='"+section+"']:not([hidden])");

        if (visible.length > 0){
            for (let i = 0; i < header.length; i ++){
                header[i].removeAttribute("hidden");
            }
        }else{
            for (let i = 0; i < header.length; i ++){
                header[i].setAttribute("hidden", "");
            }
        }
        section += 1;
    }
}