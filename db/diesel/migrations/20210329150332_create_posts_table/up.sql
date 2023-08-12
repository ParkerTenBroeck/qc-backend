CREATE TABLE qc_forms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    creation_date DATETIME NOT NULL,
    last_updated DATETIME NOT NULL,
    build_location VARCHAR NOT NULL,
    build_type VARCHAR NOT NULL,
    drive_type VARCHAR NOT NULL,
    drive_size VARCHAR NOT NULL,
    item_serial VARCHAR NOT NULL,
    asm_serial VARCHAR,
    oem_serial VARCHAR NOT NULL,
    make_model VARCHAR NOT NULL,
    mso_installed BOOLEAN NOT NULL,
    operating_system VARCHAR NOT NULL,
    processor_gen VARCHAR NOT NULL,
    processor_type VARCHAR NOT NULL,
    qc_answers VARCHAR NOT NULL,
    qc1_initial VARCHAR NOT NULL,
    qc2_initial VARCHAR,

    ram_size VARCHAR NOT NULL,
    ram_type VARCHAR NOT NULL,
    sales_order VARCHAR,
    tech_notes VARCHAR NOT NULL,
    metadata VARCHAR
);