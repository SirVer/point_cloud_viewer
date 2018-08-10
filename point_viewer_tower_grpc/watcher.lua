return {
   {
      should_run = function(path) 
         if path:find("target") ~= nil then return false end
         return path:ext() == "ts" or path:ext() == "html"
      end,
      redirect_stderr = "/tmp/client.err",
      commands = {
         {
            name = "Typescript build [web_viewer]",
            work_directory = "web_viewer/client",
            command = "npm run build",
         },
         {
            name = "Cargo build [web_viewer]",
            work_directory = "web_viewer",
            command = "cargo build --release --color=always",
         },
      }
   },
   {
      should_run = function(path) 
         if path:find("target") ~= nil then return false end
         return path:ext() == "rs" or path:ext() == "toml"
      end,
      environment = {
         CARGO_INCREMENTAL = 1,
      },
      redirect_stderr = "/tmp/cargo.err",
      commands = {
         {
            name = "Cargo build",
            command = "cargo build --release --color=always",
         },
      }
   }
}
