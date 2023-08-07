diesel::table! {
    qc_forms (id) {
        id -> Nullable<Integer>,
        creationdate -> TimestamptzSqlite,
        lastupdated -> TimestamptzSqlite,
        buildlocation -> Text,
        buildtype -> Text,
        drivetype -> Text,
        itemserial -> Text,
        asmserial -> Nullable<Text>,
        oemserial -> Text,
        makemodel -> Text,
        msoinstalled -> Bool,
        operatingsystem -> Text,
        processorgen -> Text,
        processortype -> Text,
        qc1 -> Text,
        qc1initial -> Text,
        qc2 -> Text,
        qc2initial -> Nullable<Text>,
        ramsize -> Text,
        ramtype -> Text,
        salesorder -> Nullable<Text>,
        drivesize -> Text,
        technotes -> Text,
        metadata -> Nullable<Text>,
    }
}
