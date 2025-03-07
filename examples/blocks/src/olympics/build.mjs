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

import { execSync } from "child_process";
import perspective from "@finos/perspective";
import {
    Uint8ArrayReader,
    ZipReader,
    TextWriter,
    TextReader,
} from "@zip.js/zip.js";

// import * as fs from "node:fs/promises";
import * as url from "url";
import * as fs from "node:fs";

async function main() {
    const __dirname = url
        .fileURLToPath(new URL(".", import.meta.url))
        .slice(0, -1);

    if (fs.existsSync(`${__dirname}/olympics.arrow`)) {
        return;
    }

    execSync(
        `cd ${__dirname} && kaggle datasets download -d heesoo37/120-years-of-olympic-history-athletes-and-results`,
        { stdio: "inherit" }
    );

    const zip = fs.readFileSync(
        `${__dirname}/120-years-of-olympic-history-athletes-and-results.zip`
    );

    const textReader = new TextReader(zip);
    const zipReader = new ZipReader(textReader);
    const entries = await zipReader.getEntries();
    const csv = await entries[0].getData(new TextWriter(), {
        onprogress: (p, t) => console.log(`(${p}b / ${t}b)`),
    });

    zipReader.close();

    const table = await perspective.table(csv);
    const view = await table.view();
    const arrow = await view.to_arrow();
    fs.writeFileSync(`${__dirname}/olympics.arrow`, Buffer.from(arrow));
    fs.unlinkSync(
        `${__dirname}/120-years-of-olympic-history-athletes-and-results.zip`
    );
    await view.delete();
}

main();
