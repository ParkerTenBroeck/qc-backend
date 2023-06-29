use std::str::Chars;

use diesel::sql_types::TimestamptzSqlite;
use diesel::sqlite::Sqlite;
use rocket::fairing::AdHoc;
use rocket::response::{status::Created, Debug};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Build, Rocket};

use rocket_sync_db_pools::diesel;
use serde_json::Value;
use time::PrimitiveDateTime;

use self::diesel::prelude::*;

#[database("diesel")]
pub struct Db(diesel::SqliteConnection);

impl Db {
    pub async fn validate_user(&self, email: String, password: String) -> Option<i32> {
        self.run(|db| {
            user_accounts::table
                .filter(
                    user_accounts::email
                        .eq(email)
                        .and(user_accounts::password.eq(password)),
                )
                .select(user_accounts::id)
                .first(db)
                .ok()
                .flatten()
        })
        .await
    }
}

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = posts)]
struct Post {
    #[serde(skip_deserializing)]
    id: Option<i32>,
    title: String,
    text: String,
    #[serde(skip_deserializing)]
    published: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = qc_forms)]
struct QCForm {
    #[serde(skip_deserializing)]
    id: Option<i32>,
    #[serde(with = "time::serde::iso8601")]
    #[serde(default = "time_default")]
    assemblydate: time::OffsetDateTime,
    buildlocation: String,
    buildtype: String,
    drivetype: String,
    itemserial: String,
    makemodel: String,
    msoinstalled: String,
    operatingsystem: String,
    processorgen: String,
    processortype: String,
    qc1: String,
    qc1initial: String,
    qc2: String,
    qc2initial: String,

    ramsize: String,
    ramtype: String,
    rctpackage: String,
    salesorder: String,
    technotes: String,
}

fn time_default() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc()
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = user_accounts)]
struct UserAccount {
    #[serde(skip_deserializing)]
    id: Option<i32>,
    email: String,
    password: String,
}

macro_rules! dyn_qc_form_column {
    ($column:expr, $ident:ident, $succ:block, $fail:block) => {
        match $column {
            "id" => {
                let $ident = qc_forms::id;
                $succ
            }
            "assemblydate" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "buildlocation" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "buildtype" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "drivetype" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "itemserial" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "makemodel" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "msoinstalled" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "operatingsystem" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "processorgen" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "processortype" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "qc1" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "qc1initial" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "qc2" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "qc2initial" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }

            "ramsize" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "ramtype" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "rctpackage" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "salesorder" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            "technotes" => {
                let $ident = qc_forms::assemblydate;
                $succ
            }
            _ => $fail,
        }
    };
}

table! {
    qc_forms(id) {
        id -> Nullable<Integer>,
        // assembly_date -> Text,
        assemblydate -> TimestamptzSqlite,
        buildlocation -> Text,
        buildtype -> Text,
        drivetype -> Text,
        itemserial -> Text,
        makemodel -> Text,
        msoinstalled -> Text,
        operatingsystem -> Text,
        processorgen -> Text,
        processortype -> Text,
        qc1 -> Text,
        qc1initial -> Text,
        qc2 -> Text,
        qc2initial -> Text,

        ramsize -> Text,
        ramtype -> Text,
        rctpackage -> Text,
        salesorder -> Text,
        technotes -> Text,
    }
}

table! {
    posts (id) {
        id -> Nullable<Integer>,
        title -> Text,
        text -> Text,
        published -> Bool,
    }
}
table! {
    user_accounts(id){
        id -> Nullable<Integer>,
        email -> Text,
        password -> Text,
    }
}

// #[post("/", data = "<post>")]
// async fn create(db: Db, post: Json<Post>) -> Result<Created<Json<Post>>> {
//     let post_value = post.clone();
//     db.run(move |conn| {
//         diesel::insert_into(posts::table)
//             .values(&*post_value)
//             .execute(conn)
//     })
//     .await?;

//     Ok(Created::new("/").body(post))
// }

#[post("/", data = "<post>")]
async fn create(db: Db, post: Json<QCForm>) -> Result<Created<Json<QCForm>>> {
    let post_value = post.clone();
    db.run(move |conn| {
        diesel::insert_into(qc_forms::table)
            .values(&*post_value)
            .execute(conn)
    })
    .await?;
    Ok(Created::new("/").body(post))
}

// #[get("/")]
// async fn list(db: Db) -> Result<Json<Vec<Option<i32>>>> {
//     let ids: Vec<Option<i32>> = db
//         .run(move |conn| qc_forms::table.select(qc_forms::id).load(conn))
//         .await?;

//     Ok(Json(ids))
// }

#[get("/")]
async fn list(db: Db) -> Result<Json<Vec<QCForm>>> {
    let qc_posts: Vec<QCForm> = db.run(move |conn| qc_forms::table.load(conn)).await?;

    Ok(qc_posts.into())
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
struct TokenizerPosition {
    byte_index: usize,
    char_index: usize,
}

struct Tokenizer<'a> {
    str: &'a str,
    chars: std::iter::Peekable<Chars<'a>>,
    current: TokenizerPosition,
}

impl<'a> Tokenizer<'a> {
    pub fn new(str: &'a str) -> Self {
        Self {
            chars: str.chars().peekable(),
            str,
            current: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TokenizerError{
    InvalidChar(char, TokenizerPosition),
    UnclosedString(TokenizerPosition),
}


impl<'a> Iterator for Tokenizer<'a> {
    type Item = std::result::Result<Token, TokenizerError>;

    fn next(&mut self) -> Option<Self::Item> {
        enum TokenizerState {
            Default,
            Ident,
            String,
        }
        let mut state = TokenizerState::Default;
        let mut current = self.current;
        let mut last = current;
        let mut string_builder = String::new();
        while let Some(char) = self.chars.peek().copied() {
            match state {
                TokenizerState::Default => {
                    let mut ret: Option<Token> = None;
                    match char {
                        '(' => ret = Some(Token::LPar),
                        ')' => ret = Some(Token::RPar),
                        '|' => ret = Some(Token::Or),
                        '&' => ret = Some(Token::And),
                        '>' => ret = Some(Token::Gt),
                        '<' => ret = Some(Token::Lt),
                        '*' => ret = Some(Token::Star),
                        '^' => ret = Some(Token::Carrot),
                        ':' => ret = Some(Token::Colon),
                        ';' => ret = Some(Token::Semicolon),
                        '"' => {
                            state = TokenizerState::String;
                            self.chars.next();
                            current.byte_index += char.len_utf8();
                            current.char_index += 1;
                        },
                        char if char.is_alphabetic() => state = TokenizerState::Ident,
                        char if char.is_whitespace() => {
                            self.chars.next();
                            current.byte_index += char.len_utf8();
                            current.char_index += 1;
                            self.current = current;
                        }
                        bad_char => {
                            let res = Err(TokenizerError::InvalidChar(bad_char, self.current));
                            self.chars.next();
                            current.byte_index += char.len_utf8();
                            current.char_index += 1;
                            self.current = current;
                            return Some(res);
                        }
                    }

                    if ret.is_some() {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        self.current = current;
                        return ret.map(|f|Ok(f));
                    }
                }
                TokenizerState::Ident => {
                    if char.is_alphabetic() {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                    } else {
                        let token = Token::Ident(
                            self.str[self.current.byte_index..current.byte_index].to_owned(),
                        );
                        self.current = last;
                        return Some(Ok(token));
                    }
                }
                TokenizerState::String => {
                    if char == '"' {
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                        self.current = current;

                        return Some(Ok(Token::Value(string_builder)));
                    } else {
                        string_builder.push(char);
                        self.chars.next();
                        current.byte_index += char.len_utf8();
                        current.char_index += 1;
                    }
                }
            }
            last = current;
        }
        match state{
            TokenizerState::Default => None,
            TokenizerState::Ident => {
                let token = Token::Ident(
                    self.str[self.current.byte_index..].to_owned(),
                );
                Some(Ok(token))
            },
            TokenizerState::String => {
                Some(Err(TokenizerError::UnclosedString(self.current)))
            },
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Token {
    LPar,
    RPar,
    Or,
    And,
    Lt,
    Gt,
    Colon,
    Semicolon,
    Carrot,
    Star,
    Ident(String),
    Value(String),
}

#[test]
fn test() {
    let str = "\"pa(lol | $%
    this is a test :: *^<>| hello)paasdas";
    let tokenizer = Tokenizer::new(str);

    let tokens: Vec<_> = tokenizer.collect();
    println!("stuff: {:#?}", tokens);
}

type DynTable = diesel_dynamic_schema::Table<String>;
type DynExpr =
    Box<dyn BoxableExpression<qc_forms::table, Sqlite, SqlType = diesel::sql_types::Bool>>;

#[get("/test/<search>")]
async fn list_search(db: Db, search: &str) -> Result<Json<Vec<QCForm>>> {
    let res: Box<
        dyn BoxableExpression<
            qc_forms::table,
            Sqlite,
            SqlType = diesel::expression::expression_types::NotSelectable,
        >,
    > = dyn_qc_form_column!("test", column, { Box::new(column.asc()) }, { todo!() });
    let mut boxed = qc_forms::table
        .order_by(qc_forms::id.asc())
        .limit(100)
        .into_boxed();

    use diesel_dynamic_schema::table;

    let bruh: diesel::sql_types::TimestamptzSqlite;

    let tabel = table("qc_forms");
    let comumn = tabel.column::<diesel::sql_types::Text, _>("processortype");

    // qc_forms::processortype.
    // let mut boxed_thing = Box::new(qc_forms::processorgen);
    // boxed_thing = Box::new(qc_forms::processortype);

    // boxed = boxed.filter(boxed_thing.like("other"));
    let res: Box<
        dyn BoxableExpression<qc_forms::table, Sqlite, SqlType = diesel::sql_types::Bool>,
    > = Box::new(qc_forms::processortype.like("other"));

    boxed = boxed.filter(res);
    boxed = boxed.filter(qc_forms::processortype.like("other"));
    boxed = boxed.filter(qc_forms::salesorder.like("other"));
    boxed = boxed.filter(qc_forms::salesorder.like("other"));

    let fucked: DynExpr = Box::new(qc_forms::processortype.like("other"));
    let fucked_2 = Box::new(qc_forms::processortype.like("other"));
    let totally_fucked: DynExpr = Box::new(fucked.or(fucked_2));

    boxed = boxed.filter(totally_fucked);

    // let kind = 1;
    // let name = "";
    // let table: diesel_dynamic_schema::Column<diesel_dynamic_schema::Table<qc_forms::table>, &str, diesel::sql_types::Text> = table(qc_forms::table).column::<diesel::sql_types::Text, _>("as");
    // let stupid = match name{
    //     "bruh" => {
    //         qc_forms::assemblydate
    //     },
    //     _ => {
    //         qc_forms::buildtype
    //     }
    // };
    // // None?
    // match kind {
    //     0 => {
    //         tabel
    //     }

    //     _ => {

    //     }
    // }carog

    let qc_posts: Vec<QCForm> = db.run(move |conn| boxed.load(conn)).await?;

    Ok(qc_posts.into())
}

#[get("/<id>")]
async fn read(db: Db, id: i32) -> Option<Json<QCForm>> {
    db.run(move |conn| qc_forms::table.filter(qc_forms::id.eq(id)).first(conn))
        .await
        .map(Json)
        .ok()
}

#[delete("/<id>")]
async fn delete(db: Db, id: i32) -> Result<Option<()>> {
    let affected = db
        .run(move |conn| {
            diesel::delete(posts::table)
                .filter(posts::id.eq(id))
                .execute(conn)
        })
        .await?;

    Ok((affected == 1).then_some(()))
}

#[delete("/")]
async fn destroy(db: Db) -> Result<()> {
    db.run(move |conn| diesel::delete(posts::table).execute(conn))
        .await?;

    Ok(())
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("db/diesel/migrations");

    Db::get_one(&rocket)
        .await
        .expect("database connection")
        .run(|conn| {
            conn.run_pending_migrations(MIGRATIONS)
                .expect("diesel migrations");
        })
        .await;

    rocket
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel SQLite Stage", |rocket| async {
        rocket
            .attach(Db::fairing())
            .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
            .mount(
                "/api",
                routes![list, read, create, delete, destroy, list_search],
            )
    })
}

mod tests {
    use rocket::{http::Status, local::blocking::Client};

    use super::posts;
    use crate::database::Db;
    use diesel::prelude::*;
    use rocket::fairing::AdHoc;
    use rocket::request::FromRequest;
    use rocket::response::{status::Created, Debug};
    use rocket::serde::{json::Json, Deserialize, Serialize};
    use rocket::{Build, Rocket};

    // #[get("/bruh")]
    // async fn destroy(db: Db) -> Result<()> {
    //     db.run(move |conn| diesel::delete(posts::table).execute(conn)).await?;

    //     Ok(())
    // }

    #[test]
    fn database_test() {
        let rocket = rocket::build().attach(super::stage());
        let db: &Db = rocket.state().unwrap();
        db.run(|db| {});
        let client = Client::tracked(rocket).unwrap();
        println!("{:#?}", client.put("/diesel").dispatch().into_string());
        assert_eq!(client.delete("/diesel").dispatch().status(), Status::Ok);
        assert_eq!(
            client.get("/diesel").dispatch().into_json::<Vec<i64>>(),
            Some(vec![])
        );
        // client.
    }
}
