defmodule DebianImageFinder do
  @base_url "https://cdimage.debian.org/debian-cd/current-live"


  def get_base_url_for_arch(arch \\ "amd64") do
    "#{@base_url}/#{arch}/iso-hybrid/"
  end

  def get_live_images_list(arch \\ "amd64") do
    url = get_base_url_for_arch(arch)

    case HTTPoison.get(url) do
      {:ok, response} when response.status_code == 200 ->
        response.body
        |> parse_images()

      {:ok, response} ->
        IO.puts("Unexpected status code: #{response.status_code}")
        []

      {:error, reason} ->
        IO.puts("Failed to fetch data from #{url}. Reason: #{inspect(reason)}")
        []
    end
  end

  defp parse_images(html) do
    html
    |> Floki.parse_document!()
    |> Floki.find("table#indexlist tr")
    |> Enum.filter(&is_iso_image/1)
    |> Enum.map(&extract_image_name/1)
  end

  defp is_iso_image(row) do
    row
    |> Floki.find("td.indexcolname a")
    |> Floki.attribute("href")
    |> Enum.any?(&String.ends_with?(&1, ".iso"))
  end

  defp extract_image_name(row) do
    row
    |> Floki.find("td.indexcolname a")
    |> Floki.text()
  end
end
