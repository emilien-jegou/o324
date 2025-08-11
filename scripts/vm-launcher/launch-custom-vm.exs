#!/usr/bin/env elixir

Mix.install([
  {:httpoison, "~> 1.8"},
  {:floki, "~> 0.31.0"},
  {:finch, "~> 0.8"}
])

Code.require_file("./libs/debian-image-finder.exs", __DIR__)
Code.require_file("./libs/cli-selector.exs", __DIR__)
Code.require_file("./libs/net-downloader.exs", __DIR__)
Code.require_file("./libs/qemu-runner.exs", __DIR__)

arch = case :os.type() do
  {:unix, :linux} -> System.cmd("uname", ["-m"]) |> elem(0) |> String.trim()
  _ -> "Unsupported OS"
end

if arch != "x86_64" do
  IO.puts("Invalid architecture detected only support amd64: #{arch}")
  System.halt(1);
end

# Get available images for the detected architecture
image_list = DebianImageFinder.get_live_images_list("amd64")

# Early exit if `res` is not a list
unless is_list(image_list) do
  IO.puts("Error: Expected a list of images but got #{inspect(image_list)}.")
  System.halt(1)  # Exit with a non-zero status code to indicate an error
end

labelled_images = Enum.map(image_list, fn str ->
  %{
    key: String.split(str, "-") |> List.last |> String.split(".") |> List.first(),
    value: str
  }
end)

IO.puts("Select a Desktop Environment")
[flavor, selected_image] = CLISelector.run(labelled_images)

download_url = "#{DebianImageFinder.get_base_url_for_arch("amd64")}#{selected_image}"
iso_path =
  "VM_ISO_OUT_PATH"
  |> System.get_env()
  |> (&Path.expand(&1 || ".", File.cwd!())).()
  |> Path.join(["debian-live-#{flavor}.iso"])

result = NetDownloader.save(
  download_url,
  iso_path
)

if match?({:error, _}, result) do
  {:error, reason} = result
  IO.puts("Failed to download debian iso file: #{reason}")
  System.halt(1)
else
  IO.puts("Successful downloaded iso file")
end

QemuRunner.launch(iso_path)

if match?({:error, _}, result) do
  {:error, reason} = result
  IO.puts("Failed to download debian iso file: #{reason}")
  System.halt(1)
end
