// use headless_chrome::types::PrintToPdfOptions;
use rocket::fairing::AdHoc;
use rocket_dyn_templates::context;
use rocket_dyn_templates::Template;
use std::sync::OnceLock;

use crate::database::Db;

type Result<T, E = rocket::response::Debug<diesel::result::Error>> = std::result::Result<T, E>;

#[get("/svg/<id>")]
pub async fn generate_svg(db: Db, id: i32) -> Result<Template> {
    let form = db.get_form(id).await?;

    Ok(Template::render(
        "pdf",
        context! {
            form: form
        },
    ))
}

#[get("/svg2/<id>/form.svg")]
pub async fn generate_svg2(db: Db, id: i32) -> Result<Template> {
    let form = db.get_form(id).await?;

    Ok(Template::render(
        "pdf2",
        context! {
            form_str:  serde_json::to_string_pretty(&form).unwrap(),
            form: form,
        },
    ))
}

use rocket_dyn_templates::Metadata;

// static FONT_DB: OnceLock<fontdb::Database> = OnceLock::new();

// #[get("/svg2pdf/<id>/form.pdf")]
// pub async fn svg_to_pdf(metadata: Metadata<'_>, db: Db, id: i32) -> Result<Vec<u8>> {
//     let form = db.get_form(id).await?;

//     use usvg::{TreeParsing, TreeTextToPath};
//     let svg = metadata
//         .render(
//             "pdf",
//             context! {
//                 form: form
//             },
//         )
//         .unwrap();

//     // Convert string to SVG.
//     let options = usvg::Options::default();

//     let fontdb = FONT_DB.get_or_init(|| {
//         let mut fontdb = fontdb::Database::new();
//         // fontdb::FaceInfo::
//         // fontdb.push_face_info(info);
//         fontdb.load_system_fonts();

//         for faces in fontdb.faces() {
//             println!("{:?}", faces.families);
//         }
//         fontdb
//     });

//     let mut tree = usvg::Tree::from_str(&svg.1, &options).unwrap();
//     tree.convert_text(fontdb);

//     // This can only fail if the SVG is malformed. This one is not.

//     let options = svg2pdf::Options::default();
//     let pdf = svg2pdf::convert_tree(&tree, options);

//     Ok(pdf)
// }

// #[get("/svg2pdf2/<id>/form.pdf")]
// pub async fn svg_to_pdf2(metadata: Metadata<'_>, db: Db, id: i32) -> Result<Vec<u8>> {
//     let form = db.get_form(id).await?;
//     let form_json = serde_json::to_string_pretty(&form).unwrap();
//     let form_json = form_json.split('\n').collect::<Vec<_>>();

//     use usvg::{TreeParsing, TreeTextToPath};
//     let svg = metadata
//         .render(
//             "pdf2",
//             context! {
//                 form,
//                 form_json
//             },
//         )
//         .unwrap();

//     // Convert string to SVG.
//     let options = usvg::Options::default();

//     let fontdb = FONT_DB.get_or_init(|| {
//         let mut fontdb = fontdb::Database::new();
//         // fontdb::FaceInfo::
//         // fontdb.push_face_info(info);
//         fontdb.load_system_fonts();

//         for faces in fontdb.faces() {
//             println!("{:?}", faces.families);
//         }
//         fontdb
//     });

//     let mut tree = usvg::Tree::from_str(&svg.1, &options).unwrap();
//     tree.convert_text(fontdb);

//     // This can only fail if the SVG is malformed. This one is not.

//     let options = svg2pdf::Options::default();
//     let pdf = svg2pdf::convert_tree(&tree, options);

//     Ok(pdf)
// }

// #[get("/chrome/<id>/form.pdf")]
// async fn headless_pdf(id: i32) -> Vec<u8>{
//     use headless_chrome::*;
//     let default = LaunchOptions::default_builder().build().unwrap();
//     let browser = Browser::new(default).unwrap();
//     let tab = browser.wait_for_initial_tab().unwrap();
//     let tab = browser.new_tab().unwrap();
//     let thing = tab.navigate_to(&format!("http://localhost:8000/svg2/{id}/form.svg")).unwrap();
//     thing.wait_until_navigated().unwrap();
//     thing.print_to_pdf(None).unwrap()
// }

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("pdf_gen", |rocket| async {
        rocket.mount(
            "/",
            routes![generate_svg, generate_svg2],
        )
    })
}
