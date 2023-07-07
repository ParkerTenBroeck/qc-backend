// @generated automatically by Diesel CLI.

diesel::table! {
    qc_forms (id) {
        id -> Nullable<Integer>,
        assemblydate -> TimestamptzSqlite,
        buildlocation -> Text,
        buildtype -> Text,
        drivetype -> Text,
        itemserial -> Text,
        asmserial -> Text,
        oemserial -> Text,
        makemodel -> Text,
        msoinstalled -> Bool,
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
