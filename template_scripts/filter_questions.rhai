

let questions = params[0].questions;
let qc_answers = params[1];

let sections;
if params[0].contains("pdf_form") {
    sections = params[0].pdf_form;
}else{
    sections = params[0].tech_form;
}


// removes all questions not saved in our filled out form
// and removes sections if all questions in those sections were removed
for i in range(sections.len() - 1, -1, -1){
    for j in range(sections[i].questions.len() - 1, -1, -1){
        let question = sections[i].questions[j];
        
        if !qc_answers.contains(question){
            sections[i].questions.remove(j);
        }
    }
    if sections[i].questions.len() == 0{
        sections.remove(i);
    }
}


sections