// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ ██████ ██████ ██████       █      █      █      █      █ █▄  ▀███ █       ┃
// ┃ ▄▄▄▄▄█ █▄▄▄▄▄ ▄▄▄▄▄█  ▀▀▀▀▀█▀▀▀▀▀ █ ▀▀▀▀▀█ ████████▌▐███ ███▄  ▀█ █ ▀▀▀▀▀ ┃
// ┃ █▀▀▀▀▀ █▀▀▀▀▀ █▀██▀▀ ▄▄▄▄▄ █ ▄▄▄▄▄█ ▄▄▄▄▄█ ████████▌▐███ █████▄   █ ▄▄▄▄▄ ┃
// ┃ █      ██████ █  ▀█▄       █ ██████      █      ███▌▐███ ███████▄ █       ┃
// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
// ┃ Copyright (c) 2017, the Perspective Authors.                              ┃
// ┃ ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ ┃
// ┃ This file is part of the Perspective library, distributed under the terms ┃
// ┃ of the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

use prost::bytes::Bytes;

use crate::proto;
use crate::proto::*;
use crate::view::View;
#[cfg(doc)]
use crate::{Client, Table};

/// The possible formats of input data which [`Client::table`] may take as an
/// argument.
#[derive(Debug)]
pub enum TableData {
    Schema(Vec<(String, ColumnType)>),
    Update(UpdateData),
    View(View),
}

/// The possible formats of input data which [`Table::update`] may take as an
/// argument.
#[derive(Debug)]
pub enum UpdateData {
    Csv(String),
    Arrow(Bytes),
    JsonRows(String),
    JsonColumns(String),
    Ndjson(String),
}

impl From<UpdateData> for TableData {
    fn from(value: UpdateData) -> Self {
        TableData::Update(value)
    }
}

impl From<TableData> for proto::MakeTableData {
    fn from(value: TableData) -> Self {
        let data = match value {
            TableData::Update(x) => return x.into(),
            TableData::View(view) => make_table_data::Data::FromView(view.name),
            TableData::Schema(x) => make_table_data::Data::FromSchema(proto::Schema {
                schema: x
                    .into_iter()
                    .map(|(name, r#type)| schema::KeyTypePair {
                        name,
                        r#type: r#type as i32,
                    })
                    .collect(),
            }),
        };

        MakeTableData { data: Some(data) }
    }
}

impl From<UpdateData> for proto::MakeTableData {
    fn from(value: UpdateData) -> Self {
        let data = match value {
            UpdateData::Csv(x) => make_table_data::Data::FromCsv(x),
            UpdateData::Arrow(x) => make_table_data::Data::FromArrow(x.into()),
            UpdateData::JsonRows(x) => make_table_data::Data::FromRows(x),
            UpdateData::JsonColumns(x) => make_table_data::Data::FromCols(x),
            UpdateData::Ndjson(x) => make_table_data::Data::FromNdjson(x),
        };

        MakeTableData { data: Some(data) }
    }
}
