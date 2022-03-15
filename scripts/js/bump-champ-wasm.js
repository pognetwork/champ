import { readFileSync, writeFileSync } from "fs";
import shell from "shelljs";

const hash = shell.exec("git show -s --format=%h").trim();

shell.cd('../../champ/lib/champ-wasm/pkg');
const pkg = JSON.parse(readFileSync("./package.json"));
let version = pkg.version.split(".").map(i => parseInt(i));
pkg.version = `${version[0]}.${version[1]}.${version[2]}-${hash}`;

writeFileSync("./package.json", JSON.stringify(pkg, null, 2), "utf-8");
