defmodule NetDownloader do
  @moduledoc """
  A module for downloading files from a URL.
  """

  def save(url, file_path) do
   if File.exists?(file_path) do
      IO.puts("File already exist: #{file_path}")
      {:ok, file_path}
    else
      {output, exit_code} = System.cmd("wget", ["-O", file_path, url])

      case exit_code do
        0 -> {:ok, file_path}
        _ -> {:error, "Failed to download file: #{output}"}
      end
    end
  end
end

