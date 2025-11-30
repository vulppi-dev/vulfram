import { $ } from "bun";
import { existsSync, mkdirSync, copyFileSync } from "fs";
import { join, resolve } from "path";

const TARGETS = [
  { triple: "x86_64-pc-windows-msvc", arch: "x64", os: "win32", ext: "dll" },
  { triple: "x86_64-unknown-linux-gnu", arch: "x64", os: "linux", ext: "so" },
  { triple: "x86_64-apple-darwin", arch: "x64", os: "darwin", ext: "dylib" },
  { triple: "aarch64-pc-windows-msvc", arch: "arm64", os: "win32", ext: "dll" },
  { triple: "aarch64-unknown-linux-gnu", arch: "arm64", os: "linux", ext: "so" },
  { triple: "aarch64-apple-darwin", arch: "arm64", os: "darwin", ext: "dylib" },
];

const CORE_PATH = resolve(import.meta.dir, "../../core");
const ASSETS_PATH = resolve(import.meta.dir, "../assets");

async function main() {
  console.log("Checking Rust toolchain...");

  // Check installed targets
  const installedTargetsOutput = await $`rustup target list --installed`.text();
  const installedTargets = new Set(installedTargetsOutput.trim().split("\n"));

  for (const target of TARGETS) {
    console.log(`\nProcessing target: ${target.triple} (${target.arch}/${target.os})`);

    // 1. Install target if missing
    if (!installedTargets.has(target.triple)) {
      console.log(`Target ${target.triple} not installed. Installing...`);
      try {
        await $`rustup target add ${target.triple}`;
        console.log("Installed successfully.");
      } catch (e) {
        console.error(`Failed to install target ${target.triple}. Skipping build for this target.`);
        continue;
      }
    } else {
      console.log("Target already installed.");
    }

    // 2. Build
    console.log("Building...");
    try {
      // Run cargo build in the core directory
      // We use --release by default as usually these assets are for distribution/engine usage
      // But maybe we want debug? The user didn't specify, but usually "export dynamic libs" implies release or usable artifacts.
      // Let's stick to release for optimization, or maybe add a flag?
      // User said "quando rodar o build", usually implies the main build process.
      // I'll use --release.
      
      await $`cargo build --release --target ${target.triple}`.cwd(CORE_PATH);
    } catch (e) {
      console.error(`Build failed for ${target.triple}. It might require a cross-compiler linker. Skipping.`);
      continue;
    }

    // 3. Copy artifact
    const artifactName = target.os === "win32" ? "vulfram_core.dll" : 
                         target.os === "darwin" ? "libvulfram_core.dylib" : 
                         "libvulfram_core.so";
    
    const artifactPath = join(CORE_PATH, "target", target.triple, "release", artifactName);
    
    if (!existsSync(artifactPath)) {
        console.error(`Artifact not found at ${artifactPath}`);
        continue;
    }

    const destDir = join(ASSETS_PATH, target.arch, target.os);
    const destFile = join(destDir, `core.${target.ext}`);

    console.log(`Copying to ${destFile}...`);
    
    mkdirSync(destDir, { recursive: true });
    copyFileSync(artifactPath, destFile);
    
    console.log("Success!");
  }
  
  console.log("\nAll tasks completed.");
}

main().catch(console.error);
