diesel::table! {
    qc_forms (id) {
        id -> Nullable<Integer>,
        creation_date -> TimestamptzSqlite,
        last_updated -> TimestamptzSqlite,
        build_location -> Text,
        build_type -> Text,
        drive_type -> Text,
        item_serial -> Text,
        asm_serial -> Nullable<Text>,
        oem_serial -> Text,
        make_model -> Text,
        mso_installed -> Bool,
        operating_system -> Text,
        processor_gen -> Text,
        processor_type -> Text,
        qc1 -> Text,
        qc1_initial -> Text,
        qc2 -> Text,
        qc2_initial -> Nullable<Text>,
        ram_size -> Text,
        ram_type -> Text,
        sales_order -> Nullable<Text>,
        drive_size -> Text,
        tech_notes -> Text,
        metadata -> Nullable<Text>,
    }
}
