
function getQc1Initial() {
    return document.getElementById("qc1initial").value;
}
function getQc2Initial() {
    return document.getElementById("qc2initial").value;
}
const autoFillOemSerial = new AutoFillSession("all_oem_serial", getQc1Initial, getQc2Initial);
const autoFillItemSerial = new AutoFillSession("all_item_serial", getQc1Initial, getQc2Initial);
const autoFillAsmSerial = new AutoFillSession("all_asm_serial", getQc1Initial, getQc2Initial);




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
    console.log(post);
    
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
    let qc = {};
    Object.entries(json).forEach((entry) => {
        const [key, value] = entry;
        switch (key) {
            case "id":
                var id = parseInt(value);
                if (id != null && id > 0 && !isNaN(id)) {
                    edit_id = id;
                }
                break;
            case "qc1":
                qc["qc1"] = value;
                break;
            case "qc2":
                qc["qc2"] = value;
                break;
            case "metadata":
                metadata = value;
                break
            case "creationdate":
                document.getElementById(key).value = value;
                break;
            default:
                try {
                    document.getElementById(key).value = value;
                    if (key === "buildtype" && json.id == null){
                        update_buildtype();
                    }
                } catch (e) {
                    console.error(e, key, value)
                }
        }
    });

    if (qc.qc1 != null || qc.qc2 != null){
        update_qc_questions(qc, json.id != null);
    }
}

function update_qc_questions(qc, only_show_given){
    
    let questions = document.getElementsByClassName("qc-check-answer");
    
    for(let i = 0; i < questions.length; i ++){
        let question = questions[i];
        let question_id = question.getAttribute("question_id");
        
        if(qc.qc1[question_id] != null){
            let q1 = question.getElementsByClassName('qc1');
            q1[0].checked = false;
            q1[1].checked = false;
            q1[2].checked = false;
            if (qc.qc1[question_id] > 0 && qc.qc1[question_id] <= 3 ){
                q1[qc.qc1[question_id]-1].checked = true;
            }
        }
        if(qc.qc2[question_id] != null){
            let q2 = question.getElementsByClassName('qc2');
            q2[0].checked = false;
            q2[1].checked = false;
            q2[2].checked = false;
            if (qc.qc2[question_id] > 0 && qc.qc2[question_id] <= 3 ){
                q2[qc.qc2[question_id]-1].checked = true;
            }
        }

        if (only_show_given){
            let hide = qc.qc2[question_id] == null && qc.qc2[question_id] == null ;
            let question = document.querySelectorAll("[question_id='"+question_id+"']");
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
    if (qcform.checkValidity() === false){
        qcform.reportValidity();
        return false;
    }
    return true;
}

function form_to_json(){

    let form = collect_qc_questions();
    let form_items = document.getElementsByClassName("qc-form-item");

    for(let i = 0; i < form_items.length; i ++){
        let item = form_items[i];
        if (item.value != null && item.value.length > 0){
            if (item.value != null && item.value.length > 0){
                if (item.id == "msoinstalled"){
                    form[item.id] = item.value == "true";
                }else{
                    form[item.id] = item.value;
                }
            }
        }
    }

    if (form.technotes == null){
        form.technotes = "";
    }
    form.metadata = metadata;

    return form;
}

function collect_qc_questions(){
    let questions = document.getElementsByClassName("qc-check-answer");

    let qc1 = {};
    let qc2 = {};

    for(let i = 0; i < questions.length; i ++){
        let question = questions[i];
        if (question.hasAttribute("hidden")){
            continue;
        }
        let key = question.getAttribute("question_id");
        if (key == null || key.trim().length == 0){
            continue;
        }
        let qc1_val = question.querySelector('.qc1:checked');
        if (qc1_val == null){
            qc1_val = 0;
        }else{
            qc1_val = parseInt(qc1_val.value);
        }
        let qc2_val = question.querySelector('.qc2:checked');
        if (qc2_val == null){
            qc2_val = 0;
        }else{
            qc2_val = parseInt(qc2_val.value);
        }
        qc1[key] = qc1_val;
        qc2[key] = qc2_val;
    }

    let json = {};
    if (Object.keys(qc1).length > 0){
        json.qc1 = qc1;
    }
    if (Object.keys(qc2).length > 0){
        json.qc2 = qc2;
    }
    return json;
}

function update_buildtype() {
    
    let buildtype = document.getElementById("buildtype").value;

    if(!buildtype){
        return;
    }

    let whitelists = document.querySelectorAll("[whitelist_buildtypes]");
    for(let i = 0; i < whitelists.length; i ++){
        let whitelist = whitelists[i];
        let allowed = whitelist.getAttribute("whitelist_buildtypes").split(" ").includes(buildtype.trim());
        
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