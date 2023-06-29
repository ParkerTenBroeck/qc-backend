CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title VARCHAR NOT NULL,
    text VARCHAR NOT NULL,
    published BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE qc_forms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    assemblydate VARCHAR NOT NULL,
    buildlocation VARCHAR NOT NULL,
    buildtype VARCHAR NOT NULL,
    drivetype VARCHAR NOT NULL,
    itemserial VARCHAR NOT NULL,
    makemodel VARCHAR NOT NULL,
    msoinstalled VARCHAR NOT NULL,
    operatingsystem VARCHAR NOT NULL,
    processorgen VARCHAR NOT NULL,
    processortype VARCHAR NOT NULL,
    qc1 VARCHAR NOT NULL,
    qc1initial VARCHAR NOT NULL,
    qc2 VARCHAR NOT NULL,
    qc2initial VARCHAR NOT NULL,

    ramsize VARCHAR NOT NULL,
    ramtype VARCHAR NOT NULL,
    rctpackage VARCHAR NOT NULL,
    salesorder VARCHAR NOT NULL,
    technotes VARCHAR NOT NULL
);