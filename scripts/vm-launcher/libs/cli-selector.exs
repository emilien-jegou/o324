defmodule CLISelector do
  def run(choices) do
    loop(%{choices: choices})
  end

  defp loop(%{choices: choices} = state) do
    render(choices)

    IO.write("Please select a number (1-#{length(choices)}): ")
    case IO.gets("") |> String.trim() |> Integer.parse() do
      {choice, ""} when choice in 1..length(choices) -> 
        val = Enum.at(choices, choice - 1)
        [val.key, val.value]
      _ ->
        IO.puts("Invalid choice, please try again.\n")
        loop(state)
    end
  end

  defp render(choices) do
    choices
    |> Enum.with_index(1)
    |> Enum.each(fn {choice, index} ->
      IO.puts(" #{index}. #{choice.key}")
    end)
  end
end

