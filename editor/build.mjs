import { build, context } from "esbuild";
import { mkdir, copyFile, readFile, writeFile } from "node:fs/promises";
await mkdir("build", { recursive: true });
await copyFile("index.html", "build/index.html");
await writeFile(
  "build/licenses.txt",
  (
    await Promise.all(
      ["react", "react-dom", "scheduler"].map(
        async (name) =>
          `${name}\n${await readFile(`node_modules/${name}/LICENSE`, "utf8")}`,
      ),
    )
  ).join("\n\n"),
);
const options = {
  entryPoints: { editor: "src/main.tsx", bridge: "src/bridge.ts" },
  bundle: true,
  minify: true,
  outdir: "build",
  legalComments: "external",
  define: { "process.env.NODE_ENV": '"production"' },
};
if (process.argv.includes("--watch")) {
  const c = await context(options);
  await c.watch();
} else await build(options);
