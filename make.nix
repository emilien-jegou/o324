{
  name = "dev";
  desc = "o324 Development Helper";
  scripts = [
    {
      cmd = "build";
      desc = "Build the project";
      exec = "cargo build";
    }
    {
      cmd = "test";
      desc = "Run cargo tests";
      exec = "cargo test";
    }
    {
      cmd = "watch";
      desc = "Run bacon";
      exec = "bacon";
    }
    {
      cmd = "release:prepare";
      desc = "Install dependencies in release folder";
      visible = false;
      dir = "./scripts/release";
      exec = "bun install";
    }
    {
      cmd = "relacher";
      desc = "Generate release commit and git tags";
      deps = ["release:prepare"];
      exec = "bun ./scripts/release/release.ts";
    }
    {
      cmd = "release-github";
      desc = "Generate release patch for github and deploy changes";
      deps = ["release:prepare"];
      exec = "bun ./scripts/release/release-gh.ts";
    }
  ];
}

